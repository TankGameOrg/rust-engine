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

    pub fn concat(&mut self, other: Response) {
        self.response.extend(other.response)
    }
}

#[typetag::serde]
impl JsonType for Response {}
