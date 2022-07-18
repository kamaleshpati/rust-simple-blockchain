use std::time::SystemTime;
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::cmp::PartialEq;

#[derive(Serialize, Deserialize, Clone, Display, PartialEq, Debug)]
#[display(fmt = "from {} to {} amt {}", from, to, amount)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub time: SystemTime,
    pub amount: i32,
}
