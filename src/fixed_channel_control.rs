use std::sync::Arc;

use std::sync::Mutex;

use crate::{
    Error, Result,
    commands::{OutputChannel, State},
    spd3303x::{Driver, Spd3303x},
};

pub struct FixedChannelControl<D: Driver> {
    channel: OutputChannel,
    spd: Arc<Mutex<Spd3303x<D>>>,
}

impl<D: Driver> FixedChannelControl<D> {
    pub fn new(spd: Arc<Mutex<Spd3303x<D>>>, channel: OutputChannel) -> Self {
        FixedChannelControl { spd, channel }
    }

    pub fn set_output(&self, state: State) -> Result<()> {
        let mut spd = self.spd.lock().map_err(|e| Error::Other(e.to_string()))?;
        spd.set_output(self.channel, state)
    }
}
