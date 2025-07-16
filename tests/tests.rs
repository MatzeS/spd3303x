use anyhow::anyhow;
use spd3303x::{
    Result,
    channel_control::ChannelControl,
    commands::{Channel, LimitQuantity, MemorySlot, OperationMode, Quantity, State},
    spd3303x::{Driver, NetworkDriver, Spd3303x, UsbDriver},
};

async fn test_network_device() -> Result<Spd3303x<NetworkDriver>> {
    let hostname = std::env::var("TEST_SPD3303X")
        .map_err(|e| anyhow!("Environment variable TEST_SPD3303X not set! `{e}`"))?;

    let driver = NetworkDriver::connect_hostname(hostname.as_str()).await?;
    let power_supply = Spd3303x { driver };
    Ok(power_supply)
}

async fn test_usb_device(vid: u16, pid: u16) -> Result<Spd3303x<UsbDriver>> {
    let driver = UsbDriver::connect_device(vid, pid)?;
    Ok(Spd3303x { driver })
}

async fn test_channel_network() -> Result<ChannelControl<NetworkDriver>> {
    let spd = test_network_device().await?;
    Ok(spd.into_channels().0)
}

async fn test_channel_usb() -> Result<ChannelControl<UsbDriver>> {
    let spd = test_usb_device(0xf4ec, 0x1430).await?;
    Ok(spd.into_channels().0)
}

async fn run_identity_test<D: Driver>(mut spd: Spd3303x<D>) -> Result<()> {
    let identity = spd.get_identity().await?;
    assert_eq!(identity.company_name, "Siglent Technologies");
    assert_eq!(identity.model_number, "SPD3303X");
    assert_eq!(identity.serial_number, "SPD3XJGQ805993");
    assert_eq!(identity.software_version, "1.01.01.03.11R1");
    assert_eq!(identity.hardware_version, "V6.2");
    Ok(())
}

#[tokio::test]
async fn test_identity_network() -> Result<()> {
    // This obviously only works with one specific device
    let spd = test_network_device().await?;
    run_identity_test(spd).await
}

#[tokio::test]
async fn test_identity_usb() -> Result<()> {
    // This obviously only works with one specific device
    let spd = test_usb_device(0xf4ec, 0x1430).await?;
    run_identity_test(spd).await
}

async fn run_save_recall_test<D: Driver>(mut spd: Spd3303x<D>) -> Result<()> {
    spd.set_limit(Channel::One, LimitQuantity::Current, 1.0.into())
        .await?;
    spd.save(MemorySlot::One).await?;

    spd.set_limit(Channel::One, LimitQuantity::Current, 2.0.into())
        .await?;
    spd.save(MemorySlot::Two).await?;

    assert_eq!(
        spd.get_limit(Channel::One, LimitQuantity::Current).await?,
        2.0
    );

    spd.recall(MemorySlot::One).await?;
    assert_eq!(
        spd.get_limit(Channel::One, LimitQuantity::Current).await?,
        1.0
    );

    spd.recall(MemorySlot::Two).await?;
    assert_eq!(
        spd.get_limit(Channel::One, LimitQuantity::Current).await?,
        2.0
    );

    Ok(())
}

#[tokio::test]
async fn test_save_recall_network() -> Result<()> {
    let spd = test_network_device().await?;
    run_save_recall_test(spd).await
}

#[tokio::test]
async fn test_save_recall_usb() -> Result<()> {
    let spd = test_usb_device(0xf4ec, 0x1430).await?;
    run_save_recall_test(spd).await
}

async fn run_measure_test<D: Driver>(channel: ChannelControl<D>) -> Result<()> {
    channel
        .set_limit(LimitQuantity::Voltage, 1.337.into())
        .await?;
    channel.set_output(State::Off).await?;
    assert_eq!(channel.measure(Quantity::Voltage).await?, 0.0);
    channel.set_output(State::On).await?;
    assert!(channel.measure(Quantity::Voltage).await? > 1.250);
    channel.set_output(State::Off).await?;

    Ok(())
}

#[tokio::test]
async fn test_measure_network() -> Result<()> {
    let channel = test_channel_network().await?;
    run_measure_test(channel).await
}

#[tokio::test]
async fn test_measure_usb() -> Result<()> {
    let channel = test_channel_usb().await?;
    run_measure_test(channel).await
}

async fn run_limit_test<D: Driver>(channel: ChannelControl<D>) -> Result<()> {
    channel
        .set_limit(LimitQuantity::Voltage, 1.337.into())
        .await?;
    assert_eq!(channel.get_limit(LimitQuantity::Voltage).await?, 1.337);

    channel
        .set_limit(LimitQuantity::Voltage, 2.337.into())
        .await?;
    assert_eq!(channel.get_limit(LimitQuantity::Voltage).await?, 2.337);

    Ok(())
}

#[tokio::test]
async fn test_limit_network() -> Result<()> {
    let channel = test_channel_network().await?;
    run_limit_test(channel).await
}

#[tokio::test]
async fn test_limit_usb() -> Result<()> {
    let channel = test_channel_usb().await?;
    run_limit_test(channel).await
}

async fn run_output_test<D: Driver>(channel: ChannelControl<D>) -> Result<()> {
    channel.set_output(State::On).await?;
    assert_eq!(channel.get_output().await?, State::On);

    channel.set_output(State::Off).await?;
    assert_eq!(channel.get_output().await?, State::Off);

    Ok(())
}

#[tokio::test]
async fn test_output_network() -> Result<()> {
    let channel = test_channel_network().await?;
    run_output_test(channel).await
}

#[tokio::test]
async fn test_output_usb() -> Result<()> {
    let channel = test_channel_usb().await?;
    run_output_test(channel).await
}

async fn run_operation_mode_test<D: Driver>(mut spd: Spd3303x<D>) -> Result<()> {
    spd.set_output_mode(OperationMode::Independent).await?;
    assert_eq!(
        spd.get_status().await?.operation_mode,
        OperationMode::Independent
    );

    spd.set_output_mode(OperationMode::Parallel).await?;
    assert_eq!(
        spd.get_status().await?.operation_mode,
        OperationMode::Parallel
    );

    spd.set_output_mode(OperationMode::Series).await?;
    assert_eq!(
        spd.get_status().await?.operation_mode,
        OperationMode::Series
    );

    spd.set_output_mode(OperationMode::Independent).await?;
    assert_eq!(
        spd.get_status().await?.operation_mode,
        OperationMode::Independent
    );

    Ok(())
}

#[tokio::test]
async fn test_operation_mode_network() -> Result<()> {
    let spd = test_network_device().await?;
    run_operation_mode_test(spd).await
}

#[tokio::test]
async fn test_operation_mode_usb() -> Result<()> {
    let spd = test_usb_device(0xf4ec, 0x1430).await?;
    run_operation_mode_test(spd).await
}
