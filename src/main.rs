use crate::attribute::attribute_container::{Attribute, AttributeContainer};

pub mod state;
pub mod attribute;

fn main() {
    let attribute : Attribute<i32> = Attribute::new("my_key".to_string());

    let mut container: AttributeContainer = AttributeContainer::new_with_class("ClassName".to_string());
    container.put(&attribute, 10i32);

    let json = serde_json::to_value(&container).unwrap();
    let json_string = json.to_string();
    let from_json: AttributeContainer = serde_json::from_str(&json_string).unwrap();

    let int_value = from_json.get(&attribute).unwrap();

    println!("{}", serde_json::to_value(&from_json).unwrap());
    println!("{}", int_value);
}
