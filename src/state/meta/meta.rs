use crate::util::attribute::{AttributeContainer, Container};

#[typetag::serde]
pub trait Meta : Container {}

#[typetag::serde]
impl Meta for AttributeContainer {}


pub fn new_meta() -> Box<dyn Meta> {
    Box::new(AttributeContainer::new_with_class(String::from("Meta")))
}
