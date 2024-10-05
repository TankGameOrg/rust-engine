use crate::state::element::Element;
use crate::state::meta::player::PlayerRef;
use crate::state::position::Position;
use crate::util::attribute::JsonType;
use crate::util::attributes::{PLAYER_REF, POSITION};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum Tile<'a> {
    Unit(&'a Box<dyn Element>),
    Floor(&'a Box<dyn Element>),
}

// Grid[y][x]
pub type Grid = Vec<Vec<Option<Box<dyn Element>>>>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Board {
    width: usize,
    height: usize,
    units: Grid,
    floors: Grid,
}

impl Board {
    pub fn new(width: usize, height: usize) -> Board {
        let mut units = Grid::new();
        let mut floors = Grid::new();

        for _ in 0..height {
            units.push((0..width).map(|_| None).collect());
            floors.push((0..width).map(|_| None).collect());
        }

        Board {
            width,
            height,
            units,
            floors,
        }
    }

    pub fn put_unit(&mut self, position: &Position, mut tile: Option<Box<dyn Element>>) {
        match tile {
            Some(ref mut unit) => unit.mut_container().put(&POSITION, position.clone()),
            None => {}
        }
        self.units[position.y][position.x] = tile;
    }

    pub fn put_floor(&mut self, position: &Position, mut tile: Option<Box<dyn Element>>) {
        match tile {
            Some(ref mut floor) => floor.mut_container().put(&POSITION, position.clone()),
            None => {}
        }
        self.floors[position.y][position.x] = tile;
    }

    pub fn has_unit(&self, position: &Position) -> bool {
        self.units[position.y][position.x].is_some()
    }

    pub fn get_unit(&self, position: &Position) -> Result<&Option<Box<dyn Element>>, ()> {
        self.units
            .get(position.y)
            .ok_or(())?
            .get(position.x)
            .ok_or(())
    }

    pub fn get_unit_unsafe(&self, position: &Position) -> Result<&Box<dyn Element>, ()> {
        self.units
            .get(position.y)
            .ok_or(())?
            .get(position.x)
            .ok_or(())?
            .as_ref()
            .ok_or(())
    }

    pub fn get_unit_mut_unsafe(
        &mut self,
        position: &Position,
    ) -> Result<&mut Box<dyn Element>, ()> {
        self.units
            .get_mut(position.y)
            .ok_or(())?
            .get_mut(position.x)
            .ok_or(())?
            .as_mut()
            .ok_or(())
    }

    pub fn has_floor(&self, position: &Position) -> bool {
        self.floors[position.y][position.x].is_some()
    }

    pub fn get_floor(&self, position: &Position) -> Result<&Option<Box<dyn Element>>, ()> {
        self.floors
            .get(position.y)
            .ok_or(())?
            .get(position.x)
            .ok_or(())
    }

    pub fn get_floor_unsafe(&self, position: &Position) -> Result<&Box<dyn Element>, ()> {
        self.floors
            .get(position.y)
            .ok_or(())?
            .get(position.x)
            .ok_or(())?
            .as_ref()
            .ok_or(())
    }

    pub fn get_floor_mut_unsafe(
        &mut self,
        position: &Position,
    ) -> Result<&mut Box<dyn Element>, ()> {
        self.floors
            .get_mut(position.y)
            .ok_or(())?
            .get_mut(position.x)
            .ok_or(())?
            .as_mut()
            .ok_or(())
    }

    pub fn get_highest(&self, position: &Position) -> Result<Tile, ()> {
        let unit = self.get_unit(position)?;

        if unit.is_some() {
            Ok(Tile::Unit(self.get_unit_unsafe(position)?))
        } else {
            Ok(Tile::Floor(self.get_floor_unsafe(position)?))
        }
    }

    pub fn get_tank_for_ref(&self, player_ref: &PlayerRef) -> Option<&Box<dyn Element>> {
        for row in &self.units {
            for unit in row {
                match unit {
                    Some(unit) => {
                        if unit.container().get_unsafe(&PLAYER_REF).name() == player_ref.name() {
                            return Some(unit);
                        }
                    }
                    None => continue,
                }
            }
        }
        None
    }

    pub fn get_tank_for_ref_mut(
        &mut self,
        player_ref: &PlayerRef,
    ) -> Option<&mut Box<dyn Element>> {
        for row in &mut self.units {
            for unit in row {
                match unit {
                    Some(unit) => {
                        if unit.container().get_unsafe(&PLAYER_REF).name() == player_ref.name() {
                            return Some(unit);
                        }
                    }
                    None => continue,
                }
            }
        }
        None
    }

    pub fn get_all_units(&self) -> Vec<&Box<dyn Element>> {
        self.units
            .iter()
            .map(|row| row.iter())
            .flatten()
            .filter_map(|unit| unit.as_ref())
            .collect()
    }

    pub fn get_all_units_mut(&mut self) -> Vec<&mut Box<dyn Element>> {
        self.units
            .iter_mut()
            .map(|row| row.iter_mut())
            .flatten()
            .filter_map(|unit| unit.as_mut())
            .collect()
    }

    pub fn get_all_floors(&self) -> Vec<&Box<dyn Element>> {
        self.floors
            .iter()
            .map(|row| row.iter())
            .flatten()
            .filter_map(|unit| unit.as_ref())
            .collect()
    }

    pub fn get_all_floors_mut(&mut self) -> Vec<&mut Box<dyn Element>> {
        self.floors
            .iter_mut()
            .map(|row| row.iter_mut())
            .flatten()
            .filter_map(|unit| unit.as_mut())
            .collect()
    }
}

#[typetag::serde]
impl JsonType for Board {}
