use crate::attribute::attribute::{Attribute, AttributeContainer, TEST_INT32};
use crate::state::element::Element;
use crate::state::position::Position;

pub mod state;
pub mod attribute;

fn main() {
    let attribute : &Attribute<i32> = &TEST_INT32;

    let mut container: AttributeContainer = AttributeContainer::new_with_class("ClassName".to_string());
    container.put(&attribute, 10i32);
    container.set_position(Position::new(1, 2));

    let json = serde_json::to_value(&container).unwrap();
    let json_string = json.to_string();
    let from_json: AttributeContainer = serde_json::from_str(&json_string).unwrap();

    let int_value = from_json.get(&attribute).unwrap();

    println!("{}", serde_json::to_value(&from_json).unwrap());
    println!("{}", int_value);
}
