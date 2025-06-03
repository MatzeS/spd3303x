#![feature(pattern)]

use std::str::pattern::{Pattern, Searcher};
use thiserror::Error;

pub mod channel_control;
pub mod commands;
pub mod fixed_channel_control;
pub mod spd3303x;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Received data does not match expected format: {0}")]
    ResponseDecoding(String),
    #[error("Underlying I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to connect: {0}")]
    ConnectFailed(String),
    #[error("Serial mismatch: {0}")]
    SerialMismatch(String),
    #[error("Other: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait ScpiSerialize {
    fn serialize(&self, out: &mut String);
}

pub trait ScpiDeserialize
where
    Self: Sized,
{
    fn deserialize(input: &mut &str) -> Result<Self>;
}

// Rename Query
pub trait ScpiRequest: ScpiSerialize {
    type Response: ScpiDeserialize;
}

impl<T: ScpiSerialize> ScpiSerialize for Option<T> {
    fn serialize(&self, out: &mut String) {
        if let Some(inner) = self {
            inner.serialize(out);
        }
    }
}

pub struct EmptyResponse;
impl ScpiDeserialize for EmptyResponse {
    fn deserialize(_input: &mut &str) -> Result<Self> {
        Ok(EmptyResponse)
    }
}

impl ScpiDeserialize for u16 {
    fn deserialize(input: &mut &str) -> crate::Result<Self> {
        let digits = read_while(input, char::is_numeric);
        let value: u16 = digits
            .parse()
            .map_err(|_| Error::ResponseDecoding(format!("Number parsing failed: {digits}")))?;
        Ok(value)
    }
}

#[macro_export]
macro_rules! impl_scpi_serialize {
    ($type:ty, [ $( $part:tt ),* $(,)? ]) => {
        impl $crate::ScpiSerialize for $type {
            fn serialize(&self, out: &mut String) {
                $(
                    impl_scpi_serialize!(@part self, out, $part);
                )*
            }
        }
    };

    // Handle string literals
    (@part $self:ident, $out:ident, $lit:literal) => {
        $out.push_str($lit);
    };

    // Handle field names
    (@part $self:ident, $out:ident, $field:ident) => {
        $self.$field.serialize($out);
    };
}

#[macro_export]
macro_rules! impl_scpi_request {
    ($request:ty, $response:ty) => {
        impl $crate::ScpiRequest for $request {
            type Response = $response;
        }
    };
}

pub fn match_literal(input: &mut &str, literal: &'static str) -> Result<()> {
    if let Some(rest) = input.strip_prefix(literal) {
        *input = rest;
        Ok(())
    } else {
        Err(Error::ResponseDecoding(format!(
            "Expected literal `{literal}` not matched `{input}`"
        )))
    }
}

pub fn read_until<'a>(input: &mut &'a str, delimiter: char) -> Result<&'a str> {
    if let Some(index) = input.find(delimiter) {
        let (head, tail) = input.split_at(index);
        *input = &tail[1..]; // from 1 to skip delimiter
        Ok(head)
    } else {
        Err(Error::ResponseDecoding(format!(
            "Expected `{delimiter}` in `{input}`"
        )))
    }
}

pub fn read_while<'a, P>(input: &mut &'a str, pattern: P) -> &'a str
where
    P: Pattern,
{
    let mut searcher = pattern.into_searcher(input);

    let split = searcher
        .next_reject()
        .map(|(split, _end)| split)
        .unwrap_or(input.len());

    let (head, tail) = input.split_at(split);
    *input = tail;
    head
}

pub fn read_exact<'a>(input: &mut &'a str, len: usize) -> Result<&'a str> {
    if input.len() < len {
        return Err(Error::ResponseDecoding(format!(
            "Failed to read {len} characters from `{input}`"
        )));
    }

    let (head, tail) = input.split_at(len);
    *input = tail;
    Ok(head)
}

pub fn read_all(input: &mut &str) -> Result<String> {
    Ok(read_until(input, '\n')?.to_string())
}

pub fn check_empty(input: &mut &str) -> Result<()> {
    if input.is_empty() {
        Ok(())
    } else {
        Err(Error::ResponseDecoding(format!(
            "Response should be empty/fully deserialized, but still has content: `{input}`"
        )))
    }
}

#[macro_export]
macro_rules! scpi_enum {
    (
        $(#[$enum_meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident => $literal:expr
            ),* $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        $vis enum $name {
            $(
                $(#[$variant_meta])*
                $variant,
            )*
        }

        impl ScpiDeserialize for $name {
            fn deserialize(input: &mut &str) -> $crate::Result<Self> {
                $(
                    if let Ok(()) = $crate::match_literal(input, $literal) {
                        return Ok(Self::$variant);
                    }
                )*
                Err(Error::ResponseDecoding(format!("Unexpected token for {}: `{}`", stringify!($name), input)))
            }
        }

        impl $crate::ScpiSerialize for $name {
            fn serialize(&self, out: &mut String) {
                match self {
                    $(
                        Self::$variant => out.push_str($literal),
                    )*
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_empty() {
        assert!(check_empty(&mut "").is_ok());
        assert!(check_empty(&mut "x").is_err());
    }

    #[test]
    fn test_read_exact() {
        let input = &mut "1234";
        assert_eq!(read_exact(input, 2).unwrap(), "12");
        assert!(read_exact(input, 3).is_err());
        assert_eq!(read_exact(input, 2).unwrap(), "34");
        assert!(check_empty(input).is_ok());
    }

    #[test]
    fn test_match_literal() {
        let input = &mut "1234";
        assert!(match_literal(input, "12").is_ok());
        assert!(match_literal(input, "12").is_err());
        assert!(match_literal(input, "34").is_ok());
        assert!(check_empty(input).is_ok());
    }

    #[test]
    fn test_read_until() {
        let input = &mut "12,34";
        assert_eq!(read_until(input, ',').unwrap(), "12");
        assert!(match_literal(input, "34").is_ok());
        assert!(check_empty(input).is_ok());
    }

    #[test]
    fn test_read_while() {
        let input = &mut "12,34";
        assert_eq!(read_while(input, char::is_numeric), "12");
        assert!(match_literal(input, ",").is_ok());
        assert_eq!(read_while(input, char::is_numeric), "34");
        assert!(check_empty(input).is_ok());
    }

    #[test]
    fn test_read_all() {
        let input = &mut "12,34\nasdf";
        assert_eq!(read_all(input).unwrap(), "12,34");
        assert!(match_literal(input, "asdf").is_ok());
    }
}
