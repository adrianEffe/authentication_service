#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub access_token_private_key: String,
    pub access_token_public_key: String,
    pub access_token_expires_in: String,
    pub access_token_max_age: i64,
}

fn get_env(var_name: &str) -> String {
    std::env::var(var_name).unwrap_or_else(|_| panic!("{} must be set in .env", var_name))
}

impl Config {
    pub fn init() -> Config {
        let database_url = get_env("DATABASE_URL");
        let access_token_private_key = get_env("ACCESS_TOKEN_PRIVATE_KEY");
        let access_token_public_key = get_env("ACCESS_TOKEN_PUBLIC_KEY");
        let access_token_expires_in = get_env("ACCESS_TOKEN_EXPIRED_IN");
        let access_token_max_age = get_env("ACCESS_TOKEN_MAXAGE");

        Config {
            database_url,
            access_token_private_key,
            access_token_public_key,
            access_token_expires_in,
            access_token_max_age: access_token_max_age.parse::<i64>().unwrap(),
        }
    }
}
