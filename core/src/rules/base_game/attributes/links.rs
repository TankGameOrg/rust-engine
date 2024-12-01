use std::collections::{HashMap, HashSet};

use as_any::Downcast;

use crate::rules::infrastructure::ecs::{Attribute, AttributeStore, Handle, HandleIterator, Query};

/// A link to another entity
///
/// Links can be fed to find() to find all backlinks i.e. If Tank1 has a Link(Player1) the
/// `find(Link(Player1)) = [Tank1]`
pub trait Link: Attribute + std::hash::Hash + PartialEq + Eq + Clone {
    /// The handle to the other entity
    fn get_handle(&self) -> Handle;
}

#[macro_export]
macro_rules! generic_link {
    ($name:ident, $doc:tt) => {
        #[derive(Debug, Hash, Eq, PartialEq, Clone)]
        #[doc=$doc]
        pub struct $name(Handle);

        impl $name {
            pub fn new(handle: Handle) -> Self {
                Self(handle)
            }
        }

        impl Link for $name {
            fn get_handle(&self) -> Handle {
                self.0
            }
        }
    };
}

generic_link!(
    Owner,
    "A link to the player (entity) that controls this entity"
);
generic_link!(
    TeamMember,
    "A link to the team that the current player is a part of"
);

/// A store to track a specific kind of link between entities
#[derive(Debug)]
struct HandleReverseLookup<T: Link> {
    handle_to_link: HashMap<Handle, T>,
    link_to_handle: HashMap<T, HashSet<Handle>>,
}

impl<T: Link> Default for HandleReverseLookup<T> {
    fn default() -> Self {
        Self {
            handle_to_link: HashMap::new(),
            link_to_handle: HashMap::new(),
        }
    }
}

impl<T: Link> AttributeStore for HandleReverseLookup<T> {
    fn len(&self) -> usize {
        self.handle_to_link.len()
    }

    fn iter_handles(&self) -> HandleIterator {
        HandleIterator::new(self.handle_to_link.keys().cloned())
    }

    fn get_attribute(&self, handle: Handle) -> &dyn Attribute {
        self.handle_to_link.get(&handle).unwrap()
    }

    fn set_attribute(
        &mut self,
        handle: Handle,
        value: crate::rules::infrastructure::ecs::BoxedAttribute,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let value: T = value.downcast()?;

        self.handle_to_link.insert(handle.clone(), value.clone());

        self.link_to_handle
            .entry(value.clone())
            .or_insert_with(|| HashSet::default());
        self.link_to_handle.get_mut(&value).unwrap().insert(handle);
        Ok(())
    }

    fn remove_attribute(&mut self, handle: Handle) {
        let value = self.handle_to_link.remove(&handle);
        if let Some(value) = value {
            let handle_set = self.link_to_handle.get_mut(&value).unwrap();
            handle_set.remove(&handle);

            if handle_set.is_empty() {
                self.link_to_handle.remove(&value);
            }
        }
    }
}

impl<T: Link> Attribute for T {
    fn create_store(
        &self,
        _properties: &crate::rules::infrastructure::ecs::Properties,
    ) -> Box<dyn AttributeStore>
    where
        Self: Sized,
    {
        Box::new(HandleReverseLookup::<T>::default())
    }
}

impl<T: Link> Query for T {
    type StoredAttribute = T;

    fn query<'iter>(&'iter self, store: &'iter dyn AttributeStore) -> HandleIterator<'iter> {
        let store: &HandleReverseLookup<T> = store.downcast_ref().unwrap();

        if let Some(handle_set) = store.link_to_handle.get(self) {
            HandleIterator::new(handle_set.iter().cloned())
        } else {
            HandleIterator::empty()
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use crate::rules::infrastructure::ecs::Universe;

    use super::*;

    fn find_handle_set(universe: &Universe, team_handle: Handle) -> HashSet<Handle> {
        universe
            .find(&TeamMember(team_handle))
            .unwrap()
            .map(|entity| entity.get_handle())
            .collect()
    }

    #[test]
    fn teams_can_be_added() {
        let mut universe = Universe::default();
        let team_handle = universe.add_entity().as_handle().unwrap();

        let team_member1 = universe
            .add_entity()
            .set(TeamMember::new(team_handle))
            .as_handle()
            .unwrap();

        let team_member2 = universe
            .add_entity()
            .set(TeamMember::new(team_handle))
            .as_handle()
            .unwrap();

        let other_team_handle = universe.add_entity().as_handle().unwrap();

        let other_team_member = universe
            .add_entity()
            .set(TeamMember::new(other_team_handle))
            .as_handle()
            .unwrap();

        let team = find_handle_set(&universe, team_handle);
        assert!(team.contains(&team_member1));
        assert!(team.contains(&team_member2));

        let other_team = find_handle_set(&universe, other_team_handle);
        assert!(other_team.contains(&other_team_member));

        universe
            .get_entity_mut(other_team_member)
            .unwrap()
            .remove::<TeamMember>();

        let empty_team = find_handle_set(&universe, other_team_handle);
        assert_eq!(empty_team.len(), 0);
    }
}
