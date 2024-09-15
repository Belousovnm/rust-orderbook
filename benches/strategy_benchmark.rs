// #![feature(test)]
// extern crate test;
// use orderbook::Snap;
// use std::hint::black_box;
//
// fn deserialize() -> Vec<Result<Snap, csv::Error>> {
//     let ob_path = "data/ob.csv";
//     let snap_reader = csv::Reader::from_path(ob_path).unwrap();
//     black_box(
//         black_box(snap_reader)
//             .deserialize::<Snap>()
//             .collect::<Vec<_>>(),
//     )
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use test::Bencher;
//
//     #[bench]
//     fn deserialize_bench(b: &mut Bencher) {
//         b.iter(deserialize);
//     }
// }
