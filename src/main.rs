use crate::state::board::Board;
use crate::state::position::Position;
use crate::state::state::State;
use crate::util::attribute::AttributeContainer;
use crate::util::attribute_impl::BOARD;

pub mod rule;
pub mod ruleset;
pub mod state;
pub mod util;

fn main() {
    let mut state = State::new();
    let board: Board = Board::new(4, 4);
    state.get_container_mut().put(&BOARD, board);
    state.board_put_unit(&Position::new(0, 0), AttributeContainer::new());
    println!("{:?}", state.board_get_unit(&Position::new(0, 0)));
}
