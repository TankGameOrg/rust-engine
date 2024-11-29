use std::{cmp::min, collections::HashMap, error::Error};

use crate::{basic_error, rules::infrastructure::ecs::{Attribute, AttributeStore, BoxedAttribute, Handle, HandleIterator, Properties, Property, Query, QueryOne}};

/// The part of the floor space that this entity occupies i.e. Floor
#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, PartialOrd, Ord)]
pub enum Level {
    Floor,
    Unit,
}

const NUM_LEVELS: usize = 2;

/// The position of an entity in 3d space
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

impl Position {
    #[inline]
    pub fn get_level(&self) -> Level {
        self.level
    }

    #[inline]
    pub fn get_x(&self) -> usize {
        self.x
    }

    #[inline]
    pub fn get_y(&self) -> usize {
        self.y
    }
}

impl Attribute for Position {
    fn create_store(&self, properties: &Properties) -> Box<dyn AttributeStore> where Self: Sized {
        assert!(properties.has::<Bounds>(), "The universe must have the Bounds property to use the Position attribute");
        Box::new(Board::new(properties.get::<Bounds>().unwrap().clone()))
    }
}

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

/// The dimensions of the game board
/// 
/// Bounds vs Board: The Bounds know the width and height and can find all of the [`Position`]s in or around an area.  But it
/// doesn't know where the actual [`Entities`] are located.  The board can tell you what [`Entities`] are in an area but it can't tell
/// you about unoccupied spaces in that area.
/// 
/// [`Entities`]: crate::rules::infrastructure::ecs::EntityRef
#[derive(Debug, Clone)]
pub struct Bounds {
    width: usize,
    height: usize,
}

impl Property for Bounds {}

impl Bounds {
    #[inline]
    pub fn new(width: usize, height: usize) -> Bounds {
        Bounds {
            width,
            height,
        }
    }

    #[inline]
    pub fn get_width(&self) -> usize {
        self.width
    }

    #[inline]
    pub fn get_height(&self) -> usize {
        self.height
    }

    /// Check if a position is in bounds
    #[inline]
    pub fn is_valid(&self, position: &Position) -> bool {
        position.x >= self.width || position.y >= self.height
    }

    /// Return an error if a position is out of bounds
    #[inline]
    pub fn verify_position(&self, position: &Position) -> Result<(), Box<dyn Error>> {
        if self.is_valid(position) {
            return Err(basic_error!("Position {:?} is outside the valid bounds ({}, {})", position, self.width, self.height))
        }

        Ok(())
    }
}

/// A rectangular area that spans one or more [`Level`]s that can be used to find [`Position`]s or Entities in that area
#[derive(Debug, Default)]
pub struct RectangleQuery {
    levels: Vec<Level>,
    top_left_x: usize,
    top_left_y: usize,
    bottom_right_x: usize,
    bottom_right_y: usize,
}

impl RectangleQuery {
    /// Set the top left corner of the query
    pub fn top_left(mut self, x: usize, y: usize) -> Self {
        self.top_left_x = x;
        self.top_left_y = y;
        self
    }

    /// Set the bottom right corner of the query
    pub fn bottom_right(mut self, x: usize, y: usize) -> Self {
        self.bottom_right_x = x;
        self.bottom_right_y = y;
        self
    }

    /// Add a level to search
    pub fn level(mut self, level: Level) -> Self {
        self.levels.push(level);
        self
    }

    /// Create a square where the top of the rectangle is radius away from the center (the same applies to all sides)
    pub fn centered_at(center: Position, radius: usize) -> Self {
        let top_left_x = if center.x < radius { 0 } else { center.x - radius };
        let top_left_y = if center.y < radius { 0 } else { center.y - radius };

        Self::default()
            .top_left(top_left_x, top_left_y)
            .bottom_right(center.x + radius, center.y + radius)
            .level(center.level)
    }

    /// All of the spaces adjacent to and including center
    pub fn adjacent_to(center: Position) -> Self {
        Self::centered_at(center, 1)
    }
}

enum IteratorState {
    New,
    Ended,
    Active(Position),
}

/// An iterator that can iterate all of the in bounds spaces in a [`RectangleQuery`]
pub struct RectanglePositionIterator<'iter> {
    bounds: &'iter Bounds,
    query: &'iter RectangleQuery,
    state: IteratorState,
}

impl<'iter> RectanglePositionIterator<'iter> {
    fn new(bounds: &'iter Bounds, query: &'iter RectangleQuery) -> Self {
        Self {
            bounds,
            query,
            state: IteratorState::New,
        }
    }

    #[inline]
    fn get_bottom_right_x(&self) -> usize {
        min(self.query.bottom_right_x, self.bounds.get_width() - 1)
    }

    #[inline]
    fn get_bottom_right_y(&self) -> usize {
        min(self.query.bottom_right_y, self.bounds.get_height() - 1)
    }
}

impl<'iter> Iterator for RectanglePositionIterator<'iter> {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        let query = self.query;

