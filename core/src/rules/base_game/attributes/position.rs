use std::{collections::HashMap, error::Error};

use crate::{basic_error, rules::infrastructure::ecs::{Attribute, AttributeStore, Handle, HandleIterator, Query, QueryOne}};

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, PartialOrd, Ord)]
pub enum Level {
    Unit,
    Floor,
}

const NUM_LEVELS: usize = 2;

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, PartialOrd, Ord)]
pub struct Position {
    x: usize,
    y: usize,
    level: Level,
}

impl Position {
    pub fn new(level: Level, x: usize, y: usize) -> Position {
        Position {
            level,
            x,
            y,
        }
    }
}

impl Attribute for Position {}

impl QueryOne for Position {
    type StoredAttribute = Position;
    type Store = Board;

    fn query_one(&self, store: &Self::Store) -> Option<Handle> {
        match store.get_index(self) {
            Ok(index) => store.board[index],
            Err(_) => None,
        }
    }
}

#[derive(Debug)]
pub struct Board {
    width: usize,
    height: usize,
    board: Vec<Option<Handle>>,
    reverse_lookups: HashMap<Handle, Position>,
}

impl Board {
    #[inline]
    pub fn new(width: usize, height: usize) -> Board {
        Board {
            width,
            height,
            board: vec![None; NUM_LEVELS * width * height],
            reverse_lookups: HashMap::new(),
        }
    }

    fn get_index(&self, position: &Position) -> Result<usize, Box<dyn Error>> {
        if position.x >= self.width || position.y >= self.height {
            return Err(basic_error!("Position {:?} is outside the valid bounds ({}, {})", position, self.width, self.height))
        }

        let floor_index = match position.level {
            Level::Unit => 0,
            Level::Floor => 1,
        };

        Ok((floor_index * self.width * self.height) +
            (position.y * self.width) +
            position.x)
    }
}

impl AttributeStore for Board {
    type StoredAttribute = Position;

    fn len(&self) -> usize {
        self.reverse_lookups.len()
    }

    fn iter_handles(&self) -> HandleIterator {
        HandleIterator::new(self.reverse_lookups.iter()
            .map(|(handle, _)| *handle))
    }

    fn get_attribute(&self, handle: Handle) -> &Self::StoredAttribute {
        self.reverse_lookups.get(&handle).unwrap()
    }

    fn set_attribute(
            &mut self,
            handle: Handle,
            position: Self::StoredAttribute,
        ) -> Result<(), Box<dyn std::error::Error>> {
        let index = self.get_index(&position)?;

        if let Some(occupying_handle) = self.board[index] {
            return Err(basic_error!("The position {:?} is already occupied by {:?}", position, occupying_handle));
        }

        self.remove_attribute(handle);

        self.board[index] = Some(handle);
        self.reverse_lookups.insert(handle, position);
        Ok(())
    }

    fn remove_attribute(&mut self, handle: Handle) {
        if let Some(current_position) = self.reverse_lookups.remove(&handle) {
            let index = self.get_index(&current_position).unwrap();
            self.board[index] = None;
        }
    }
}

pub struct AreaQuery {
    center: Position,
    radius: usize,
}

impl AreaQuery {
    pub fn new(center: Position, radius: usize) -> AreaQuery {
        AreaQuery {
            center,
            radius,
        }
    }

    pub fn adjacent_to(center: Position) -> AreaQuery {
        AreaQuery::new(center, 1)
    }

    fn iter<'iter>(&'iter self, store: &'iter Board) -> AreaIterator<'iter> {
        let start_x = if self.center.x < self.radius { 0 } else { self.center.x - self.radius };
        let start_y = if self.center.y < self.radius { 0 } else { self.center.y - self.radius };
        let start_position = Position::new(self.center.level, start_x, start_y);
        let end_position = Position::new(self.center.level, self.center.x + self.radius, self.center.y + self.radius);

        AreaIterator::new(store, start_position, end_position)
    }
}

struct AreaIterator<'store> {
    current_position: Position,
    start_x: usize,
    end_position: Position,
    store: &'store Board
}

impl<'store> AreaIterator<'store> {
    fn new(store: &Board, start_position: Position, end_position: Position) -> AreaIterator {
        assert!(start_position.level == end_position.level);

        AreaIterator {
            start_x: start_position.x,
            current_position: start_position,
            end_position,
            store,
        }
    }

    fn advance_position(&mut self) {
        if self.current_position.x < self.end_position.x {
            self.current_position.x += 1;
        }
        else {
            self.current_position.x = self.start_x;
            self.current_position.y += 1;
        }
    }
}

impl<'store> Iterator for AreaIterator<'store> {
    type Item = Handle;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // We've already gone through the entire area
            if self.current_position > self.end_position {
                return None;
            }
            
