use crate::util::attribute::JsonType;

#[typetag::serde]
impl JsonType for bool {}

#[typetag::serde]
impl JsonType for i32 {}

#[typetag::serde]
impl JsonType for i64 {}

#[typetag::serde]
impl JsonType for String {}
