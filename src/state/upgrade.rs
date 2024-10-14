use derive_defaults::derive_defaults;

#[derive_defaults()]
pub enum Upgrade {
    Range(i32),
    Speed(i32),
    Attack(i32),
    Defence(i32),
}
