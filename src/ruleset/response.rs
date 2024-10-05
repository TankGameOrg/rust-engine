use crate::util::attribute::JsonType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    response: Vec<String>,
}

impl Response {
    pub fn new_empty() -> Response {
        Response {
            response: Vec::new(),
        }
    }

    pub fn new(response: Vec<String>) -> Response {
        Response { response }
    }
}

#[typetag::serde]
impl JsonType for Response {}
