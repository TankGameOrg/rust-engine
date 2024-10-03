use serde::{Deserialize, Serialize};
use crate::attribute::attribute::JsonType;
use crate::state::element::{Element, Floor, Unit};
use crate::state::position::Position;

#[derive(Serialize, Deserialize, Debug)]
pub enum Tile {
    Empty,
    Unit(Box<dyn Unit>),
    Floor(Box<dyn Floor>),
}

impl Tile {
    pub fn is_empty(&self) -> bool {
        match self {
            Tile::Empty => true,
            _ => false,
        }
    }

    pub fn is_unit(&self) -> bool {
        match self {
            Tile::Unit(_) => true,
            _ => false,
        }
    }

    pub fn is_floor(&self) -> bool {
        match self {
            Tile::Floor(_) => true,
            _ => false,
        }
    }

    pub fn is_walkable(&self) -> bool {
        match self {
            Tile::Empty => true,
            Tile::Unit(_) => false,
            Tile::Floor(floor) => floor.is_walkable(),
        }
    }
}

// Grid[y][x]
pub type Grid = Vec<Vec<Tile>>;

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
            units.push((0..width).map(|_| Tile::Empty).collect());
            floors.push((0..width).map(|_| Tile::Empty).collect());
        }

        Board { width, height, units, floors }
    }

    pub fn put_unit(&mut self, position: &Position, tile: Tile) {
        if tile.is_floor() {
            panic!("Got floor when placing unit");
        }
        self.units[position.y][position.x] = tile;
    }

    pub fn put_floor(&mut self, position: &Position, tile: Tile) {
        if tile.is_unit() {
            panic!("Got unit when placing floor");
        }
        self.floors[position.y][position.x] = tile;
    }

    pub fn get_unit(&self, position: &Position) -> Option<&Tile> {
        self.units.get(position.y).map(|row| row.get(position.x)).flatten()
    }

    pub fn get_floor(&self, position: &Position) -> Option<&Tile> {
        self.floors.get(position.y).map(|row| row.get(position.x)).flatten()
    }
}

#[typetag::serde]
impl JsonType for Board {}
