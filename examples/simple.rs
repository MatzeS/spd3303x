use anyhow::anyhow;
use spd3303x::{
    Result,
    commands::{LimitQuantity, Quantity, Reading, State},
    spd3303x::{NetworkDriver, Spd3303x},
};

fn main() -> Result<()> {
    let hostname = std::env::var("TEST_SPD3303X")
        .map_err(|e| anyhow!("Environment variable TEST_SPD3303X not set! `{e}`"))?;
    let serial_number = std::env::var("TEST_SPD3303X_SERIAL")
        .map_err(|e| anyhow!("Environment variable TEST_SPD3303X_SERIAL not set! `{e}`"))?;

    let driver = NetworkDriver::connect_hostname(hostname.as_str())?;
    let mut power_supply = Spd3303x { driver };

    // Serial number verification is recommended, to ensure
    // you are not accidentally connecting to the wrong device.
    power_supply.verify_serial_number(serial_number.as_str())?;

    let (ch1, _ch2, ch3) = power_supply.into_channels();

    ch1.set_limit(LimitQuantity::Voltage, Reading::from(1.000))?;
    ch1.set_limit(LimitQuantity::Current, Reading::from(0.1))?;

    let voltage = ch1.measure(Quantity::Voltage)?;
    println!("V {voltage}");

    ch3.set_output(State::On)?;
    ch3.set_output(State::Off)?;

    ch1.set_output(State::On)?;
    ch1.set_output(State::Off)?;

    Ok(())
}
