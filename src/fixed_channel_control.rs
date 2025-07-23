use std::sync::Arc;

use std::sync::Mutex;

use crate::{
    Error, Result,
    commands::{OutputChannel, State},
    spd3303x::Spd3303x,
};

pub struct FixedChannelControl {
    channel: OutputChannel,
    spd: Arc<Mutex<Spd3303x>>,
}

impl FixedChannelControl {
    pub fn new(spd: Arc<Mutex<Spd3303x>>, channel: OutputChannel) -> Self {
        FixedChannelControl { spd, channel }
    }

    pub fn set_output(&self, state: State) -> Result<()> {
        let mut spd = self.spd.lock().map_err(|e| Error::Other(e.to_string()))?;
        spd.set_output(self.channel, state)
    }
}
