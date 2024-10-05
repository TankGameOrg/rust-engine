use crate::util::attribute::JsonType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Upgrade {
    Range(i32),
    Speed(i32),
    Attack(i32),
    Defence(i32),
}

#[typetag::serde]
impl JsonType for Upgrade {}

#[typetag::serde]
impl JsonType for Vec<Upgrade> {}
