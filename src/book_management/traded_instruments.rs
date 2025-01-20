use std::fmt::{self, Debug, Display};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Instrument {
    BtcUsdt,
    EthUsdc,
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
            Instrument::EthUsdc => write!(f, "ETH_USDC"),
        }
    }
}
