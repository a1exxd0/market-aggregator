pub mod traded_instruments;

use std::sync::Weak;

use traded_instruments::Instrument;

use crate::exchange_connectivity::ExchangeType;

pub trait Order {
    fn new(instrument: Instrument, exchange: ExchangeType, quantity: usize, price: f64) -> Self;
    fn quantity(&self) -> usize;
    fn price(&self) -> f64;
    fn instrument(&self) -> Instrument;
    fn exchange(&self) -> ExchangeType;
}

#[derive(Debug)]
pub struct Bid {
    instrument: Instrument,
    exchange: ExchangeType,
    quantity: usize,
    price: f64,
}

impl Order for Bid {
    fn new(instrument: Instrument, exchange: ExchangeType, quantity: usize, price: f64) -> Self {
        Bid {
            instrument: instrument,
            exchange: exchange,
            quantity: quantity,
            price: price,
        }
    }

    fn quantity(&self) -> usize {
        self.quantity
    }

    fn price(&self) -> f64 {
        self.price
    }

    fn instrument(&self) -> Instrument {
        self.instrument
    }

    fn exchange(&self) -> ExchangeType {
        self.exchange
    }
}

#[derive(Debug)]
pub struct Ask {
    quantity: usize,
    price: f64,
    instrument: Instrument,
    exchange: ExchangeType,
}

impl Order for Ask {
    fn new(instrument: Instrument, exchange: ExchangeType, quantity: usize, price: f64) -> Self {
        Ask {
            instrument: instrument,
            exchange: exchange,
            quantity: quantity,
            price: price,
        }
    }

    fn quantity(&self) -> usize {
        self.quantity
    }

    fn price(&self) -> f64 {
        self.price
    }

    fn instrument(&self) -> Instrument {
        self.instrument
    }

    fn exchange(&self) -> ExchangeType {
        self.exchange
    }
}
