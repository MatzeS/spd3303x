use spd3303x::{
    Error, Result,
    channel_control::ChannelControl,
    commands::{Channel, LimitQuantity, MemorySlot, Quantity, State},
    spd3303x::Spd3303x,
};

async fn test_device() -> Result<Spd3303x> {
    let hostname = std::env::var("TEST_SPD3303X")
        .map_err(|e| Error::Other(format!("Environment variable TEST_SPD3303X not set! `{e}`")))?;

    let power_supply = Spd3303x::connect_hostname(hostname.as_str()).await?;
    Ok(power_supply)
}

async fn test_channel() -> Result<ChannelControl> {
    let spd = test_device().await?;
    Ok(spd.into_channels().0)
}

#[tokio::test]
async fn test_identity() -> Result<()> {
    // This obviously only works with one specific device
    let mut spd = test_device().await?;
    let identity = spd.get_identity().await?;
    assert_eq!(identity.company_name, "Siglent Technologies");
    assert_eq!(identity.model_number, "SPD3303X");
    assert_eq!(identity.serial_number, "SPD3XJGQ805993");
    assert_eq!(identity.software_version, "1.01.01.03.11R1");
    assert_eq!(identity.hardware_version, "V6.2");
    Ok(())
}

#[tokio::test]
async fn test_save_recall() -> Result<()> {
    let mut spd = test_device().await?;

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
async fn test_measure() -> Result<()> {
    let channel = test_channel().await?;

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
async fn test_limit() -> Result<()> {
    let channel = test_channel().await?;

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
async fn test_output() -> Result<()> {
    let channel = test_channel().await?;

    channel.set_output(State::On).await?;
    assert_eq!(channel.get_output().await?, State::On);

    channel.set_output(State::Off).await?;
    assert_eq!(channel.get_output().await?, State::Off);

    Ok(())
}
