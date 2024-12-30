use crate::{utils, AppContext, Command, CreateVCNormalCommand, Did, Output, ScreenEvent};
use anyhow::Result;
use identity_iota::core::{FromJson, Url};
use identity_iota::credential::{Credential, CredentialBuilder, Subject};
use identity_iota::did::DID;
use identity_iota::iota::IotaDocument;
use sd_jwt_payload::SdObjectEncoder;
use serde_json::Value;


pub struct CreateVCSDCommand<'a> {
    context: &'a AppContext,
    create_vc: CreateVCNormalCommand<'a>,
}

impl<'a> Command for CreateVCSDCommand<'a> {
    fn execute(&mut self) -> ScreenEvent {
        self.create_vc.execute();
        ScreenEvent::Success
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

        let subject: Subject = Subject::from_json_value(json)?;

        let credential: Credential = CredentialBuilder::default()
            .type_(&credential_type)
            .issuer(Url::parse(issuer_document.id().as_str())?)
            .non_transferable(true)
            .subject(subject)
            .build()?;

        let payload = credential.serialize_jwt(None)?;


        Ok(ScreenEvent::Success)
    }

    fn add_disclousures(&self, payload : &String, template : &String) -> Result<()> {
        let mut encoder = SdObjectEncoder::new(&payload)?;
        
        // 
        
        Ok(())
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
