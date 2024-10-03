use crate::state::board::{Board, Tile};
use crate::state::position::Position;
use crate::util::attribute::AttributeContainer;
use crate::util::attributes::POSITION;

pub mod state;
pub mod util;

fn main() {
    let mut container: AttributeContainer = AttributeContainer::new_with_class("Class".to_string());
    let position: Position = Position::new(1, 2);
    container.put(&POSITION, position.clone());

    let json = serde_json::to_value(&container).unwrap();
    let json_string = json.to_string();
    let from_json: AttributeContainer = serde_json::from_str(&json_string).unwrap();

    let position_from_json = from_json.get(&POSITION).unwrap();

    println!("{}", serde_json::to_value(&from_json).unwrap());
    println!("{:?}", position_from_json);

    let mut board: Board = Board::new(12, 12);
    board.put_unit(&position, Tile::Unit(Box::new(from_json)));
    let board_json = serde_json::to_value(&board).unwrap();
    let board_json_string = board_json.to_string();
    let board_from_json: Board = serde_json::from_str(&board_json_string).unwrap();
    println!("{}", board_json_string);
    println!("{:?}", board_from_json);
}
