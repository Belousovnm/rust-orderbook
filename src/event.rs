use crate::matching_engine::{Order, Side};
use crate::snap::Snap;
use serde::de::Deserializer;
use serde::de::Error;
use serde::de::MapAccess;
use serde::de::Visitor;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, Serialize, Default)]
pub struct LimitOrder {
    pub side: Side,
    pub price: u32,
    pub qty: u32,
}

impl<'de> Deserialize<'de> for Snap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EventVisitor;

        impl<'de> Visitor<'de> for EventVisitor {
            type Value = Snap;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Event")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut exch_epoch: Option<u64> = None;
                let mut bid_1_price: Option<u32> = None;
                let mut bid_2_price: Option<u32> = None;
                let mut bid_3_price: Option<u32> = None;
                let mut bid_4_price: Option<u32> = None;
                let mut bid_5_price: Option<u32> = None;
                let mut bid_6_price: Option<u32> = None;
                let mut bid_7_price: Option<u32> = None;
                let mut bid_8_price: Option<u32> = None;
                let mut bid_9_price: Option<u32> = None;
                let mut bid_10_price: Option<u32> = None;
                let mut ask_1_price: Option<u32> = None;
                let mut ask_2_price: Option<u32> = None;
                let mut ask_3_price: Option<u32> = None;
                let mut ask_4_price: Option<u32> = None;
                let mut ask_5_price: Option<u32> = None;
                let mut ask_6_price: Option<u32> = None;
                let mut ask_7_price: Option<u32> = None;
                let mut ask_8_price: Option<u32> = None;
                let mut ask_9_price: Option<u32> = None;
                let mut ask_10_price: Option<u32> = None;
                let mut bid_1_qty: Option<u32> = None;
                let mut bid_2_qty: Option<u32> = None;
                let mut bid_3_qty: Option<u32> = None;
                let mut bid_4_qty: Option<u32> = None;
                let mut bid_5_qty: Option<u32> = None;
                let mut bid_6_qty: Option<u32> = None;
                let mut bid_7_qty: Option<u32> = None;
                let mut bid_8_qty: Option<u32> = None;
                let mut bid_9_qty: Option<u32> = None;
                let mut bid_10_qty: Option<u32> = None;
                let mut ask_1_qty: Option<u32> = None;
                let mut ask_2_qty: Option<u32> = None;
                let mut ask_3_qty: Option<u32> = None;
                let mut ask_4_qty: Option<u32> = None;
                let mut ask_5_qty: Option<u32> = None;
                let mut ask_6_qty: Option<u32> = None;
                let mut ask_7_qty: Option<u32> = None;
                let mut ask_8_qty: Option<u32> = None;
                let mut ask_9_qty: Option<u32> = None;
                let mut ask_10_qty: Option<u32> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "exch_epoch" => {
                            if exch_epoch.is_some() {
                                return Err(Error::duplicate_field("exch_epoch"));
                            }
                            exch_epoch = Some(map.next_value()?);
                        }
                        "bid_1_price" => {
                            if bid_1_price.is_some() {
                                return Err(Error::duplicate_field("bid_1_price"));
                            }
                            bid_1_price = Some(map.next_value()?);
                        }
                        "bid_2_price" => {
                            if bid_2_price.is_some() {
                                return Err(Error::duplicate_field("bid_2_price"));
                            }
                            bid_2_price = Some(map.next_value()?);
                        }
                        "bid_3_price" => {
                            if bid_3_price.is_some() {
                                return Err(Error::duplicate_field("bid_3_price"));
                            }
                            bid_3_price = Some(map.next_value()?);
                        }
                        "bid_4_price" => {
                            if bid_4_price.is_some() {
                                return Err(Error::duplicate_field("bid_4_price"));
                            }
                            bid_4_price = Some(map.next_value()?);
                        }
                        "bid_5_price" => {
                            if bid_5_price.is_some() {
                                return Err(Error::duplicate_field("bid_5_price"));
                            }
                            bid_5_price = Some(map.next_value()?);
                        }
                        "bid_6_price" => {
                            if bid_6_price.is_some() {
                                return Err(Error::duplicate_field("bid_6_price"));
                            }
                            bid_6_price = Some(map.next_value()?);
                        }
                        "bid_7_price" => {
                            if bid_7_price.is_some() {
                                return Err(Error::duplicate_field("bid_7_price"));
                            }
                            bid_7_price = Some(map.next_value()?);
                        }
                        "bid_8_price" => {
                            if bid_8_price.is_some() {
                                return Err(Error::duplicate_field("bid_8_price"));
                            }
                            bid_8_price = Some(map.next_value()?);
                        }
                        "bid_9_price" => {
                            if bid_9_price.is_some() {
                                return Err(Error::duplicate_field("bid_9_price"));
                            }
                            bid_9_price = Some(map.next_value()?);
                        }
                        "bid_10_price" => {
                            if bid_10_price.is_some() {
                                return Err(Error::duplicate_field("bid_10_price"));
                            }
                            bid_10_price = Some(map.next_value()?);
                        }
                        "ask_1_price" => {
                            if ask_1_price.is_some() {
                                return Err(Error::duplicate_field("ask_1_price"));
                            }
                            ask_1_price = Some(map.next_value()?);
                        }
                        "ask_2_price" => {
                            if ask_2_price.is_some() {
                                return Err(Error::duplicate_field("ask_2_price"));
                            }
                            ask_2_price = Some(map.next_value()?);
                        }
                        "ask_3_price" => {
                            if ask_3_price.is_some() {
                                return Err(Error::duplicate_field("ask_3_price"));
                            }
                            ask_3_price = Some(map.next_value()?);
                        }
                        "ask_4_price" => {
                            if ask_4_price.is_some() {
                                return Err(Error::duplicate_field("ask_4_price"));
                            }
                            ask_4_price = Some(map.next_value()?);
                        }
                        "ask_5_price" => {
                            if ask_5_price.is_some() {
                                return Err(Error::duplicate_field("ask_5_price"));
                            }
                            ask_5_price = Some(map.next_value()?);
                        }
                        "ask_6_price" => {
                            if ask_6_price.is_some() {
                                return Err(Error::duplicate_field("ask_6_price"));
                            }
                            ask_6_price = Some(map.next_value()?);
                        }
                        "ask_7_price" => {
                            if ask_7_price.is_some() {
                                return Err(Error::duplicate_field("ask_7_price"));
                            }
                            ask_7_price = Some(map.next_value()?);
                        }
                        "ask_8_price" => {
                            if ask_8_price.is_some() {
                                return Err(Error::duplicate_field("ask_8_price"));
                            }
                            ask_8_price = Some(map.next_value()?);
                        }
                        "ask_9_price" => {
                            if ask_9_price.is_some() {
                                return Err(Error::duplicate_field("ask_9_price"));
                            }
                            ask_9_price = Some(map.next_value()?);
                        }
                        "ask_10_price" => {
                            if ask_10_price.is_some() {
                                return Err(Error::duplicate_field("ask_10_price"));
                            }
                            ask_10_price = Some(map.next_value()?);
                        }
                        "bid_1_qty" => {
                            if bid_1_qty.is_some() {
                                return Err(Error::duplicate_field("bid_1_qty"));
                            }
                            bid_1_qty = Some(map.next_value()?);
                        }
                        "bid_2_qty" => {
                            if bid_2_qty.is_some() {
                                return Err(Error::duplicate_field("bid_2_qty"));
                            }
                            bid_2_qty = Some(map.next_value()?);
                        }
                        "bid_3_qty" => {
                            if bid_3_qty.is_some() {
                                return Err(Error::duplicate_field("bid_3_qty"));
                            }
                            bid_3_qty = Some(map.next_value()?);
                        }
                        "bid_4_qty" => {
                            if bid_4_qty.is_some() {
                                return Err(Error::duplicate_field("bid_4_qty"));
                            }
                            bid_4_qty = Some(map.next_value()?);
                        }
                        "bid_5_qty" => {
                            if bid_5_qty.is_some() {
                                return Err(Error::duplicate_field("bid_5_qty"));
                            }
                            bid_5_qty = Some(map.next_value()?);
                        }
                        "bid_6_qty" => {
                            if bid_6_qty.is_some() {
                                return Err(Error::duplicate_field("bid_6_qty"));
                            }
                            bid_6_qty = Some(map.next_value()?);
                        }
                        "bid_7_qty" => {
                            if bid_7_qty.is_some() {
                                return Err(Error::duplicate_field("bid_7_qty"));
                            }
                            bid_7_qty = Some(map.next_value()?);
                        }
                        "bid_8_qty" => {
                            if bid_8_qty.is_some() {
                                return Err(Error::duplicate_field("bid_8_qty"));
                            }
                            bid_8_qty = Some(map.next_value()?);
                        }
                        "bid_9_qty" => {
                            if bid_9_qty.is_some() {
                                return Err(Error::duplicate_field("bid_9_qty"));
                            }
                            bid_9_qty = Some(map.next_value()?);
                        }
                        "bid_10_qty" => {
                            if bid_10_qty.is_some() {
                                return Err(Error::duplicate_field("bid_10_qty"));
                            }
                            bid_10_qty = Some(map.next_value()?);
                        }
                        "ask_1_qty" => {
                            if ask_1_qty.is_some() {
                                return Err(Error::duplicate_field("ask_1_qty"));
                            }
                            ask_1_qty = Some(map.next_value()?);
                        }
                        "ask_2_qty" => {
                            if ask_2_qty.is_some() {
                                return Err(Error::duplicate_field("ask_2_qty"));
                            }
                            ask_2_qty = Some(map.next_value()?);
                        }
                        "ask_3_qty" => {
                            if ask_3_qty.is_some() {
                                return Err(Error::duplicate_field("ask_3_qty"));
                            }
                            ask_3_qty = Some(map.next_value()?);
                        }
                        "ask_4_qty" => {
                            if ask_4_qty.is_some() {
                                return Err(Error::duplicate_field("ask_4_qty"));
                            }
                            ask_4_qty = Some(map.next_value()?);
                        }
                        "ask_5_qty" => {
                            if ask_5_qty.is_some() {
                                return Err(Error::duplicate_field("ask_5_qty"));
                            }
                            ask_5_qty = Some(map.next_value()?);
                        }
                        "ask_6_qty" => {
                            if ask_6_qty.is_some() {
                                return Err(Error::duplicate_field("ask_6_qty"));
                            }
                            ask_6_qty = Some(map.next_value()?);
                        }
                        "ask_7_qty" => {
                            if ask_7_qty.is_some() {
                                return Err(Error::duplicate_field("ask_7_qty"));
                            }
                            ask_7_qty = Some(map.next_value()?);
                        }
                        "ask_8_qty" => {
                            if ask_8_qty.is_some() {
                                return Err(Error::duplicate_field("ask_8_qty"));
                            }
                            ask_8_qty = Some(map.next_value()?);
                        }
                        "ask_9_qty" => {
                            if ask_9_qty.is_some() {
                                return Err(Error::duplicate_field("ask_9_qty"));
                            }
                            ask_9_qty = Some(map.next_value()?);
                        }
                        "ask_10_qty" => {
                            if ask_10_qty.is_some() {
                                return Err(Error::duplicate_field("ask_10_qty"));
                            }
                            ask_10_qty = Some(map.next_value()?);
                        }
                        _ => {
                            // Ignore unknown fields
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                let mut vec: Vec<LimitOrder> = Vec::with_capacity(32);
                let side = Side::Bid;
                let price = bid_1_price.ok_or_else(|| Error::missing_field("bid_1_price"))?;
                let qty = bid_1_qty.ok_or_else(|| Error::missing_field("bid_1_qty"))?;
                vec.push(LimitOrder { side, price, qty });
                let price = bid_2_price.ok_or_else(|| Error::missing_field("bid_2_price"))?;
                let qty = bid_2_qty.ok_or_else(|| Error::missing_field("bid_2_qty"))?;
                vec.push(LimitOrder { side, price, qty });
                let price = bid_3_price.ok_or_else(|| Error::missing_field("bid_3_price"))?;
                let qty = bid_3_qty.ok_or_else(|| Error::missing_field("bid_3_qty"))?;
                vec.push(LimitOrder { side, price, qty });
                let price = bid_4_price.ok_or_else(|| Error::missing_field("bid_4_price"))?;
                let qty = bid_4_qty.ok_or_else(|| Error::missing_field("bid_4_qty"))?;
                vec.push(LimitOrder { side, price, qty });
                let price = bid_5_price.ok_or_else(|| Error::missing_field("bid_5_price"))?;
                let qty = bid_5_qty.ok_or_else(|| Error::missing_field("bid_5_qty"))?;
                vec.push(LimitOrder { side, price, qty });
                let price = bid_6_price.ok_or_else(|| Error::missing_field("bid_6_price"))?;
                let qty = bid_6_qty.ok_or_else(|| Error::missing_field("bid_6_qty"))?;
                vec.push(LimitOrder { side, price, qty });
                let price = bid_7_price.ok_or_else(|| Error::missing_field("bid_7_price"))?;
                let qty = bid_7_qty.ok_or_else(|| Error::missing_field("bid_7_qty"))?;
                vec.push(LimitOrder { side, price, qty });
                let price = bid_8_price.ok_or_else(|| Error::missing_field("bid_8_price"))?;
                let qty = bid_8_qty.ok_or_else(|| Error::missing_field("bid_8_qty"))?;
                vec.push(LimitOrder { side, price, qty });
                let price = bid_9_price.ok_or_else(|| Error::missing_field("bid_9_price"))?;
                let qty = bid_9_qty.ok_or_else(|| Error::missing_field("bid_9_qty"))?;
                vec.push(LimitOrder { side, price, qty });
                let price = bid_10_price.ok_or_else(|| Error::missing_field("bid_10_price"))?;
                let qty = bid_10_qty.ok_or_else(|| Error::missing_field("bid_10_qty"))?;
                vec.push(LimitOrder { side, price, qty });
                let side = Side::Ask;
                let price = ask_1_price.ok_or_else(|| Error::missing_field("ask_1_price"))?;
                let qty = ask_1_qty.ok_or_else(|| Error::missing_field("ask_1_qty"))?;
                vec.push(LimitOrder { side, price, qty });
                let price = ask_2_price.ok_or_else(|| Error::missing_field("ask_2_price"))?;
                let qty = ask_2_qty.ok_or_else(|| Error::missing_field("ask_2_qty"))?;
                vec.push(LimitOrder { side, price, qty });
                let price = ask_3_price.ok_or_else(|| Error::missing_field("ask_3_price"))?;
                let qty = ask_3_qty.ok_or_else(|| Error::missing_field("ask_3_qty"))?;
                vec.push(LimitOrder { side, price, qty });
                let price = ask_4_price.ok_or_else(|| Error::missing_field("ask_4_price"))?;
                let qty = ask_4_qty.ok_or_else(|| Error::missing_field("ask_4_qty"))?;
                vec.push(LimitOrder { side, price, qty });
                let price = ask_5_price.ok_or_else(|| Error::missing_field("ask_5_price"))?;
                let qty = ask_5_qty.ok_or_else(|| Error::missing_field("ask_5_qty"))?;
                vec.push(LimitOrder { side, price, qty });
                let price = ask_6_price.ok_or_else(|| Error::missing_field("ask_6_price"))?;
                let qty = ask_6_qty.ok_or_else(|| Error::missing_field("ask_6_qty"))?;
                vec.push(LimitOrder { side, price, qty });
                let price = ask_7_price.ok_or_else(|| Error::missing_field("ask_7_price"))?;
                let qty = ask_7_qty.ok_or_else(|| Error::missing_field("ask_7_qty"))?;
                vec.push(LimitOrder { side, price, qty });
                let price = ask_8_price.ok_or_else(|| Error::missing_field("ask_8_price"))?;
                let qty = ask_8_qty.ok_or_else(|| Error::missing_field("ask_8_qty"))?;
                vec.push(LimitOrder { side, price, qty });
                let price = ask_9_price.ok_or_else(|| Error::missing_field("ask_9_price"))?;
                let qty = ask_9_qty.ok_or_else(|| Error::missing_field("ask_9_qty"))?;
                vec.push(LimitOrder { side, price, qty });
                let price = ask_10_price.ok_or_else(|| Error::missing_field("ask_10_price"))?;
                let qty = ask_10_qty.ok_or_else(|| Error::missing_field("ask_10_qty"))?;
                vec.push(LimitOrder { side, price, qty });

                Ok(Snap {
                    exch_epoch: exch_epoch.unwrap(),
                    vec,
                })
            }
        }

        deserializer.deserialize_map(EventVisitor)
    }
}

