pub enum Status {
    Success,
    Failure,
}

impl Status {
    fn raw_value(&self) -> String {
        match &self {
            Status::Success => String::from("success"),
            Status::Failure => String::from("failure"),
        }
    }
}

pub fn response_message(status: &Status, message: &str) -> serde_json::Value {
    serde_json::json!({
        "status": status.raw_value(),
        "message": message,
    })
}
