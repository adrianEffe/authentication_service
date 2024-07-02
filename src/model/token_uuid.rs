#[derive(Debug)]
pub struct TokenUuid(uuid::Uuid);

impl TokenUuid {
    pub fn get_string(&self) -> String {
        self.0.to_string()
    }
}
