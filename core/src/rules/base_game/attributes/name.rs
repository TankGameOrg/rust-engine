use crate::rules::infrastructure::ecs::Attribute;

/// The name of an entity generally this is just show to use user
#[derive(Debug)]
pub struct Name(String);

impl Attribute for Name {}

impl Name {
    pub fn new(name: String) -> Self {
        Self(name)
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
