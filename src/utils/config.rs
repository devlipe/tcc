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
    vc_table_size: usize,
    did_table_size: usize,
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
            (
                "network_address".to_string(),
                env::var("NETWORK_ADDRESS").expect("NETWORK_ADDRESS must be set"),
            ),
            (
                "credentials_template_directory".to_string(),
                env::var("CREDENTIALS_TEMPLATE_DIRECTORY")
                    .expect("CREDENTIALS_TEMPLATE_DIRECTORY must be set"),
            ),
            (
                "credentials_sd_directory".to_string(),
                env::var("CREDENTIALS_SD_DIRECTORY").expect("CREDENTIALS_SD_DIRECTORY must be set"),
            ),
        ]),
        vc_table_size: 10,
        did_table_size: 10,
    }
});

impl VariablesConfig {
    pub fn get() -> &'static Self {
        &CONFIG
    }

    pub fn vc_table_size(&self) -> usize {
        self.vc_table_size
    }

    pub fn did_table_size(&self) -> usize {
        self.did_table_size
    }
}

// Implementation of the Config trait for VariablesConfig
impl Config for VariablesConfig {
    fn get_value(&self, key: &str) -> &String {
        // Verify if the key exists and return the value
        self.config.get(key).expect("Key not found")
    }
}
