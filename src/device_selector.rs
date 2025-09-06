use std::{num::ParseIntError, str::FromStr};
use thiserror::Error;

const SCPI_DEFAULT_PORT: u16 = 5025;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommunicationInterface {
    Usb,
    TcpIp { hostname: String, port: u16 },
}

/// A device selector expressing how to connect to a SPD3303X and what serial number to expect.
///
/// Intended to be be parsed from string (typically user input).
/// Examples: "usb", "usb/SPDserialnum", "192.168.0.42/SPDserialnum", "192.168.0.42:1337"
///
/// Slash (/) separates the communication interface from the expected serial number.
/// The expected serial number is optional.
/// Colon (:) can be used to provide an optional port number for TCP/IP.
/// If none is provided, the default SCPI port is assumed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceSelector {
    pub communication_interface: CommunicationInterface,
    pub serial_number: Option<String>,
}

#[derive(Debug, Clone, Error)]
pub enum ParsingError {
    #[error("Failed to parse port: {0}")]
    FailedToParsePort(ParseIntError),
}

impl FromStr for CommunicationInterface {
    type Err = ParsingError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input == "usb" {
            return Ok(CommunicationInterface::Usb);
        }

        const PORT_SEPARATOR: &str = ":";
        if let Some(separator) = input.find(PORT_SEPARATOR) {
            let (hostname, port) = split_at_discard(input, separator);
            let hostname = hostname.to_string();
            let port: u16 = port.parse().map_err(ParsingError::FailedToParsePort)?;
            return Ok(CommunicationInterface::TcpIp { hostname, port });
        }

        Ok(CommunicationInterface::TcpIp {
            hostname: input.to_string(),
            port: SCPI_DEFAULT_PORT,
        })
    }
}

impl FromStr for DeviceSelector {
    type Err = ParsingError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        const INTERFACE_SERIAL_NUMBER_SEPARATOR: &str = "/";
        if let Some(separator) = input.find(INTERFACE_SERIAL_NUMBER_SEPARATOR) {
            let (interface, serial_number) = split_at_discard(input, separator);
            return Ok(Self {
                communication_interface: interface.parse()?,
                serial_number: Some(serial_number.to_string()),
            });
        }

        Ok(Self {
            communication_interface: input.parse()?,
            serial_number: None,
        })
    }
}

fn split_at_discard(s: &str, mid: usize) -> (&str, &str) {
    let (left, right) = s.split_at(mid);
    (left, &right[1..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_at() {
        assert_eq!(split_at_discard("0123456", 3), ("012", "456"));
    }

    #[test]
    fn device_selector_usb() {
        assert_eq!(
            DeviceSelector {
                communication_interface: CommunicationInterface::Usb,
                serial_number: None,
            },
            "usb".parse().unwrap()
        );
    }

    #[test]
    fn device_selector_usb_and_serial() {
        assert_eq!(
            DeviceSelector {
                communication_interface: CommunicationInterface::Usb,
                serial_number: Some("asdf".to_string()),
            },
            "usb/asdf".parse().unwrap()
        );
    }

    #[test]
    fn device_selector_host_and_port() {
        assert_eq!(
            DeviceSelector {
                communication_interface: CommunicationInterface::TcpIp {
                    hostname: "host".to_string(),
                    port: 1234
                },
                serial_number: None,
            },
            "host:1234".parse().unwrap()
        );
    }

    #[test]
    fn device_selector_host_port_and_serial() {
        assert_eq!(
            DeviceSelector {
                communication_interface: CommunicationInterface::TcpIp {
                    hostname: "host".to_string(),
                    port: 1234
                },
                serial_number: Some("serial".to_string()),
            },
            "host:1234/serial".parse().unwrap()
        );
    }

    #[test]
    fn device_selector_host_and_serial() {
        assert_eq!(
            DeviceSelector {
                communication_interface: CommunicationInterface::TcpIp {
                    hostname: "host".to_string(),
                    port: 5025
                },
                serial_number: Some("serial".to_string()),
            },
            "host/serial".parse().unwrap()
        );
    }

    #[test]
    fn device_selector_host() {
        assert_eq!(
            DeviceSelector {
                communication_interface: CommunicationInterface::TcpIp {
                    hostname: "host".to_string(),
                    port: 5025
                },
                serial_number: None,
            },
            "host".parse().unwrap()
        );
    }
}
