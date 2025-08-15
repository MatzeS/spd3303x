use std::sync::Arc;

use std::sync::Mutex;

use anyhow::anyhow;

use crate::{
    Result,
    commands::{
        Channel, GetTimingParametersResponse, LimitQuantity, Quantity, Reading, State,
        TimeInterval, TimingGroup,
    },
    fixed_channel_control::FixedChannelControl,
    spd3303x::{Driver, Spd3303x},
};

pub struct ChannelControl<D: Driver> {
    channel: Channel,
    spd: Arc<Mutex<Spd3303x<D>>>,
}

impl<D: Driver> ChannelControl<D> {
    pub fn new(spd: Arc<Mutex<Spd3303x<D>>>, channel: Channel) -> Self {
        ChannelControl { spd, channel }
    }

    pub fn measure(&self, quantity: Quantity) -> Result<f32> {
        let mut spd = self.spd.lock().map_err(|e| anyhow!("{e}"))?;
        spd.measure(self.channel, quantity)
    }

    pub fn set_limit(&self, quantity: LimitQuantity, value: Reading) -> Result<()> {
        let mut spd = self.spd.lock().map_err(|e| anyhow!("{e}"))?;
        spd.set_limit(self.channel, quantity, value)
    }

    pub fn get_limit(&self, quantity: LimitQuantity) -> Result<f32> {
        let mut spd = self.spd.lock().map_err(|e| anyhow!("{e}"))?;
        spd.get_limit(self.channel, quantity)
    }

    pub fn set_output(&self, state: State) -> Result<()> {
        let mut spd = self.spd.lock().map_err(|e| anyhow!("{e}"))?;
        spd.set_output(self.channel.into(), state)
    }

    pub fn get_output(&self) -> Result<State> {
        let mut spd = self.spd.lock().map_err(|e| anyhow!("{e}"))?;
        spd.get_output(self.channel)
    }

    pub fn set_waveform_display(&self, state: State) -> Result<()> {
        let mut spd = self.spd.lock().map_err(|e| anyhow!("{e}"))?;
        spd.set_waveform_display(self.channel, state)
    }

    pub fn set_timing_parameters(
        &self,
        group: TimingGroup,
        voltage: Reading,
        current: Reading,
        time: TimeInterval,
    ) -> Result<()> {
        let mut spd = self.spd.lock().map_err(|e| anyhow!("{e}"))?;
        spd.set_timing_parameters(self.channel, group, voltage, current, time)
    }

    pub fn get_timing_parameters(&self, group: TimingGroup) -> Result<GetTimingParametersResponse> {
        let mut spd = self.spd.lock().map_err(|e| anyhow!("{e}"))?;
        spd.get_timing_parameters(self.channel, group)
    }

    pub fn set_timer(&self, state: State) -> Result<()> {
        let mut spd = self.spd.lock().map_err(|e| anyhow!("{e}"))?;
        spd.set_timer(self.channel, state)
    }

    pub fn to_fixed(self) -> FixedChannelControl<D> {
        self.into()
    }
}

impl<D: Driver> From<ChannelControl<D>> for FixedChannelControl<D> {
    fn from(value: ChannelControl<D>) -> Self {
        FixedChannelControl::new(value.spd, value.channel.into())
    }
}
