use spd3303x::{
    Error, Result,
    commands::{LimitQuantity, Quantity, Reading, State},
    spd3303x::Spd3303x,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let hostname = std::env::var("TEST_SPD3303X")
        .map_err(|e| Error::Other(format!("Environment variable TEST_SPD3303X not set! `{e}`")))?;
    let serial_number = std::env::var("TEST_SPD3303X_SERIAL").map_err(|e| {
        Error::Other(format!(
            "Environment variable TEST_SPD3303X_SERIAL not set! `{e}`"
        ))
    })?;

    let mut power_supply = Spd3303x::connect_hostname(hostname.as_str()).await?;

    // Serial verification is recommended, to ensure
    // you are not accidentally connecting to the wrong device.
    power_supply
        .verify_serial_number(serial_number.as_str())
        .await?;

    let (ch1, _ch2, ch3) = power_supply.into_channels();

    ch1.set_limit(LimitQuantity::Voltage, Reading::from(1.000))
        .await?;
    ch1.set_limit(LimitQuantity::Current, Reading::from(0.1))
        .await?;

    let voltage = ch1.measure(Quantity::Voltage).await?;
    println!("V {voltage}");

    ch3.set_output(State::On).await?;
    ch3.set_output(State::Off).await?;

    ch1.set_output(State::On).await?;
    ch1.set_output(State::Off).await?;

    Ok(())
}
