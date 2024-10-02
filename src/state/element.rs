use crate::attribute::attribute_container::JsonType;
use crate::state::position::Position;

trait Element : JsonType {
    fn position(&self) -> &Position;


}