impl<'de> Deserialize<'de> for Order {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EventVisitor;

        impl<'de> Visitor<'de> for EventVisitor {
            type Value = Order;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Event")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut exch_epoch: Option<u64> = None;
                let mut side: Option<u8> = None;
                let mut price: Option<u32> = None;
                let mut qty: Option<u32> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "exch_epoch" => {
                            if exch_epoch.is_some() {
                                return Err(Error::duplicate_field("exch_epoch"));
                            }
                            exch_epoch = Some(map.next_value()?);
                        }
                        "side" => {
                            if side.is_some() {
                                return Err(Error::duplicate_field("side"));
                            }
                            side = Some(map.next_value()?);
                        }
                        "price" => {
                            if price.is_some() {
                                return Err(Error::duplicate_field("price"));
                            }
                            price = Some(map.next_value()?);
                        }
                        "qty" => {
                            if qty.is_some() {
                                return Err(Error::duplicate_field("qty"));
                            }
                            qty = Some(map.next_value()?);
                        }
                        _ => {
                            // Ignore unknown fields
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }
                let side = match side.unwrap() {
                    0 => Side::Ask,
                    1 => Side::Bid,
                    _ => unreachable!(),
                };

                Ok(Order {
                    // HACK
                    id: exch_epoch.unwrap(),
                    side,
                    price: price.unwrap(),
                    qty: qty.unwrap(),
                })
            }
        }

        deserializer.deserialize_map(EventVisitor)
    }
}
