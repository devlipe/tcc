use std::io;
use std::io::Write;
use crate::{AppContext, Command, Output, ScreenEvent};
use identity_iota::iota::{IotaClientExt, IotaDocument, IotaIdentityClientExt, NetworkName};
use identity_iota::storage::{JwkDocumentExt, JwkMemStore};
use identity_iota::verification::jws::JwsAlgorithm;
use identity_iota::verification::MethodScope;
use iota_sdk::types::block::output::AliasOutput;
use tokio::sync::watch;
use tokio::time::{sleep, Duration};

pub struct CreateDIDCommand;

impl Command for CreateDIDCommand {
    fn execute(&mut self, context: &AppContext) -> ScreenEvent {
        self.print_tile();
        // Block on the async function using block_in_place
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.handle_did_creation(context))
        })
        .unwrap_or_else(|_| ScreenEvent::Cancel)
    }

    fn print_tile(&self) {
        Output::clear_screen();
        Output::print_screen_title("Create DID")
    }
}

impl CreateDIDCommand {
    pub fn new() -> CreateDIDCommand {
        CreateDIDCommand
    }

    pub async fn handle_did_creation(&self, context: &AppContext) -> anyhow::Result<ScreenEvent> {
        let owner = self.get_did_owner();

        let (tx, rx) = watch::channel(true);
        // Spawn the loading animation as a background task
        let animation_handle = tokio::spawn(Output::loading_animation(rx));

        let (document, _fragment) = self.create_did(context).await?;
        
        // Clear the line before printing this
        Output::print_during_loading("Saving DID to database");
        context.db.save_did_document(&document, &owner)?;

        // Signal the animation to stop
        let _ = tx.send(false);
        // Wait for the animation task to finish
        animation_handle.await?;

        Output::print_during_loading("DID created successfully!");
        // Sleep for 2 seconds to allow the user to read the message
        sleep(Duration::from_secs(2)).await;

        Ok(ScreenEvent::Success)
    }

    fn get_did_owner(&self) -> String {
        loop {
            print!("Please enter a name to be linked with the DID: ");
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();

            let trimmed_input = input.trim();

            // Check if input is a blank line
            if trimmed_input.is_empty() {
                println!("Input cannot be blank. Please try again.");
                continue;
            }

            return trimmed_input.to_string();
        }
    }

    pub async fn create_did(&self, context: &AppContext) -> anyhow::Result<(IotaDocument, String)> {
        Output::print_during_loading("Creating DID...");
        let (document, fragment): (IotaDocument, String) =
            self.create_did_document(context).await?;

        Output::print_during_loading("Creating Alias...");
        let alias_output: AliasOutput = context
            .client
            .new_did_output(context.address, document, None)
            .await?;

        Output::print_during_loading("Publishing DID...");
        let document: IotaDocument = context
            .client
            .publish_did_output(context.stronghold_storage.as_secret_manager(), alias_output)
            .await?;

        Ok((document, fragment))
    }

    async fn create_did_document(
        &self,
        context: &AppContext,
    ) -> anyhow::Result<(IotaDocument, String)> {
        let network_name: NetworkName = context.client.network_name().await?;

        let mut document: IotaDocument = IotaDocument::new(&network_name);

        let fragment: String = document
            .generate_method(
                &context.storage,
                JwkMemStore::ED25519_KEY_TYPE,
                JwsAlgorithm::EdDSA,
                None,
                MethodScope::VerificationMethod,
            )
            .await?;

        Ok((document, fragment))
    }
}