        // Attempt to find the next position even if it isn't valid
        self.state = match self.state {
            IteratorState::Active(mut current_position) => {
                if current_position.x < self.get_bottom_right_x() {
                    current_position.x += 1;
                    IteratorState::Active(current_position)
                }
                else if current_position.y < self.get_bottom_right_y() {
                    current_position.x = query.top_left_x;
                    current_position.y += 1;
                    IteratorState::Active(current_position)
                }
                else {
                    let index = query.levels.iter()
                        .position(|level| *level == current_position.level)
                        .unwrap();

                    if index + 1 == query.levels.len() {
                        IteratorState::Ended
                    }
                    else {
                        current_position.x = query.top_left_x;
                        current_position.y = query.top_left_y;
                        current_position.level = query.levels[index + 1];
                        IteratorState::Active(current_position)
                    }
                }
            },
            IteratorState::New => {
                if query.levels.is_empty() {
                    IteratorState::Ended
                }
                else if query.top_left_x > self.get_bottom_right_x() || query.top_left_y > self.get_bottom_right_y() {
                    IteratorState::Ended
                }
                else {
                    IteratorState::Active(Position::new(query.levels[0], query.top_left_x, query.top_left_y))
                }
            },
            IteratorState::Ended => IteratorState::Ended,
        };

        match self.state {
            IteratorState::Active(current_position) => Some(current_position),
            _ => None,
        }
    }
}

/// An [`AttributeStore`] that manages the [`Position`]s of [`Entities`].
/// 
/// Clients should not use this directly if you want to find an entity call [`Universe::find`]
/// with a [`RectangleQuery`] or [`Universe::find_one`] with a [`Position`].
/// 
/// [`Entities`]: crate::rules::infrastructure::ecs::EntityRef
/// [`Universe::find`]: crate::rules::infrastructure::ecs::Universe::find
/// [`Universe::find_one`]: crate::rules::infrastructure::ecs::Universe::find_one
#[derive(Debug)]
pub struct Board {
    bounds: Bounds,
    board: Vec<Option<Handle>>,
    reverse_lookups: HashMap<Handle, Position>,
}

impl Board {
    #[inline]
    fn new(bounds: Bounds) -> Board {
        Board {
            board: vec![None; NUM_LEVELS * bounds.get_width() * bounds.get_height()],
            bounds,
            reverse_lookups: HashMap::new(),
        }
    }

    fn get_index(&self, position: &Position) -> Result<usize, Box<dyn Error>> {
        self.bounds.verify_position(position)?;

        let floor_index = match position.level {
            Level::Unit => 0,
            Level::Floor => 1,
        };

        Ok((floor_index * self.bounds.get_width() * self.bounds.get_height()) +
            (position.y * self.bounds.get_width()) +
            position.x)
    }

    fn get_from_position(&self, position: &Position) -> Option<Handle> {
        self.board[self.get_index(position).ok()?]
    }
}

impl AttributeStore for Board {
    fn len(&self) -> usize {
        self.reverse_lookups.len()
    }

    fn iter_handles(&self) -> HandleIterator {
        HandleIterator::new(self.reverse_lookups.iter()
            .map(|(handle, _)| *handle))
    }

    fn get_attribute(&self, handle: Handle) -> &dyn Attribute {
        self.reverse_lookups.get(&handle).unwrap()
    }

    fn set_attribute(
            &mut self,
            handle: Handle,
            position: BoxedAttribute,
        ) -> Result<(), Box<dyn std::error::Error>> {
        let position: Position = position.downcast()?;
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

impl Query for RectangleQuery {
    type StoredAttribute = Position;
    type Store = Board;

    fn query<'iter>(&'iter self, store: &'iter Self::Store) -> HandleIterator<'iter> {
        HandleIterator::new(
            RectanglePositionIterator::new(&store.bounds, self)
                .map(|position| store.get_from_position(&position))
                .flatten()
        )
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
        universe.get_properties_mut().set(Bounds::new(3, 4));
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

        let found: Vec<EntityRef> = universe.find_signature(signature!(Position)).collect();
        assert_eq!(found.len(), 3);

        universe.get_entity_mut(handle).unwrap().remove::<Position>();

        let found: Vec<EntityRef> = universe.find_signature(signature!(Position)).collect();
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn our_of_bounds_and_overlap() {
        let mut universe = Universe::default();
        universe.get_properties_mut().set(Bounds::new(5, 3));

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
        universe.get_properties_mut().set(Bounds::new(3, 3));

        let position = Position::new(Level::Unit, 0, 0);
        let expected_handle = universe.add_entity()
            .set(position.clone())
            .as_handle()
            .unwrap();

        let found_entity = universe.find_one(position.clone()).unwrap();
        assert_eq!(found_entity.get_handle(), expected_handle);

        assert!(universe.find_one(Position::new(Level::Floor, 2, 2)).is_none());
    }

    fn verify_query(universe: &Universe, query: impl Query, expected: Vec<Handle>) {
        let found: HashSet<Handle> = universe.find(&query)
            .unwrap()
            .map(|entity| entity.get_handle())
            .collect();

        let expected_set: HashSet<Handle> = expected.into_iter().collect();
        assert_eq!(found, expected_set);
    }

    #[test]
    fn find_by_area() {
        let mut universe = Universe::default();
        universe.get_properties_mut().set(Bounds::new(5, 5));

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
            RectangleQuery::adjacent_to(Position::new(Level::Unit, 0, 0)),
            vec![handle_0_1]);

        verify_query(&universe, 
            RectangleQuery::adjacent_to(Position::new(Level::Unit, 1, 1)),
            vec![handle_0_1, handle_2_2]);

        verify_query(&universe, 
            RectangleQuery::centered_at(Position::new(Level::Floor, 4, 2), 4),
            vec![handle_0_0_floor]);
    }
}