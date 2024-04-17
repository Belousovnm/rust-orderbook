// use orderbook_lib::Side;
use serde::de::Deserializer;
use serde::de::Error;
use serde::de::MapAccess;
use serde::de::Visitor;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, Serialize, Default)]
pub struct Event {
    pub bid_price: u64,
    pub ask_price: u64,
    pub bid_qty: u64,
    pub ask_qty: u64,
}

impl<'de> Deserialize<'de> for Event {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EventVisitor;

        impl<'de> Visitor<'de> for EventVisitor {
            type Value = Event;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Event")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut bid_price = None;
                let mut ask_price = None;
                let mut bid_qty = None;
                let mut ask_qty = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "bid_price" => {
                            if bid_price.is_some() {
                                return Err(Error::duplicate_field("bid_price"));
                            }
                            bid_price = Some(map.next_value()?);
                        }
                        "ask_price" => {
                            if ask_price.is_some() {
                                return Err(Error::duplicate_field("ask_price"));
                            }
                            ask_price = Some(map.next_value()?);
                        }
                        "bid_qty" => {
                            if bid_qty.is_some() {
                                return Err(Error::duplicate_field("bid_qty"));
                            }
                            bid_qty = Some(map.next_value()?);
                        }
                        "ask_qty" => {
                            if ask_qty.is_some() {
                                return Err(Error::duplicate_field("ask_qty"));
                            }
                            ask_qty = Some(map.next_value()?);
                        }
                        _ => {
                            // Ignore unknown fields
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let bid_price = bid_price.ok_or_else(|| Error::missing_field("bid_price"))?;
                let ask_price = ask_price.ok_or_else(|| Error::missing_field("ask_price"))?;
                let bid_qty = bid_qty.ok_or_else(|| Error::missing_field("bid_qty"))?;
                let ask_qty = ask_qty.ok_or_else(|| Error::missing_field("ask_qty"))?;

                Ok(Event {
                    bid_price,
                    ask_price,
                    bid_qty,
                    ask_qty,
                })
            }
        }

        deserializer.deserialize_map(EventVisitor)
    }
}
