use crate::{
    AppContext, Command, Did, Input, ListDIDsCommand, ListVCsCommand, Output, ScreenEvent, Vc,
};
use anyhow::Result;
use crossterm::terminal::{Clear, ClearType};
use crossterm::{cursor, ExecutableCommand};
use identity_eddsa_verifier::EdDSAJwsVerifier;
use identity_iota::core::Timestamp;
use identity_iota::core::{Duration as IotaDuration, Object};
use identity_iota::credential::{
    DecodedJwtCredential, DecodedJwtPresentation, FailFast, Jwt, JwtCredentialValidator,
    JwtPresentationOptions, JwtPresentationValidationOptions, JwtPresentationValidator,
    JwtPresentationValidatorUtils, SubjectHolderRelationship,
};
use identity_iota::credential::{JwtCredentialValidatorUtils, Presentation};
use identity_iota::did::{CoreDID, DID};
use identity_iota::iota::IotaDocument;
use std::collections::HashMap;

use colored::Colorize;
use rand::Rng;
use std::io::{stdout, Write};
use std::thread::sleep;
use std::time::Duration;
use uuid::Uuid;

use identity_iota::credential::PresentationBuilder;
use identity_iota::document::verifiable::JwsVerificationOptions;
use identity_iota::storage::{JwkDocumentExt, JwsSignatureOptions};

use identity_iota::credential::JwtCredentialValidationOptions;

pub struct CreateVPCommand<'a> {
    context: &'a AppContext,
    verifier: Option<Did>,
    vc: Option<Vc>,
}

impl Command for CreateVPCommand<'_> {
    fn execute(&mut self) -> ScreenEvent {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.handle_vp_creation())
        })
        .unwrap_or_else(|e| {
            println!("Error: {}", e);
            Input::wait_for_user_input("Press enter to continue");
            ScreenEvent::Cancel
        })
    }

    fn print_tile(&self) {
        let mut title = "Create VP".bold().blue();

        if let Some(verifier) = &self.verifier {
            title = format!(
                "{} {} {}",
                title,
                "| Verifier:",
                verifier.name().bold().purple()
            )
            .into();
        }

        // If `holder` is present, append its name to the title
        if let Some(vc) = &self.vc {
            title = format!(
                "{} {} {}",
                title,
                "| Holder:",
                vc.holder().name().bold().purple()
            )
            .into();
            title = format!("{} {} {}", title, "| Type:", vc.tp().bold().purple()).into();
        }

        Output::clear_screen();
        println!("\n{}", title);
        // Print 2 blank lines
        println!("\n");
    }
}

