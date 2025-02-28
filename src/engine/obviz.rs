use crate::utils::repeat;
use std::fmt;

use crate::engine::OrderBook;

impl fmt::Display for OrderBook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // let my_string = concat!("First Line\n", "Second Line",);
        // println!("Here is love for you: {}", repeat(10, '♥'));
        const MAX_LEN: usize = 20;
        let mut str = String::new();
        let mut max_qty: u32 = 0;
        let mut max_price: u32 = 0;
        let mut min_qty: u32 = u32::MAX;
        let mut qty: u32;
        let mut bid_viz: Vec<(u32, u32)> = vec![];
        let mut ask_viz: Vec<(u32, u32)> = vec![];
        for (price, _idx) in self.ask_book.price_map.iter().rev() {
            qty = self.ask_book.get_total_qty(*price);
            if qty > 0 {
                min_qty = min_qty.min(qty);
                max_qty = max_qty.max(qty);
                max_price = max_price.max(*price);
                ask_viz.push((*price, qty));
            }
        }
        for (price, _idx) in self.bid_book.price_map.iter().rev() {
            qty = self.bid_book.get_total_qty(*price);
            if qty > 0 {
                min_qty = min_qty.min(qty);
                max_qty = max_qty.max(qty);
                max_price = max_price.max(*price);
                bid_viz.push((*price, qty));
            }
        }
        if max_qty != 0 {
            let digits_num = max_price.ilog10() + 1;
            let step_size = max_qty / u32::try_from(MAX_LEN).expect("Casting failed miserably") + 1;
            for (price, qty) in &ask_viz {
                let size = 1.max(qty / step_size) as usize;
                let line = format!(
                    "[  OB  ]{}{:>width$}[{}]\n",
                    repeat(MAX_LEN + 2, " "),
                    price,
                    repeat(size, "#"),
                    width = digits_num as usize,
                )
                .to_string();
                str.push_str(&line);
            }
            for (price, qty) in &bid_viz {
                let size = 1.max(qty / step_size) as usize;
                let line = format!(
                    "[  OB  ]{}[{}]{:>width$}\n",
                    repeat(MAX_LEN - size, " "),
                    repeat(size, "#"),
                    price,
                    width = digits_num as usize,
                )
                .to_string();
                str.push_str(&line);
            }

            write!(f, "{str}")
        } else {
            write!(f, "OB is empty")
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::engine::{Order, Side};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_viz() {
        let mut ob = OrderBook::new();
        ob.add_limit_order(Order {
            id: 1,
            price: 100,
            side: Side::Bid,
            qty: 5,
        });
        ob.add_limit_order(Order {
            id: 2,
            price: 101,
            side: Side::Bid,
            qty: 4,
        });
        ob.add_limit_order(Order {
            id: 3,
            price: 102,
            side: Side::Bid,
            qty: 3,
        });
        ob.add_limit_order(Order {
            id: 4,
            price: 103,
            side: Side::Bid,
            qty: 2,
        });
        ob.add_limit_order(Order {
            id: 5,
            price: 104,
            side: Side::Bid,
            qty: 1,
        });
        assert_eq!(
            "[  OB  ]                   [#]104
[  OB  ]                  [##]103
[  OB  ]                 [###]102
[  OB  ]                [####]101
[  OB  ]               [#####]100
",
            ob.to_string()
        );
    }
}
