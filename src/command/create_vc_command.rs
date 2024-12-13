use crate::{
    is_command_available, AppContext, Command, Did, Input, ListDIDsCommand, Output, ScreenEvent,
};
use crossterm::style::Stylize;
use identity_iota::iota::IotaDocument;
use std::io;

pub struct CreateVCCommand<'a> {
    context: &'a AppContext,
}

impl Command for CreateVCCommand<'_> {
    fn execute(&mut self) -> ScreenEvent {
        // Block on the async function using block_in_place
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.handle_vc_creation())
        })
        .unwrap_or_else(|_| ScreenEvent::Cancel)
    }

    fn print_tile(&self) {
        Output::clear_screen();
        Output::print_screen_title("Create VC")
    }
}

impl CreateVCCommand<'_> {
    pub fn new(context: &AppContext) -> CreateVCCommand {
        CreateVCCommand { context }
    }

    async fn handle_vc_creation(&self) -> anyhow::Result<ScreenEvent> {
        let (issuer_did, issuer_name, holder_did, holder_name, ok) = self.choose_dids().await?;

        match ok {
            ScreenEvent::Success => {
                self.print_information_status(&issuer_did, &issuer_name, &holder_did, &holder_name);
                println!("Creating VC with theses credentials");
                Output::cooldown().await;
                self.create_credential()?;
            }
            _ => return Ok(ScreenEvent::Cancel),
        }

        Ok(ScreenEvent::Success)
    }

    async fn choose_dids(
        &self,
    ) -> anyhow::Result<(IotaDocument, String, IotaDocument, String, ScreenEvent)> {
        let dids = self.context.db.get_stored_dids()?;
        let (mut issuer_did, mut issuer_name): (IotaDocument, String) =
            self.get_issuer_did(&dids).await;

        let (mut holder_did, mut holder_name): (IotaDocument, String) =
            self.get_holder_did(&dids).await;

        let ok = self
            .confirm_user_selection(
                &dids,
                &mut issuer_did,
                &mut issuer_name,
                &mut holder_did,
                &mut holder_name,
            )
            .await?;
        Ok((issuer_did, issuer_name, holder_did, holder_name, ok))
    }

    fn create_credential( &self ) -> anyhow::Result<()> {
        let _editor = self.choose_editor()?;
        
        
        Ok(())
    }

    fn choose_editor(&self) ->  anyhow::Result<String> {
        self.print_tile();
        let editors = ["nvim", "vim", "nano", "vi", "code"];

        // Categorize editors into available and unavailable
        let (available_editors, unavailable_editors): (Vec<&str>, Vec<&str>) = editors
            .iter()
            .partition(|&&editor| is_command_available(editor));

        // Check if any editors are available
        if available_editors.is_empty() {
            eprintln!("No supported editor found (nvim, vim, nano, vi). Exiting.");
            return Err(anyhow::anyhow!("No supported editor found"));
        }
        
        // Print unavailable editors to the user
        if !unavailable_editors.is_empty() {
            println!("Unavailable editors:");
            for (index, editor) in unavailable_editors.iter().enumerate() {
                println!("{}: {}", index + 1, editor.red());
            }
        }
        
        // Print available editors to the user
        println!("Available editors:");
        for (index, editor) in available_editors.iter().enumerate() {
            println!("{}: {}", index + 1, editor.green());
        }
        
        // Prompt the user to choose an editor
        println!("Please select an editor:");

        let input = Input::get_number_input(1, available_editors.len());
        
        Ok(available_editors[input - 1].to_string())
    }

    async fn confirm_user_selection(
        &self,
        dids: &Vec<Did>,
        issuer_did: &mut IotaDocument,
        issuer_name: &mut String,
        holder_did: &mut IotaDocument,
        holder_name: &mut String,
    ) -> anyhow::Result<ScreenEvent> {
        loop {
            self.print_information_status(&issuer_did, &issuer_name, &holder_did, &holder_name);
            println!(
                "{} {} {} {} {} {}",
                "\nPress",
                "enter to continue".green().bold(),
                "or type",
                "'back' to go back ".red().bold(),
                "to the main menu, or",
                "'issuer'/'holder' to open the selection".blue().bold()
            );
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            // Trim input and make it lowercase
            let input = input.trim().to_lowercase();

            match input.as_str() {
                "back" => return Err(anyhow::anyhow!("User cancelled operation")),
                "issuer" => {
                    (*issuer_did, *issuer_name) = self.get_issuer_did(&dids).await;
                }
                "holder" => {
                    (*holder_did, *holder_name) = self.get_holder_did(&dids).await;
                }
                "" => break Ok(ScreenEvent::Success),
                _ => continue,
            }
        }
    }

    fn print_information_status(
        &self,
        issuer_did: &IotaDocument,
        issuer_name: &String,
        holder_did: &IotaDocument,
        holder_name: &String,
    ) {
        self.print_tile();
        println!("Issuer DID: {} {}", issuer_name, issuer_did.id());
        println!("Holder DID: {} {}", holder_name, holder_did.id());
    }

    pub async fn get_issuer_did(&self, dids: &Vec<Did>) -> (IotaDocument, String) {
        self.print_tile();
        ListDIDsCommand::display_dids_table(dids);
        println!("Select the DID row to use as the issuer:");
        let did: Did = self.get_did(dids);
        (
            did.resolve_to_iota_document(&self.context.resolver).await,
            did.name().to_string(),
        )
    }

    pub async fn get_holder_did(&self, dids: &Vec<Did>) -> (IotaDocument, String) {
        self.print_tile();
        ListDIDsCommand::display_dids_table(dids);
        println!("Select the DID row to use as the holder:");

        let did: Did = self.get_did(dids);
        (
            did.resolve_to_iota_document(&self.context.resolver).await,
            did.name().to_string(),
        )
    }

    pub fn get_did(&self, dids: &Vec<Did>) -> Did {
        let user_input = Input::get_number_input(1, dids.len());
        let selected_did = dids.get(user_input - 1).unwrap();
        selected_did.clone()
    }
}
