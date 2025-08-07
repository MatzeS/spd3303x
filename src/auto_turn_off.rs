use std::ops::{Deref, DerefMut};

use crate::{
    channel_control::ChannelControl,
    commands::{OutputChannel, State},
    fixed_channel_control::FixedChannelControl,
    spd3303x::Spd3303x,
};

trait TurnOff {
    fn turn_off(&mut self);
}

#[allow(private_bounds)]
pub struct AutoTurnOff<T>
where
    T: TurnOff,
{
    inner: Option<T>,
}

#[allow(private_bounds)]
impl<T> AutoTurnOff<T>
where
    T: TurnOff,
{
    pub fn new(inner: T) -> Self {
        Self { inner: Some(inner) }
    }

    pub fn into_inner(mut self) -> T {
        self.inner.take().unwrap()
    }
}

impl<T> Deref for AutoTurnOff<T>
where
    T: TurnOff,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap()
    }
}

impl<T> DerefMut for AutoTurnOff<T>
where
    T: TurnOff,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().unwrap()
    }
}

impl<T> Drop for AutoTurnOff<T>
where
    T: TurnOff,
{
    fn drop(&mut self) {
        if let Some(mut e) = self.inner.take() {
            e.turn_off()
        }
    }
}

impl TurnOff for Spd3303x {
    fn turn_off(&mut self) {
        let _ = self.set_output(OutputChannel::One, State::Off);
        let _ = self.set_output(OutputChannel::Two, State::Off);
        let _ = self.set_output(OutputChannel::Three, State::Off);
    }
}

impl TurnOff for ChannelControl {
    fn turn_off(&mut self) {
        let _ = self.set_output(State::Off);
    }
}

impl TurnOff for FixedChannelControl {
    fn turn_off(&mut self) {
        let _ = self.set_output(State::Off);
    }
}

#[cfg(test)]
#[allow(clippy::bool_assert_comparison)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::auto_turn_off::{AutoTurnOff, TurnOff};

    impl TurnOff for Rc<RefCell<bool>> {
        fn turn_off(&mut self) {
            let mut e = self.borrow_mut();
            *e = false;
        }
    }

    #[test]
    fn turn_off_on_drop() {
        let on = Rc::new(RefCell::new(true));
        {
            let on = AutoTurnOff::new(on.clone());
            assert_eq!(*on.borrow(), true);
        }
        assert_eq!(*on.borrow(), false);
    }

    #[test]
    fn into_inner_does_not_turn_off_on_drop() {
        let on = Rc::new(RefCell::new(true));
        {
            let on = AutoTurnOff::new(on.clone());
            assert_eq!(*on.borrow(), true);

            let _ = on.into_inner();
        }
        assert_eq!(*on.borrow(), true);
    }
}
