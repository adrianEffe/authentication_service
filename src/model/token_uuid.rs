#[derive(Debug)]
pub struct TokenUuid(uuid::Uuid);

impl TokenUuid {
    pub fn new(value: uuid::Uuid) -> TokenUuid {
        TokenUuid(value)
    }

    pub fn get_string(&self) -> String {
        self.0.to_string()
    }
}
