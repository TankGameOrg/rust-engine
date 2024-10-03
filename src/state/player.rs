use crate::util::attribute::{AttributeContainer, Container};
use std::fmt::Debug;

trait Player: Container + Debug {}

impl Player for AttributeContainer {}
