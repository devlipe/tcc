use dotenv::dotenv;
use once_cell::sync::Lazy;
use std::{collections::HashMap, env};

// Trait definition for configuration management
pub trait Config {
    fn get_value(&self, key: &str) -> &String;
}

// Struct to hold configuration variables
pub struct VariablesConfig {
    // Store configuration in a key-value manner using a HashMap
    config: HashMap<String, String>,
}


// Singleton instance of VariablesConfig
static CONFIG: Lazy<VariablesConfig> = Lazy::new(|| {
    dotenv().ok(); // Load environment variables

    VariablesConfig {
        config: HashMap::from([
            (
                "api_endpoint".to_string(),
                env::var("API_ENDPOINT").expect("API_ENDPOINT must be set"),
            ),
            (
                "faucet_endpoint".to_string(),
                env::var("FAUCET_ENDPOINT").expect("FAUCET_ENDPOINT must be set"),
            ),
            (
                "stronghold_password".to_string(),
                env::var("STRONGHOLD_PASSWORD").expect("STRONGHOLD_PASSWORD must be set"),
            ),
            (
                "stronghold_path".to_string(),
                env::var("STRONGHOLD_VAULT_PATH").expect("STRONGHOLD_VAULT_PATH must be set"),
            ),
            (
                "sqlite_path".to_string(),
                env::var("SQLITE_PATH").unwrap_or_default(),
            ),
        ]),
    }
});

impl VariablesConfig {
    pub fn get() -> &'static Self {
        &CONFIG
    }
}

// Implementation of the Config trait for VariablesConfig
impl Config for VariablesConfig {


    fn get_value(&self, key: &str) -> &String {
        // Verify if the key exists and return the value
        self.config.get(key).expect("Key not found")
    }
}
