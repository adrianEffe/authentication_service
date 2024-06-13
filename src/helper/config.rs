#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
}

fn get_env(var_name: &str) -> String {
    std::env::var(var_name).unwrap_or_else(|_| panic!("{} must be set in .env", var_name))
}

impl Config {
    pub fn init() -> Config {
        let database_url = get_env("DATABASE_URL");

        Config { database_url }
    }
}
