use crate::engine::Ticker;

#[allow(unused)]
pub const IMOEXF: Ticker = Ticker {
    ticker_id: 1,
    tick_size: 1.0,
    step_price: 1.0,
    taker_fee: 0.000_066,
    maker_fee: 0.0,
};

#[allow(unused)]
pub const MM: Ticker = Ticker {
    ticker_id: 2,
    tick_size: 5.0,
    step_price: 0.5,
    taker_fee: 0.000_066,
    maker_fee: 0.0,
};

#[allow(unused)]
pub const ALRS: Ticker = Ticker {
    ticker_id: 3,
    tick_size: 1.0,
    step_price: 0.1,
    taker_fee: 0.0,
    maker_fee: 0.0,
};
