use crate::orderbook::OrderBook;

pub enum Indicator {
    Midprice,
}

impl Indicator {
    pub fn evaluate<'a>(&self, ob: &'a OrderBook) -> Result<f32, &'a str> {
        match self {
            Indicator::Midprice => {
                let (bid, ask, _spread) = ob.get_bbo()?;
                Ok(midprice(bid, ask))
            }
        }
    }
}

pub fn midprice(bid: u64, ask: u64) -> f32 {
    (bid + ask) as f32 / 2.0
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::orderbook::{Order, Side};
    use pretty_assertions::assert_eq;
    use rstest::rstest;

    fn empty_ob() -> OrderBook {
        OrderBook::new("Indicator test".to_string())
    }

    fn full_ob(bid: u64, ask: u64) -> OrderBook {
        let mut ob = OrderBook::new("Indicator test".to_string());
        let buy_order = Order {
            id: 666,
            side: Side::Bid,
            price: bid,
            qty: 10,
        };
        ob.add_limit_order(buy_order);
        let sell_order = Order {
            id: 999,
            side: Side::Ask,
            price: ask,
            qty: 10,
        };
        ob.add_limit_order(sell_order);
        ob
    }

    #[rstest]
    #[case(empty_ob(), Err("Both bid and offer HalfBooks are empty"))]
    #[case(full_ob(99, 101), Ok(100.0))]
    fn midprice_test(#[case] input: OrderBook, #[case] expected: Result<f32, &str>) {
        let midprice = Indicator::Midprice;
        assert_eq!(midprice.evaluate(&input), expected);
    }
}
