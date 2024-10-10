use crate::state::position::Position;
use crate::state::state::{Reference, State};
use crate::util::attribute::AttributeContainer;
use crate::util::attribute_impl::BOARD;

type Grid = Vec<Vec<Option<Reference>>>;

#[derive(Debug)]
pub struct Board {
    width: usize,
    height: usize,
    units: Grid,
    floors: Grid,
}

impl State {
    pub fn get_board(&self) -> &Board {
        self.get_container().get_unsafe(&BOARD)
    }

    pub fn get_board_mut(&mut self) -> &mut Board {
        self.get_container_mut().get_mut_unsafe(&BOARD)
    }

    pub fn board_put_unit(&mut self, position: &Position, unit: AttributeContainer) {
        let reference = self.put(unit);
        self.get_board_mut().put_unit(position, reference);
    }

    pub fn board_put_floor(&mut self, position: &Position, unit: AttributeContainer) {
        let reference = self.put(unit);
        self.get_board_mut().put_floor(position, reference);
    }

    pub fn board_get_unit(&self, position: &Position) -> Option<&AttributeContainer> {
        let reference = self.get_board().get_unit(position)?;
        self.get(&reference)
    }

    pub fn board_get_unit_mut(&mut self, position: &Position) -> Option<&mut AttributeContainer> {
        let reference = self.get_board().get_unit(position)?;
        self.get_mut(&reference)
    }

    pub fn board_get_floor(&self, position: &Position) -> Option<&AttributeContainer> {
        let reference = self.get_board().get_floor(position)?;
        self.get(&reference)
    }

    pub fn board_get_floor_mut(&mut self, position: &Position) -> Option<&mut AttributeContainer> {
        let reference = self.get_board().get_floor(position)?;
        self.get_mut(&reference)
    }
}

impl Board {
    pub fn new(width: usize, height: usize) -> Self {
        let units = vec![vec![None; width]; height];
        let floors = vec![vec![None; width]; height];
        Board {
            width,
            height,
            units,
            floors,
        }
    }

    #[inline]
    fn is_position_invalid(&self, position: &Position) -> bool {
        position.x >= self.width || position.y >= self.height
    }

    #[inline]
    fn panic_if_out_of_bounds(&self, position: &Position) {
        if self.is_position_invalid(position) {
            panic!("{}", format!("Invalid position: {:?}", position).as_str());
        }
    }

    pub fn put_unit(&mut self, position: &Position, unit: Reference) {
        self.panic_if_out_of_bounds(&position);
        self.units[position.y][position.x] = Some(unit);
    }

    pub fn put_floor(&mut self, position: &Position, floor: Reference) {
        self.panic_if_out_of_bounds(&position);
        self.floors[position.y][position.x] = Some(floor);
    }

    pub fn get_unit(&self, position: &Position) -> Option<Reference> {
        self.panic_if_out_of_bounds(&position);
        Some(
            self.units
                .get(position.y)?
                .get(position.x)?
                .as_ref()?
                .clone(),
        )
    }

    pub fn get_floor(&self, position: &Position) -> Option<Reference> {
        self.panic_if_out_of_bounds(&position);
        Some(
            self.floors
                .get(position.y)?
                .get(position.x)?
                .as_ref()?
                .clone(),
        )
    }
}
