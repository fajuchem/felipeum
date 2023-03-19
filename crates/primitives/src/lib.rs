// use bits::B160;

// pub mod bits;
pub mod signature;
pub mod transaction;

pub type TxHash = String;

// TODO: look into why this is not working
// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct Address(B160);
//
// impl Address {
//     pub fn new(value: B160) -> Self {
//         Address(B160::from(value))
//     }
// }
