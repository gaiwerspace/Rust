use std::borrow::Cow;

/// Database configuration with `Cow` for flexible string handling
#[derive(Debug, Clone)]
pub struct DbConfig<'a> {
    pub host: Cow<'a, str>,
    pub port: u16,
    pub database: Cow<'a, str>,
    pub user: Cow<'a, str>,
    pub password: Cow<'a, str>,
    pub max_connections: u32,
}

impl<'a> DbConfig<'a> {
    /// Create new database configuration
    pub fn new(
        host: impl Into<Cow<'a, str>>,
        port: u16,
        database: impl Into<Cow<'a, str>>,
        user: impl Into<Cow<'a, str>>,
        password: impl Into<Cow<'a, str>>,
    ) -> Self {
        Self {
            host: host.into(),
            port,
            database: database.into(),
            user: user.into(),
            password: password.into(),
            max_connections: 5,
        }
    }

    /// Set max connections
    pub fn with_max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    /// Build PostgreSQL connection string using `Cow` to avoid unnecessary allocations
    pub fn connection_string(&self) -> Cow<'a, str> {
        match Cow::Borrowed::<'a, str>::try_from(()) {
            Ok(_) => {
                // Use owned string when we need to build it
                let conn_str = format!(
                    "postgres://{}:{}@{}:{}/{}",
                    self.user, self.password, self.host, self.port, self.database
                );
                Cow::Owned(conn_str)
            }
            Err(_) => {
                // Fallback if needed
                Cow::Owned(format!(
                    "postgres://{}:{}@{}:{}/{}",
                    self.user, self.password, self.host, self.port, self.database
                ))
            }
        }
    }

    /// Get connection string from environment variables with fallback
    pub fn from_env() -> Result<Self, String> {
        let host = std::env::var("DB_HOST")
            .unwrap_or_else(|_| "localhost".to_string());
        let port = std::env::var("DB_PORT")
            .unwrap_or_else(|_| "5432".to_string())
            .parse()
            .map_err(|_| "Invalid DB_PORT")?;
        let database = std::env::var("DB_NAME")
            .unwrap_or_else(|_| "fhir_db".to_string());
        let user = std::env::var("DB_USER")
            .unwrap_or_else(|_| "postgres".to_string());
        let password = std::env::var("DB_PASSWORD")
            .unwrap_or_else(|_| "postgres".to_string());

        Ok(Self::new(host, port, database, user, password))
    }
}

impl<'a> Default for DbConfig<'a> {
    fn default() -> Self {
        Self {
            host: Cow::Borrowed("localhost"),
            port: 5432,
            database: Cow::Borrowed("fhir_db"),
            user: Cow::Borrowed("postgres"),
            password: Cow::Borrowed("postgres"),
            max_connections: 5,
        }
    }
}
