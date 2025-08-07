use std::sync::Arc;

use std::sync::Mutex;

use crate::auto_turn_off::AutoTurnOff;
use crate::{
    Error, Result,
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

    pub fn measure(&self, quantity: Quantity) -> Result<f32> {
        let mut spd = self.spd.lock().map_err(|e| Error::Other(e.to_string()))?;
        spd.measure(self.channel, quantity)
    }

    pub fn set_limit(&self, quantity: LimitQuantity, value: Reading) -> Result<()> {
        let mut spd = self.spd.lock().map_err(|e| Error::Other(e.to_string()))?;
        spd.set_limit(self.channel, quantity, value)
    }

    pub fn get_limit(&self, quantity: LimitQuantity) -> Result<f32> {
        let mut spd = self.spd.lock().map_err(|e| Error::Other(e.to_string()))?;
        spd.get_limit(self.channel, quantity)
    }

    pub fn set_output(&self, state: State) -> Result<()> {
        let mut spd = self.spd.lock().map_err(|e| Error::Other(e.to_string()))?;
        spd.set_output(self.channel.into(), state)
    }

    pub fn get_output(&self) -> Result<State> {
        let mut spd = self.spd.lock().map_err(|e| Error::Other(e.to_string()))?;
        spd.get_output(self.channel)
    }

    pub fn set_waveform_display(&self, state: State) -> Result<()> {
        let mut spd = self.spd.lock().map_err(|e| Error::Other(e.to_string()))?;
        spd.set_waveform_display(self.channel, state)
    }

    pub fn set_timing_parameters(
        &self,
        group: TimingGroup,
        voltage: Reading,
        current: Reading,
        time: TimeInterval,
    ) -> Result<()> {
        let mut spd = self.spd.lock().map_err(|e| Error::Other(e.to_string()))?;
        spd.set_timing_parameters(self.channel, group, voltage, current, time)
    }

    pub fn get_timing_parameters(&self, group: TimingGroup) -> Result<GetTimingParametersResponse> {
        let mut spd = self.spd.lock().map_err(|e| Error::Other(e.to_string()))?;
        spd.get_timing_parameters(self.channel, group)
    }

    pub fn set_timer(&self, state: State) -> Result<()> {
        let mut spd = self.spd.lock().map_err(|e| Error::Other(e.to_string()))?;
        spd.set_timer(self.channel, state)
    }

    pub fn to_fixed(self) -> FixedChannelControl {
        self.into()
    }

    pub fn into_auto_turn_off(self) -> AutoTurnOff<Self> {
        AutoTurnOff::new(self)
    }
}

impl From<ChannelControl> for FixedChannelControl {
    fn from(value: ChannelControl) -> Self {
        FixedChannelControl::new(value.spd, value.channel.into())
    }
}
