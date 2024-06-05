pub enum Status {
    Success,
    Failure,
}

impl Status {
    fn raw_value(&self) -> &str {
        match &self {
            Status::Success => "success",
            Status::Failure => "failure",
        }
    }
}

pub fn response_message(status: &Status, message: &str) -> serde_json::Value {
    serde_json::json!({
        "status": status.raw_value(),
        "message": message,
    })
}
