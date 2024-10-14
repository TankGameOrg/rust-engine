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