impl CreateVPCommand<'_> {
    pub fn new(context: &AppContext) -> CreateVPCommand {
        CreateVPCommand {
            context,
            verifier: None,
            vc: None,
        }
    }

    async fn handle_vp_creation(&mut self) -> Result<ScreenEvent> {
        let (mut verifier_document, mut verifier_did) = self.choose_did().await?;
        self.confirm_verifier_selection(&mut verifier_did, &mut verifier_document)
            .await;
        self.verifier = Some(verifier_did);
        let mut vc = self.choose_vc()?;
        self.confirm_vc_selection(&mut vc).await;
        self.vc = Some(vc.clone());

        let (vp_jwt, challenge) = self.create_vp(&verifier_document, &vc).await?;

        self.verify_jwt_presentation(challenge, &vp_jwt).await?;

        Input::wait_for_user_input("Press enter to continue");

        Ok(ScreenEvent::Success)
    }

    async fn create_vp(&self, _verifier_document: &IotaDocument, vc: &Vc) -> Result<(Jwt, String)> {
        self.print_tile();
        let expires = self.define_expiration();
        let challenge = self.exchange_challenge();

        let vc_jwt = Jwt::from(vc.vc().to_string());

        print!("Holder is signing the VP...");
        let holder_document = vc
            .holder()
            .resolve_to_iota_document(&self.context.resolver)
            .await;

        let presentation: Presentation<Jwt> =
            PresentationBuilder::new(holder_document.id().to_url().into(), Default::default())
                .credential(vc_jwt)
                .build()?;
        // and include the requested challenge and expiry timestamp.
        let presentation_jwt: Jwt = holder_document
            .create_presentation_jwt(
                &presentation,
                &self.context.storage,
                &vc.holder().fragment(),
                &JwsSignatureOptions::default().nonce(challenge.to_owned()),
                &JwtPresentationOptions::default().expiration_date(expires),
            )
            .await?;
        println!("Ok!");

        print!("Sending presentation (as JWT) to the verifier...");

        println!("Ok!");

        Ok((presentation_jwt, challenge))
    }

    async fn verify_jwt_presentation(
        &self,
        challenge: String,
        presentation_jwt: &Jwt,
    ) -> Result<()> {
        // Resolve the holder's document.
        print!("Verifying the Holder of the VP...");
        let holder_did: CoreDID = JwtPresentationValidatorUtils::extract_holder(&presentation_jwt)?;
        let holder: IotaDocument = self.context.resolver.resolve(&holder_did).await?;
        println!("Ok!");

        print!("Verifying the VP Challenge and Expiration...");
        let presentation_verifier_options: JwsVerificationOptions =
            JwsVerificationOptions::default().nonce(challenge.to_owned());
        let presentation_validation_options = JwtPresentationValidationOptions::default()
            .presentation_verifier_options(presentation_verifier_options);
        let presentation: DecodedJwtPresentation<Jwt> =
            JwtPresentationValidator::with_signature_verifier(EdDSAJwsVerifier::default())
                .validate(&presentation_jwt, &holder, &presentation_validation_options)?;
        println!("Ok!");

        print!("Verifying the Issuer...");
        let jwt_credentials: &Vec<Jwt> = &presentation.presentation.verifiable_credential;
        let issuers: Vec<CoreDID> = jwt_credentials
            .iter()
            .map(JwtCredentialValidatorUtils::extract_issuer_from_jwt)
            .collect::<Result<Vec<CoreDID>, _>>()?;
        let issuers_documents: HashMap<CoreDID, IotaDocument> =
            self.context.resolver.resolve_multiple(&issuers).await?;
        println!("Ok!");

        print!("Verifying the credentials and the relationship (Holder<>Subject)...");
        let credential_validator: JwtCredentialValidator<EdDSAJwsVerifier> =
            JwtCredentialValidator::with_signature_verifier(EdDSAJwsVerifier::default());
        let validation_options: JwtCredentialValidationOptions =
            JwtCredentialValidationOptions::default().subject_holder_relationship(
                holder_did.to_url().into(),
                SubjectHolderRelationship::AlwaysSubject,
            );
        for (index, jwt_vc) in jwt_credentials.iter().enumerate() {
            // SAFETY: Indexing should be fine since we extracted the DID from each credential and resolved it.
            let issuer_document: &IotaDocument = &issuers_documents[&issuers[index]];

            let _decoded_credential: DecodedJwtCredential<Object> = credential_validator
                .validate::<_, Object>(
                    jwt_vc,
                    issuer_document,
                    &validation_options,
                    FailFast::FirstError,
                )?;
        }
        println!("Ok!");
        Ok(())
    }

    fn exchange_challenge(&self) -> String {
        println!("Exchanging challenge with verifier and Holder...");
        let challenge = self.generate_uuid4();
        challenge
    }

    fn define_expiration(&self) -> Timestamp {
        //Ask for the expiration time
        println!("Please enter the expiration time in minutes:");
        let expiration_time = Input::get_number_input(0, 60);
        let expiration = Timestamp::now_utc()
            .checked_add(IotaDuration::minutes(expiration_time as u32))
            .unwrap();
        println!(
            "{} {} {}",
            "Verifier and Hold have agreed upon",
            expiration_time.to_string().green(),
            "minutes expiration".green()
        );

        expiration
    }

    fn generate_uuid4(&self) -> String {
        let uuid = Uuid::new_v4().to_string();
        let mut rng = rand::thread_rng();
        let mut stdout = stdout();

        println!("UUID:\n");
        for i in 0..uuid.len() {
            for _ in 0..rand::thread_rng().gen_range(1..=10) {
                let mut display_string: Vec<char> = uuid.chars().collect();
                for j in i + 1..uuid.len() {
                    display_string[j] = rng.gen_range(b'a'..=b'z') as char;
                }
                // Clear the last 2 lines

                stdout.execute(cursor::MoveUp(1)).unwrap();
                stdout.execute(Clear(ClearType::CurrentLine)).unwrap();

                println!("{}", display_string.iter().collect::<String>());
                stdout.flush().unwrap();
                sleep(Duration::from_millis(40));
            }
        }

        // Display the final UUID
        stdout.execute(cursor::MoveUp(1)).unwrap();
        stdout.execute(Clear(ClearType::CurrentLine)).unwrap();

        // // Display the final UUID
        // stdout.execute(Clear(ClearType::All)).unwrap();
        // stdout.execute(cursor::MoveTo(0, 0)).unwrap();
        println!("{}", uuid);
        stdout.flush().unwrap();

        uuid
    }

    fn display_verifier_selection(&self, verifier_did: &Did) {
        println!("{}\n", "Selected verifier".yellow().bold());
        println!("Name: {}", verifier_did.name());
        println!("DID: {}", verifier_did.did());
    }

    fn display_vc_selection(&self, vc: &Vc) {
        println!("{}\n", "Selected VC".yellow().bold());
        println!("Holder: {}", vc.holder().name());
        println!("Issuer: {}", vc.issuer().name());
        println!("Type: {}", vc.tp());
        println!("JWT: {}", vc.vc());
        println!("Created: {}", vc.created_at());
        println!("Id: {}", vc.id());
    }

    async fn confirm_vc_selection(&self, vc: &mut Vc) {
        loop {
            self.display_vc_selection(&vc);
            println!(
                "{} {} {} {} ",
                "\nPress",
                "enter to continue".green().bold(),
                "or type",
                "'back' to open the selection ".red().bold()
            );
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();

            match input.trim() {
                "" => return,
                "back" => {
                    *vc = self.choose_vc().unwrap();
                }
                _ => continue,
            }
        }
    }

    async fn confirm_verifier_selection(
        &self,
        verifier_did: &mut Did,
        verifier_document: &mut IotaDocument,
    ) {
        loop {
            self.display_verifier_selection(&verifier_did);
            println!(
                "{} {} {} {} ",
                "\nPress",
                "enter to continue".green().bold(),
                "or type",
                "'back' to open the selection ".red().bold()
            );
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            match input.trim() {
                "" => return,
                "back" => {
                    (*verifier_document, *verifier_did) = self.choose_did().await.unwrap();
                }
                _ => continue,
            }
        }
    }

    async fn choose_did(&self) -> Result<(IotaDocument, Did)> {
        self.print_tile();
        let dids = self.context.db.get_stored_dids().unwrap_or_default();

        // Check if there are any DIDs stored
        if dids.is_empty() {
            println!(
                "{}",
                "No DIDs found. Please create a DID first.".red().bold()
            );
            return Err(anyhow::anyhow!("No DIDs found"));
        }

        Ok(self.get_verifier_did(&dids).await)
    }

    async fn get_verifier_did(&self, dids: &Vec<Did>) -> (IotaDocument, Did) {
        self.print_tile();
        let index = Output::display_with_pagination(dids, Self::choose_verifier_table, 15, true);
        let did: Did = dids.get(index - 1).unwrap().clone();
        (
            did.resolve_to_iota_document(&self.context.resolver).await,
            did,
        )
    }

    fn choose_verifier_table(dids: &Vec<Did>, first_row_index: usize) {
        ListDIDsCommand::display_dids_table(dids, first_row_index);
        println!("Select the DID row to use as the verifier:");
    }

    fn choose_vc(&self) -> Result<Vc> {
        self.print_tile();
        let vcs: Vec<Vc> = self.context.db.get_stored_vcs().unwrap_or_default();
        if vcs.is_empty() {
            println!("{}", "No VCs found. Please create one first.".red().bold());

            return Err(anyhow::anyhow!("No VCs found"));
        }
        let vc = self.get_vc(&vcs).unwrap_or_default();
        Ok(vc)
    }

    fn get_vc(&self, vcs: &Vec<Vc>) -> Result<Vc> {
        let index = Output::display_with_pagination(&vcs, Self::choose_vc_to_vp_table, 2, true);
        let vc = vcs
            .get(index - 1)
            .map_or(Err(anyhow::anyhow!("Invalid index")), |vc| Ok(vc.clone()));
        vc
    }

    fn choose_vc_to_vp_table(vcs: &Vec<Vc>, first_row_index: usize) {
        ListVCsCommand::display_vcs_table(vcs, first_row_index);
        println!("Choose a VC to create the VP by entering the row number:");
    }
}
