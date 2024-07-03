#[derive(Debug)]
pub struct UserId(uuid::Uuid);

impl UserId {
    pub fn new(id: uuid::Uuid) -> UserId {
        UserId(id)
    }

    pub fn get(&self) -> &uuid::Uuid {
        &self.0
    }
}
