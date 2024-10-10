use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Upgrade {
    Range(i32),
    Speed(i32),
    Attack(i32),
    Defence(i32),
}
