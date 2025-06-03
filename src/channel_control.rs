use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
    Result,
    commands::{
        Channel, GetTimingParametersResponse, LimitQuantity, Quantity, Reading, State,
        TimeInterval, TimingGroup,
    },
    fixed_channel_control::FixedChannelControl,
    spd3303x::Spd3303x,
};

pub struct ChannelControl {
    channel: Channel,
    spd: Arc<Mutex<Spd3303x>>,
}

impl ChannelControl {
    pub fn new(spd: Arc<Mutex<Spd3303x>>, channel: Channel) -> Self {
        ChannelControl { spd, channel }
    }

    pub async fn measure(&self, quantity: Quantity) -> Result<f32> {
        let mut spd = self.spd.lock().await;
        spd.measure(self.channel, quantity).await
    }

    pub async fn set_limit(&self, quantity: LimitQuantity, value: Reading) -> Result<()> {
        let mut spd = self.spd.lock().await;
        spd.set_limit(self.channel, quantity, value).await
    }

    pub async fn get_limit(&self, quantity: LimitQuantity) -> Result<f32> {
        let mut spd = self.spd.lock().await;
        spd.get_limit(self.channel, quantity).await
    }

    pub async fn set_output(&self, state: State) -> Result<()> {
        let mut spd = self.spd.lock().await;
        spd.set_output(self.channel.into(), state).await
    }

    pub async fn get_output(&self) -> Result<State> {
        let mut spd = self.spd.lock().await;
        spd.get_output(self.channel).await
    }

    pub async fn set_waveform_display(&self, state: State) -> Result<()> {
        let mut spd = self.spd.lock().await;
        spd.set_waveform_display(self.channel, state).await
    }

    pub async fn set_timing_parameters(
        &self,
        group: TimingGroup,
        voltage: Reading,
        current: Reading,
        time: TimeInterval,
    ) -> Result<()> {
        let mut spd = self.spd.lock().await;
        spd.set_timing_parameters(self.channel, group, voltage, current, time)
            .await
    }

    pub async fn get_timing_parameters(
        &self,
        group: TimingGroup,
    ) -> Result<GetTimingParametersResponse> {
        let mut spd = self.spd.lock().await;
        spd.get_timing_parameters(self.channel, group).await
    }

    pub async fn set_timer(&self, state: State) -> Result<()> {
        let mut spd = self.spd.lock().await;
        spd.set_timer(self.channel, state).await
    }

    pub fn to_fixed(self) -> FixedChannelControl {
        self.into()
    }
}

impl From<ChannelControl> for FixedChannelControl {
    fn from(value: ChannelControl) -> Self {
        FixedChannelControl::new(value.spd, value.channel.into())
    }
}
