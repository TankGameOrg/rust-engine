use crate::util::attribute::{Attribute, AttributeContainer, AttributeValue};
use std::collections::HashMap;
use uuid::Uuid;

pub type Reference = Uuid;
pub type Pool = HashMap<Reference, AttributeContainer>;

#[derive(Debug)]
pub struct State {
    pool: Pool,
    container: Reference,
}

impl State {
    pub fn new() -> State {
        let mut initial_map = HashMap::new();
        let container = AttributeContainer::new();
        let reference = Reference::new_v4();
        initial_map.insert(reference, container);

        State {
            pool: initial_map,
            container: reference,
        }
    }

    pub fn create_reference(&self) -> Reference {
        Reference::new_v4()
    }

    pub fn put(&mut self, attribute: AttributeContainer) -> Reference {
        let reference = Reference::new_v4();
        self.pool.insert(reference, attribute);
        reference.clone()
    }

    pub fn put_with_given_reference(
        &mut self,
        reference: Reference,
        container: AttributeContainer,
    ) -> Option<AttributeContainer> {
        self.pool.insert(reference, container)
    }

    pub fn remove(&mut self, reference: &Reference) -> Option<AttributeContainer> {
        self.pool.remove(reference)
    }

    pub fn get(&self, reference: &Reference) -> Option<&AttributeContainer> {
        self.pool.get(reference)
    }

    pub fn get_mut(&mut self, reference: &Reference) -> Option<&mut AttributeContainer> {
        self.pool.get_mut(reference)
    }

    pub fn get_container(&self) -> &AttributeContainer {
        self.get(&self.container).unwrap()
    }

    pub fn get_container_mut(&mut self) -> &mut AttributeContainer {
        let reference = self.container.clone();
        self.get_mut(&reference).expect("Pool does not have state")
    }

    pub fn gather<T: AttributeValue + Eq>(
        &self,
        attribute: &Attribute<T>,
        value: &T,
    ) -> Vec<(&Reference, &AttributeContainer)> {
        self.pool
            .iter()
            .filter(|(_, container)| {
                container
                    .get(attribute)
                    .map(|t| t == value)
                    .unwrap_or(false)
            })
            .collect()
    }

    pub fn gather_mut<T: AttributeValue + Eq>(
        &mut self,
        attribute: &Attribute<T>,
        value: &T,
    ) -> Vec<(&Reference, &mut AttributeContainer)> {
        self.pool
            .iter_mut()
            .filter(|(_, container)| {
                container
                    .get(attribute)
                    .map(|t| t == value)
                    .unwrap_or(false)
            })
            .collect()
    }
}
