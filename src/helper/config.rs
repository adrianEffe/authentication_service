/// Configuration settings for the application.
///
/// The `Config` struct holds various configuration parameters required by the application.
/// These include settings for database connection, token management, and Redis configuration.
/// It is designed to be initialized from environment variables, ensuring that sensitive
/// information like keys and URLs are not hardcoded into the application.
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub access_token_private_key: String,
    pub access_token_public_key: String,
    pub access_token_expires_in: String,
    pub access_token_max_age: i64,
    pub refresh_token_private_key: String,
    pub refresh_token_public_key: String,
    pub refresh_token_expires_in: String,
    pub refresh_token_max_age: i64,
    pub redis_url: String,
}

fn get_env(var_name: &str) -> String {
    std::env::var(var_name).unwrap_or_else(|_| panic!("{} must be set in .env", var_name))
}

impl Config {
    /// Initializes a new `Config` instance from environment variables.
    ///
    /// This method reads configuration values from environment variables and constructs
    /// a `Config` struct with these values. If any environment variable is missing or
    /// cannot be parsed (such as integer values), the application will panic with an
    /// appropriate error message. This ensures that all required configuration parameters
    /// are provided before the application starts.
    ///
    /// # Returns
    ///
    /// A `Config` instance initialized with values from the environment.
    ///
    /// # Panics
    ///
    /// This method will panic if any required environment variable is not set or if integer
    /// values cannot be parsed correctly.
    pub fn init() -> Config {
        let database_url = get_env("DATABASE_URL");
        let access_token_private_key = get_env("ACCESS_TOKEN_PRIVATE_KEY");
        let access_token_public_key = get_env("ACCESS_TOKEN_PUBLIC_KEY");
        let access_token_expires_in = get_env("ACCESS_TOKEN_EXPIRES_IN");
        let access_token_max_age = get_env("ACCESS_TOKEN_MAXAGE");
        let refresh_token_private_key = get_env("REFRESH_TOKEN_PRIVATE_KEY");
        let refresh_token_public_key = get_env("REFRESH_TOKEN_PUBLIC_KEY");
        let refresh_token_expires_in = get_env("REFRESH_TOKEN_EXPIRES_IN");
        let refresh_token_max_age = get_env("REFRESH_TOKEN_MAXAGE");
        let redis_url = format!(
            "redis://{}:{}",
            get_env("REDIS_HOST"),
            get_env("REDIS_PORT")
        );

        Config {
            database_url,
            access_token_private_key,
            access_token_public_key,
            access_token_expires_in,
            access_token_max_age: access_token_max_age
                .parse::<i64>()
                .expect("Access token max age failed to parse from .env"),
            refresh_token_private_key,
            refresh_token_public_key,
            refresh_token_expires_in,
            refresh_token_max_age: refresh_token_max_age
                .parse::<i64>()
                .expect("Refresh token max age failed to parse from .env"),
            redis_url,
        }
    }
}
