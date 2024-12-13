use crate::{AppContext, Command, Output, ScreenEvent};
use identity_iota::iota::{IotaClientExt, IotaDocument, IotaIdentityClientExt, NetworkName};
use identity_iota::storage::{JwkDocumentExt, JwkMemStore};
use identity_iota::verification::jws::JwsAlgorithm;
use identity_iota::verification::MethodScope;
use iota_sdk::types::block::output::AliasOutput;
use std::io;
use std::io::Write;
use tokio::sync::watch;
use tokio::time::Instant;

pub struct CreateDIDCommand<'a> {
    context: &'a AppContext,
}

impl Command for CreateDIDCommand<'_> {
    fn execute(&mut self) -> ScreenEvent {
        self.print_tile();
        // Block on the async function using block_in_place
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.handle_did_creation())
        })
        .unwrap_or_else(|_| ScreenEvent::Cancel)
    }

    fn print_tile(&self) {
        Output::clear_screen();
        Output::print_screen_title("Create DID")
    }
}

impl CreateDIDCommand<'_> {
    pub fn new(app_context: &AppContext) -> CreateDIDCommand {
        CreateDIDCommand {
            context: app_context,
        }
    }

    async fn handle_did_creation(&self) -> anyhow::Result<ScreenEvent> {
        let owner = self.get_did_owner();

        let (tx, rx) = watch::channel(true);
        // Spawn the loading animation as a background task
        let animation_handle = tokio::spawn(Output::loading_animation(rx));

        let start = Instant::now();

        let (document, _fragment) = self.create_did().await?;

        Output::print_during_loading(
            format!("Time to create DID: {} s", start.elapsed().as_secs()).as_str(),
        );

        // Clear the line before printing this
        Output::print_during_loading("Saving DID to database");
        self.context.db.save_did_document(&document, &owner)?;

        // Signal the animation to stop
        let _ = tx.send(false);
        // Wait for the animation task to finish
        animation_handle.await?;

        Output::print_during_loading("DID created successfully!");
        // Sleep for 2 seconds to allow the user to read the message
        Output::cooldown().await;

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

    pub async fn create_did(&self) -> anyhow::Result<(IotaDocument, String)> {
        Output::print_during_loading("Creating DID...");
        let (document, fragment): (IotaDocument, String) = self.create_did_document().await?;

        Output::print_during_loading("Creating Alias...");
        let alias_output: AliasOutput = self
            .context
            .client
            .new_did_output(self.context.address, document, None)
            .await?;

        Output::print_during_loading("Publishing DID...");
        let document: IotaDocument = self
            .context
            .client
            .publish_did_output(
                self.context.stronghold_storage.as_secret_manager(),
                alias_output,
            )
            .await?;

        Ok((document, fragment))
    }

    async fn create_did_document(&self) -> anyhow::Result<(IotaDocument, String)> {
        let network_name: NetworkName = self.context.client.network_name().await?;

        let mut document: IotaDocument = IotaDocument::new(&network_name);

        let fragment: String = document
            .generate_method(
                &self.context.storage,
                JwkMemStore::ED25519_KEY_TYPE,
                JwsAlgorithm::EdDSA,
                None,
                MethodScope::VerificationMethod,
            )
            .await?;

        Ok((document, fragment))
    }
}