            if let Some(handle) = self.current_position.query_one(self.store) {
                self.advance_position();
                return Some(handle);
            }

            self.advance_position();
        }
    }
}

impl Query for AreaQuery {
    type StoredAttribute = Position;
    type Store = Board;

    fn query<'iter>(&'iter self, store: &'iter Self::Store) -> HandleIterator<'iter> {
        HandleIterator::new(self.iter(store))
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use crate::{rules::infrastructure::ecs::{EntityRef, Universe}, signature};

    use super::*;

    #[test]
    fn get_and_set_attributes() {
        let mut universe = Universe::default();
        universe.set_attribute_store(Board::new(3, 4));
        let position1 = Position::new(Level::Unit, 1, 3);
        let position2 = Position::new(Level::Floor, 1, 3);
        let position3 = Position::new(Level::Unit, 2, 2);

        universe.add_entity().unwrap();
        universe.add_entity().set(position2).unwrap();
        universe.add_entity().set(position3).unwrap();

        let handle = universe.add_entity()
            .set(position1)
            .as_handle()
            .unwrap();

        let gathered: Vec<EntityRef> = universe.gather(signature!(Position)).collect();
        assert_eq!(gathered.len(), 3);

        universe.get_entity_mut(handle).unwrap().remove::<Position>();

        let gathered: Vec<EntityRef> = universe.gather(signature!(Position)).collect();
        assert_eq!(gathered.len(), 2);
    }

    #[test]
    fn our_of_bounds_and_overlap() {
        let mut universe = Universe::default();
        universe.set_attribute_store(Board::new(5, 3));

        // Out of bounds
        let result = universe.add_entity()
            .set(Position::new(Level::Unit, 1, 4))
            .as_result();

        assert!(result.is_err());

        let result = universe.add_entity()
            .set(Position::new(Level::Unit, 5, 0))
            .as_result();

        assert!(result.is_err());

        // Attempt to place entity in an occupied space
        let origin_handle = universe.add_entity()
            .set(Position::new(Level::Unit, 0, 0))
            .as_handle()
            .unwrap();

        let result = universe.add_entity()
            .set(Position::new(Level::Unit, 0, 0))
            .as_result();

        assert!(result.is_err());

        universe.remove_entity(origin_handle).unwrap();

        // Place an entity in a formerly occupied space
        let mut entity = universe.add_entity()
            .set(Position::new(Level::Unit, 0, 0))
            .as_entity_mut()
            .unwrap();

        entity.set(Position::new(Level::Floor, 0, 0)).unwrap();

        // Place an entity after moving an entity out of a space
        universe.add_entity()
            .set(Position::new(Level::Unit, 0, 0))
            .unwrap();
    }

    #[test]
    fn find_by_position() {
        let mut universe = Universe::default();
        universe.set_attribute_store(Board::new(3, 3));

        let position = Position::new(Level::Unit, 0, 0);
        let expected_handle = universe.add_entity()
            .set(position.clone())
            .as_handle()
            .unwrap();

        let found_entity = universe.query_one(position.clone()).unwrap();
        assert_eq!(found_entity.get_handle(), expected_handle);

        assert!(universe.query_one(Position::new(Level::Floor, 2, 2)).is_none());
    }

    fn verify_query(universe: &Universe, query: impl Query, expected: Vec<Handle>) {
        let found: HashSet<Handle> = universe.query(&query)
            .unwrap()
            .map(|entity| entity.get_handle())
            .collect();

        let expected_set: HashSet<Handle> = expected.into_iter().collect();
        assert_eq!(found, expected_set);
    }

    #[test]
    fn find_by_area() {
        let mut universe = Universe::default();
        universe.set_attribute_store(Board::new(5, 5));

        let handle_0_1 = universe.add_entity()
            .set(Position::new(Level::Unit, 0, 1))
            .as_handle()
            .unwrap();

        let handle_2_2 = universe.add_entity()
            .set(Position::new(Level::Unit, 2, 2))
            .as_handle()
            .unwrap();

        let handle_0_0_floor = universe.add_entity()
            .set(Position::new(Level::Floor, 0, 0))
            .as_handle()
            .unwrap();

        verify_query(&universe, 
            AreaQuery::adjacent_to(Position::new(Level::Unit, 0, 0)),
            vec![handle_0_1]);

        verify_query(&universe, 
            AreaQuery::adjacent_to(Position::new(Level::Unit, 1, 1)),
            vec![handle_0_1, handle_2_2]);

        verify_query(&universe, 
            AreaQuery::new(Position::new(Level::Floor, 4, 2), 4),
            vec![handle_0_0_floor]);
    }
}