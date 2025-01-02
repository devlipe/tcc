use crate::{
    utils, AppContext, Command, Config, CreateVCNormalCommand, Did, Input, Output, ScreenEvent,
    VariablesConfig,
};
use anyhow::Result;
use colored::Colorize;
use identity_iota::core::{FromJson, Url};
use identity_iota::credential::{Credential, CredentialBuilder, Jws, Subject};
use identity_iota::did::DID;
use identity_iota::iota::IotaDocument;
use identity_iota::storage::{JwkDocumentExt, JwsSignatureOptions};
use sd_jwt_payload::{Disclosure, SdJwt, SdObjectEncoder};
use serde_json::Value;
use std::fs::File;

pub struct CreateVCSDCommand<'a> {
    context: &'a AppContext,
    create_vc: CreateVCNormalCommand<'a>,
}

impl<'a> Command for CreateVCSDCommand<'a> {
    fn execute(&mut self) -> ScreenEvent {
        // Block on the async function using block_in_place
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.handle_vc_sd_creation())
        })
        .unwrap_or_else(|e| {
            println!("Error: {}", e);
            Input::wait_for_user_input("Press enter to continue");
            ScreenEvent::Cancel
        })
    }

    fn print_tile(&self) {
        println!("Create Verifiable Credential with Selective Disclosure");
    }
}

impl<'a> CreateVCSDCommand<'a> {
    pub fn new(context: &'a AppContext) -> Self {
        Self {
            context,
            create_vc: CreateVCNormalCommand::new(context),
        }
    }

    pub async fn handle_vc_sd_creation(&self) -> anyhow::Result<ScreenEvent> {
        let (issuer_document, issuer, holder_document, holder) = self.select_dids().await?;

        let (path, template): (String, String) = self.create_vc.create_credential()?;

        let credential_type = Output::snake_to_camel_case(&template);

        let json: Value = utils::build_json_credential(holder_document, &path)?;

        let json_paths = self.get_json_sd_paths(template, &json);

        println!("{:?}", json_paths);

        let subject: Subject = Subject::from_json_value(json)?;

        let credential: Credential = CredentialBuilder::default()
            .type_(&credential_type)
            .issuer(Url::parse(issuer_document.id().as_str())?)
            .non_transferable(true)
            .subject(subject)
            .build()?;

        let payload = credential.serialize_jwt(None)?;

        let (encoded_payload, disclosures) = self.add_disclosures(&payload, json_paths)?;

        let jwt: Jws = issuer_document
            .create_jws(
                &self.context.storage,
                utils::extract_kid(&issuer_document)?.as_str(),
                encoded_payload.as_bytes(),
                &JwsSignatureOptions::default(),
            )
            .await?;
        
        println!("JWS: {:?}", jwt);

        let disclosures: Vec<String> = disclosures
            .into_iter()
            .map(|disclosure| disclosure.to_string())
            .collect();

        let sd_jwt_str = SdJwt::new(jwt.into(), disclosures, None).presentation();

        self.context.db.save_vc(
            &sd_jwt_str,
            issuer.id(),
            holder.id(),
            &credential_type,
            true,
        )?;

        utils::pretty_print_json("VC-SD Created successfully!", &encoded_payload);

        Input::wait_for_user_input("Press enter to continue");

        Ok(ScreenEvent::Success)
    }

    fn add_disclosures(
        &self,
        payload: &String,
        json_paths: Vec<String>,
    ) -> Result<(String, Vec<Disclosure>)> {
        let mut encoder = SdObjectEncoder::new(&payload)?;

        let disclosures: Vec<Disclosure> = json_paths
            .iter()
            .map(|path| encoder.conceal(path, None).unwrap())
            .collect::<Vec<Disclosure>>();

        // Add the `_sd_alg` property.
        encoder.add_sd_alg_property();

        let encoded_payload = encoder.try_to_string()?;

        Ok((encoded_payload, disclosures))
    }

    fn get_json_sd_paths(&self, template: String, json: &Value) -> Vec<String> {
        let path = Self::get_sd_file_path(&template);
        let json_paths = utils::generate_json_paths(&json, "/vc/credentialSubject");

        // check if the file exists
        if !utils::file_exists(&path) {
            println!("File does not exist: {}", path);
            println!("Creating...");
            let mut file = File::create(&path).unwrap();

            utils::prepend_comment_to_file(&mut file).unwrap();
            utils::write_vec_to_file(&mut file, &json_paths).unwrap();
            println!("File created!");
        }
        self.edit_sd_paths_file(&path);
        let sd_paths = self.get_sd_paths_from_file(path);
        sd_paths
    }

    fn edit_sd_paths_file(&self, path: &String) {
        // Ask the user if they want to edit the file (Default: No)
        println!(
            "{} {} {} {}",
            "You",
            "should comment the fields".bold().red(),
            "that you",
            "don't want to selective disclose".bold().red()
        );
        let user_input = Input::wait_for_user_input("Do you want to edit the file? (y/N)");
        if user_input == "y" {
            let editor = self.create_vc.choose_editor().unwrap();
            utils::edit_file(editor, &path).unwrap();
        }
    }

    fn get_sd_paths_from_file(&self, path: String) -> Vec<String> {
        let json_paths = utils::read_file_ignoring_comments(&path).unwrap();
        json_paths
    }

    fn get_sd_file_path(template: &String) -> String {
        let path = VariablesConfig::get().get_value("credentials_sd_directory");
        // remove the extension of template
        let template = utils::remove_file_extension(&template);
        let path = format!("{}/{}.txt", path, template);
        path
    }

    async fn select_dids(&self) -> Result<(IotaDocument, Did, IotaDocument, Did)> {
        let (issuer_document, issuer, holder_document, holder, ok) =
            self.create_vc.choose_dids().await?;

        if ok != ScreenEvent::Success {
            // Return an Error
            return Err(anyhow::anyhow!("Error selecting DIDs"));
        }

        self.create_vc.print_information_status(
            &issuer_document,
            &issuer.name().to_string(),
            &holder_document,
            &holder.name().to_string(),
        );
        println!("Creating VC with theses credentials");
        Output::cooldown().await;
        Ok((issuer_document, issuer, holder_document, holder))
    }
}
