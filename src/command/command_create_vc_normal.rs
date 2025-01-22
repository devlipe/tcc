use crate::{
    is_command_available, utils, AppContext, Command, Config, Did, Input, ListDIDsCommand, Output,
    ScreenEvent, VariablesConfig, VerifyVCCommand,
};

use colored::*;
use identity_iota::core::{FromJson, Url};
use identity_iota::credential::{Credential, CredentialBuilder, Jwt, Subject};
use identity_iota::did::DID;
use identity_iota::iota::IotaDocument;
use identity_iota::storage::{JwkDocumentExt, JwsSignatureOptions};
use serde_json::Value;
use std::path::Path;
use std::{fs, io};

pub struct CreateVCNormalCommand<'a> {
    context: &'a AppContext,
}

impl Command for CreateVCNormalCommand<'_> {
    fn execute(&mut self) -> ScreenEvent {
        // Block on the async function using block_in_place
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.handle_vc_normal_creation())
        })
        .unwrap_or_else(|_| ScreenEvent::Cancel)
    }

    fn print_tile(&self) {
        Output::clear_screen();
        Output::print_screen_title("Create VC")
    }
}

impl CreateVCNormalCommand<'_> {
    pub fn new(context: &AppContext) -> CreateVCNormalCommand {
        CreateVCNormalCommand { context }
    }

    async fn handle_vc_normal_creation(&self) -> anyhow::Result<ScreenEvent> {
        let (issuer_document, issuer, holder_document, holder, ok) = self.choose_dids().await?;

        match ok {
            ScreenEvent::Success => {
                self.print_information_status(
                    &issuer_document,
                    &issuer.name().to_string(),
                    &holder_document,
                    &holder.name().to_string(),
                );
                println!("Creating VC with theses credentials");
                Output::cooldown().await;
            }
            _ => return Ok(ScreenEvent::Cancel),
        }

        let (path, template): (String, String) = self.create_credential()?;

        let credential_type = Output::snake_to_camel_case(&template);

        let mut json: Value = utils::read_json_file(&path)?;

        json = utils::insert_holder_did(&mut json, holder_document.id().as_str())?;

        let subject: Subject = Subject::from_json_value(json)?;

        let credential: Credential = CredentialBuilder::default()
            .issuer(Url::parse(issuer_document.id().as_str())?)
            .type_(&credential_type)
            .non_transferable(true)
            .subject(subject)
            .build()?;

        let credential_jwt: Jwt = issuer_document
            .create_credential_jwt(
                &credential,
                &self.context.storage,
                utils::extract_kid(&issuer_document)?.as_str(),
                &JwsSignatureOptions::default(),
                None,
            )
            .await?;

        let _decoded_credential =
            VerifyVCCommand::verify_normal_vc(&credential_jwt, &issuer_document)?;

        utils::pretty_print_json(
            "VC Created",
            _decoded_credential.credential.to_string().as_str(),
        );

        // Save the credential to the database
        self.context.db.save_vc(
            &credential_jwt.as_str(),
            issuer.id(),
            holder.id(),
            &credential_type,
            false,
        )?;

        Ok(ScreenEvent::Success)
    }

    pub(crate) async fn choose_dids(
        &self,
    ) -> anyhow::Result<(IotaDocument, Did, IotaDocument, Did, ScreenEvent)> {
        let dids = self.context.db.get_stored_dids()?;

        // Check if there are any DIDs stored
        if dids.is_empty() {
            println!(
                "{}",
                "No DIDs found. Please create a DID first.".red().bold()
            );
            return Err(anyhow::anyhow!("No DIDs found"));
        }

        let (mut issuer_document, mut issuer): (IotaDocument, Did) =
            self.get_issuer_did(&dids).await;

        let (mut holder_document, mut holder): (IotaDocument, Did) =
            self.get_holder_did(&dids).await;

        let ok = self
            .confirm_user_selection(
                &dids,
                &mut issuer_document,
                &mut issuer,
                &mut holder_document,
                &mut holder,
            )
            .await?;
        Ok((issuer_document, issuer, holder_document, holder, ok))
    }

    pub(crate) fn create_credential(&self) -> anyhow::Result<(String, String)> {
        let template = self.choose_credential_template()?;
        let editor = self.choose_editor()?;
        let path = self.copy_template_to_file(&template)?;

        utils::edit_file(editor, &path)?;

        Ok((path, template))
    }

    fn copy_template_to_file(&self, template: &String) -> anyhow::Result<String> {
        let copy_file = utils::random_credential_path();

        // create a file variable that is the directory of templates + the template name
        let file_path =
            Path::new(VariablesConfig::get().get_value("credentials_template_directory"))
                .join(template);

        // copy the template to the file
        fs::copy(file_path, &copy_file)?;

        Ok(copy_file.to_str().unwrap().to_string())
    }

    fn choose_credential_template(&self) -> anyhow::Result<String> {
        self.print_tile();
        let templates = self.get_available_templates();

        // Print available templates to the user
        println!("Available templates:");
        for (index, template) in templates.iter().enumerate() {
            println!(
                "{}: {}",
                index + 1,
                Output::snake_to_title_case(template).blue()
            );
        }

        // Prompt the user to choose a template
        println!("Please select a template:");

        let input = Input::get_number_input(1, templates.len());

        Ok(templates[input - 1].to_string())
    }

    fn get_available_templates(&self) -> Vec<String> {
        let directory = VariablesConfig::get().get_value("credentials_template_directory");
        match fs::read_dir(directory) {
            Ok(entries) => entries
                .filter_map(|entry| entry.ok())
                .filter_map(|entry| entry.file_name().into_string().ok())
                .collect(),
            Err(_) => vec![],
        }
    }

    pub fn choose_editor(&self) -> anyhow::Result<String> {
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
            for (_, editor) in unavailable_editors.iter().enumerate() {
                println!("- : {}", editor.red());
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
        issuer: &mut Did,
        holder_did: &mut IotaDocument,
        holder: &mut Did,
    ) -> anyhow::Result<ScreenEvent> {
        loop {
            self.print_information_status(
                &issuer_did,
                &issuer.name().to_string(),
                &holder_did,
                &holder.name().to_string(),
            );
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
                    (*issuer_did, *issuer) = self.get_issuer_did(&dids).await;
                }
                "holder" => {
                    (*holder_did, *holder) = self.get_holder_did(&dids).await;
                }
                "" => break Ok(ScreenEvent::Success),
                _ => continue,
            }
        }
    }

    pub(crate) fn print_information_status(
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

    async fn get_issuer_did(&self, dids: &Vec<Did>) -> (IotaDocument, Did) {
        self.print_tile();
        let index = Output::display_with_pagination(
            dids,
            Self::choose_issuer_table,
            VariablesConfig::get().did_table_size(),
            true,
            Some(Box::new(|| self.print_tile())),
        );
        let did: Did = self.get_did(dids, index);
        (
            did.resolve_to_iota_document(&self.context.resolver).await,
            did,
        )
    }

    fn choose_issuer_table(dids: &Vec<Did>, first_row_index: usize) {
        ListDIDsCommand::display_dids_table(dids, first_row_index);
        println!("Select the DID row to use as the issuer:");
    }

    async fn get_holder_did(&self, dids: &Vec<Did>) -> (IotaDocument, Did) {
        self.print_tile();
        let index = Output::display_with_pagination(
            dids,
            Self::choose_holder_table,
            VariablesConfig::get().did_table_size(),
            true,
            Some(Box::new(|| self.print_tile())),
        );

        let did: Did = self.get_did(dids, index);
        (
            did.resolve_to_iota_document(&self.context.resolver).await,
            did,
        )
    }

    fn choose_holder_table(dids: &Vec<Did>, first_row_index: usize) {
        ListDIDsCommand::display_dids_table(dids, first_row_index);
        println!("Select the DID row to use as the holder:");
    }

    fn get_did(&self, dids: &Vec<Did>, index: usize) -> Did {
        let selected_did = dids.get(index - 1).unwrap();
        selected_did.clone()
    }
}
