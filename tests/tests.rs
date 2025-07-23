use anyhow::anyhow;
use spd3303x::{
    Result,
    channel_control::ChannelControl,
    commands::{Channel, LimitQuantity, MemorySlot, OperationMode, Quantity, State},
    spd3303x::Spd3303x,
};

fn test_device() -> Result<Spd3303x> {
    let hostname = std::env::var("TEST_SPD3303X")
        .map_err(|e| anyhow!("Environment variable TEST_SPD3303X not set! `{e}`"))?;

    let power_supply = Spd3303x::connect_hostname(hostname.as_str())?;
    Ok(power_supply)
}

fn test_channel() -> Result<ChannelControl> {
    let spd = test_device()?;
    Ok(spd.into_channels().0)
}

#[test]
fn test_identity() -> Result<()> {
    // This obviously only works with one specific device
    let mut spd = test_device()?;
    let identity = spd.get_identity()?;
    assert_eq!(identity.company_name, "Siglent Technologies");
    assert_eq!(identity.model_number, "SPD3303X");
    assert_eq!(identity.serial_number, "SPD3XJGQ805993");
    assert_eq!(identity.software_version, "1.01.01.03.11R1");
    assert_eq!(identity.hardware_version, "V6.2");
    Ok(())
}

#[test]
fn test_save_recall() -> Result<()> {
    let mut spd = test_device()?;

    spd.set_limit(Channel::One, LimitQuantity::Current, 1.0.into())?;
    spd.save(MemorySlot::One)?;

    spd.set_limit(Channel::One, LimitQuantity::Current, 2.0.into())?;
    spd.save(MemorySlot::Two)?;

    assert_eq!(spd.get_limit(Channel::One, LimitQuantity::Current)?, 2.0);

    spd.recall(MemorySlot::One)?;
    assert_eq!(spd.get_limit(Channel::One, LimitQuantity::Current)?, 1.0);

    spd.recall(MemorySlot::Two)?;
    assert_eq!(spd.get_limit(Channel::One, LimitQuantity::Current)?, 2.0);

    Ok(())
}

#[test]
fn test_measure() -> Result<()> {
    let channel = test_channel()?;

    channel.set_limit(LimitQuantity::Voltage, 1.337.into())?;
    channel.set_output(State::Off)?;
    assert_eq!(channel.measure(Quantity::Voltage)?, 0.0);
    channel.set_output(State::On)?;
    assert!(channel.measure(Quantity::Voltage)? > 1.250);
    channel.set_output(State::Off)?;

    Ok(())
}

#[test]
fn test_limit() -> Result<()> {
    let channel = test_channel()?;

    channel.set_limit(LimitQuantity::Voltage, 1.337.into())?;
    assert_eq!(channel.get_limit(LimitQuantity::Voltage)?, 1.337);

    channel.set_limit(LimitQuantity::Voltage, 2.337.into())?;
    assert_eq!(channel.get_limit(LimitQuantity::Voltage)?, 2.337);

    Ok(())
}

#[test]
fn test_output() -> Result<()> {
    let channel = test_channel()?;

    channel.set_output(State::On)?;
    assert_eq!(channel.get_output()?, State::On);

    channel.set_output(State::Off)?;
    assert_eq!(channel.get_output()?, State::Off);

    Ok(())
}

#[test]
fn test_operation_mode() -> Result<()> {
    let mut spd = test_device()?;
    spd.set_output_mode(OperationMode::Independent)?;
    assert_eq!(spd.get_status()?.operation_mode, OperationMode::Independent);

    spd.set_output_mode(OperationMode::Parallel)?;
    assert_eq!(spd.get_status()?.operation_mode, OperationMode::Parallel);

    spd.set_output_mode(OperationMode::Series)?;
    assert_eq!(spd.get_status()?.operation_mode, OperationMode::Series);

    spd.set_output_mode(OperationMode::Independent)?;
    assert_eq!(spd.get_status()?.operation_mode, OperationMode::Independent);

    Ok(())
}
