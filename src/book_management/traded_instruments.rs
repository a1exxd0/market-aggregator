use std::fmt::{self, Debug, Display};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Instrument {
    BtcUsdt,
}

impl Debug for Instrument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return <dyn Display>::fmt(self, f);
    }
}

impl fmt::Display for Instrument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instrument::BtcUsdt => write!(f, "BTC_USDT"),
        }
    }
}
