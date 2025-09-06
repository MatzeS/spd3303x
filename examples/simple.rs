use anyhow::anyhow;
use spd3303x::{
    Result,
    commands::{LimitQuantity, Quantity, Reading, State},
    device_selector::DeviceSelector,
    spd3303x::Spd3303x,
};

fn main() -> Result<()> {
    let selector = std::env::var("TEST_SPD3303X")
        .map_err(|e| anyhow!("Environment variable TEST_SPD3303X not set! `{e}`"))?;
    let selector = selector.parse::<DeviceSelector>().map_err(|e| anyhow!(e))?;
    let power_supply = Spd3303x::connect(selector)?;

    // Auto turn off ensures the channel is turned off, when the channel is dropped.
    let (ch1, _ch2, ch3) = power_supply.into_auto_turn_off_channels();

    ch1.set_limit(LimitQuantity::Voltage, Reading::from(1.000))?;
    ch1.set_limit(LimitQuantity::Current, Reading::from(0.1))?;

    let voltage = ch1.measure(Quantity::Voltage)?;
    println!("V {voltage}");

    ch3.set_output(State::On)?;
    ch3.set_output(State::Off)?;

    ch1.set_output(State::On)?;
    // Due to auto turnoff, ch1 will be turned off before exiting.

    Ok(())
}
