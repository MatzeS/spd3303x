use anyhow::anyhow;
use serial_test::serial;
use spd3303x::{
    Result,
    channel_control::ChannelControl,
    commands::{Channel, LimitQuantity, MemorySlot, OperationMode, Quantity, State},
    spd3303x::{Driver, NetworkDriver, Spd3303x, UsbDriver},
};

fn test_network_device() -> Result<Spd3303x<NetworkDriver>> {
    let hostname = std::env::var("TEST_SPD3303X")
        .map_err(|e| anyhow!("Environment variable TEST_SPD3303X not set! `{e}`"))?;

    let driver = NetworkDriver::connect_hostname(hostname.as_str())?;
    let power_supply = Spd3303x { driver };
    Ok(power_supply)
}

fn test_usb_device() -> Result<Spd3303x<UsbDriver>> {
    let driver = UsbDriver::connect_device()?;
    Ok(Spd3303x { driver })
}

fn test_channel_network() -> Result<ChannelControl<NetworkDriver>> {
    let spd = test_network_device()?;
    Ok(spd.into_channels().0)
}

fn test_channel_usb() -> Result<ChannelControl<UsbDriver>> {
    let spd = test_usb_device()?;
    Ok(spd.into_channels().0)
}

fn run_identity_test<D: Driver>(mut spd: Spd3303x<D>) -> Result<()> {
    let identity = spd.get_identity()?;
    assert_eq!(identity.company_name, "Siglent Technologies");
    assert_eq!(identity.model_number, "SPD3303X");
    assert_eq!(identity.serial_number, "SPD3XJGQ805993");
    assert_eq!(identity.software_version, "1.01.01.03.11R1");
    assert_eq!(identity.hardware_version, "V6.2");
    Ok(())
}

#[test]
#[serial]
fn test_identity_network() -> Result<()> {
    // This obviously only works with one specific device
    let spd = test_network_device()?;
    run_identity_test(spd)
}

#[test]
#[serial]
fn test_identity_usb() -> Result<()> {
    // This obviously only works with one specific device
    let spd = test_usb_device()?;
    run_identity_test(spd)
}

fn run_save_recall_test<D: Driver>(mut spd: Spd3303x<D>) -> Result<()> {
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
#[serial]
fn test_save_recall_network() -> Result<()> {
    let spd = test_network_device()?;
    run_save_recall_test(spd)
}

#[test]
#[serial]
fn test_save_recall_usb() -> Result<()> {
    let spd = test_usb_device()?;
    run_save_recall_test(spd)
}

fn run_measure_test<D: Driver>(channel: ChannelControl<D>) -> Result<()> {
    channel.set_limit(LimitQuantity::Voltage, 1.337.into())?;
    channel.set_output(State::Off)?;
    assert_eq!(channel.measure(Quantity::Voltage)?, 0.0);
    channel.set_output(State::On)?;
    assert!(channel.measure(Quantity::Voltage)? > 1.250);
    channel.set_output(State::Off)?;

    Ok(())
}

#[test]
#[serial]
fn test_measure_network() -> Result<()> {
    let channel = test_channel_network()?;
    run_measure_test(channel)
}

#[test]
#[serial]
fn test_measure_usb() -> Result<()> {
    let channel = test_channel_usb()?;
    run_measure_test(channel)
}

fn run_limit_test<D: Driver>(channel: ChannelControl<D>) -> Result<()> {
    channel.set_limit(LimitQuantity::Voltage, 1.337.into())?;
    assert_eq!(channel.get_limit(LimitQuantity::Voltage)?, 1.337);

    channel.set_limit(LimitQuantity::Voltage, 2.337.into())?;
    assert_eq!(channel.get_limit(LimitQuantity::Voltage)?, 2.337);

    Ok(())
}

#[test]
#[serial]
fn test_limit_network() -> Result<()> {
    let channel = test_channel_network()?;
    run_limit_test(channel)
}

#[test]
#[serial]
fn test_limit_usb() -> Result<()> {
    let channel = test_channel_usb()?;
    run_limit_test(channel)
}

fn run_output_test<D: Driver>(channel: ChannelControl<D>) -> Result<()> {
    channel.set_output(State::On)?;
    assert_eq!(channel.get_output()?, State::On);

    channel.set_output(State::Off)?;
    assert_eq!(channel.get_output()?, State::Off);

    Ok(())
}

#[test]
#[serial]
fn test_output_network() -> Result<()> {
    let channel = test_channel_network()?;
    run_output_test(channel)
}

#[test]
#[serial]
fn test_output_usb() -> Result<()> {
    let channel = test_channel_usb()?;
    run_output_test(channel)
}

fn run_operation_mode_test<D: Driver>(mut spd: Spd3303x<D>) -> Result<()> {
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

#[test]
#[serial]
fn test_operation_mode_network() -> Result<()> {
    let spd = test_network_device()?;
    run_operation_mode_test(spd)
}

#[test]
#[serial]
fn test_operation_mode_usb() -> Result<()> {
    let spd = test_usb_device()?;
    run_operation_mode_test(spd)
}
