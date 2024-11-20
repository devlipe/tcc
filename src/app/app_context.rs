use crate::{Config, DBConnector, Output, SQLiteConnector, VariablesConfig};
use identity_iota::storage::Storage;
use identity_stronghold::StrongholdStorage;
use iota_sdk::client::secret::stronghold::StrongholdSecretManager;
use iota_sdk::client::{Client, Password};
use tokio::sync::watch;

pub struct AppContext {
    pub tangle_client: Client,
    pub db: Box<dyn DBConnector>,
    pub stronghold_storage: StrongholdStorage,
    pub storage: Storage<StrongholdStorage, StrongholdStorage>,
}

impl AppContext {
    pub async fn build_app_context_with_loading() -> Self {
        let (tx, rx) = watch::channel(true);
        // Spawn the loading animation as a background task
        let animation_handle = tokio::spawn(Output::loading_animation(rx));

        let context = AppContext::my_app_context().await;

        // Signal the animation to stop
        let _ = tx.send(false);

        // Wait for the animation task to finish
        animation_handle.await.unwrap();

        context
    }

    pub async fn my_app_context() -> Self {
        let config = VariablesConfig::get();
        let tangle_client = AppContext::get_tangle_client(config).await.unwrap();
        let db = AppContext::get_sqlite_database(config);
        let stronghold_storage = AppContext::get_stronghold_storage(config);
        let storage = Storage::new(stronghold_storage.clone(), stronghold_storage.clone());

        AppContext {
            tangle_client,
            db,
            stronghold_storage,
            storage,
        }
    }

    fn get_sqlite_database(config: &dyn Config) -> Box<dyn DBConnector> {
        let sqlite_path: &String = config.get_value("sqlite_path");
        let sqlite = SQLiteConnector::new(sqlite_path).unwrap_or_default();

        Box::new(sqlite)
    }
    async fn get_tangle_client(config: &dyn Config) -> anyhow::Result<Client> {
        let client: Client = Client::builder()
            .with_primary_node(config.get_value("api_endpoint"), None)?
            .finish()
            .await?;
        Ok(client)
    }

    fn get_stronghold_storage(config: &dyn Config) -> StrongholdStorage {
        // Stronghold password.
        let password = Password::from(config.get_value("stronghold_password").to_owned());
        // Stronghold snapshot path.
        let stronghold_path: std::path::PathBuf =
            std::path::PathBuf::from(config.get_value("stronghold_path").to_owned());

        let mut start = std::time::Instant::now();

        let stronghold = StrongholdSecretManager::builder()
            .password(password.clone())
            .build(stronghold_path.clone()).unwrap();

        println!("Time to get stronghold: {:?}", start.elapsed());
        start = std::time::Instant::now();
        let stronghold_storage = StrongholdStorage::new(stronghold);
        println!("Time to get stronghold storage: {:?}", start.elapsed());

        stronghold_storage
    }
}
