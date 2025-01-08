use crate::{
    utils, AppContext, Command, Did, Input, ListDIDsCommand, ListVCsCommand, Output, ScreenEvent,
    Vc,
};
use anyhow::Result;
use colored::Colorize;
use crossterm::terminal::{Clear, ClearType};
use crossterm::{cursor, ExecutableCommand};
use identity_eddsa_verifier::EdDSAJwsVerifier;
use identity_iota::core::{Duration as IotaDuration, Object};
use identity_iota::core::{Timestamp, ToJson};
use identity_iota::credential::{
    DecodedJwtCredential, DecodedJwtPresentation, FailFast, Jws, Jwt, JwtCredentialValidator,
    JwtPresentationOptions, JwtPresentationValidationOptions, JwtPresentationValidator,
    JwtPresentationValidatorUtils, KeyBindingJWTValidationOptions, SdJwtCredentialValidator,
    SubjectHolderRelationship,
};
use identity_iota::credential::{JwtCredentialValidatorUtils, Presentation};
use identity_iota::did::{CoreDID, DID};
use identity_iota::iota::{IotaDID, IotaDocument};
use rand::Rng;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::{stdout, Write};
use std::thread::sleep;
use std::time::Duration;
use uuid::Uuid;

use identity_iota::credential::PresentationBuilder;
use identity_iota::document::verifiable::JwsVerificationOptions;
use identity_iota::storage::{JwkDocumentExt, JwsSignatureOptions};

use identity_iota::credential::JwtCredentialValidationOptions;
use sd_jwt_payload::{KeyBindingJwtClaims, SdJwt, SdObjectDecoder, Sha256Hasher};

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
                "{} | Holder: {} | Type: {} | SD: {}",
                title,
                vc.holder().name().bold().purple(),
                vc.tp().bold().purple(),
                vc.sd().to_string().bold().purple()
            )
            .into();
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
        let mut vc = self.choose_vc()?;
        self.confirm_vc_selection(&mut vc).await;
        self.vc = Some(vc.clone());

        if vc.sd() {
            self.handle_sd_vp(&vc).await?;
        } else {
            self.handle_normal_vp(&vc).await?;
        }

        Input::wait_for_user_input("Press enter to continue");

        Ok(ScreenEvent::Success)
    }

    async fn handle_sd_vp(&self, vc: &Vc) -> Result<()> {
        let (verifier_document, _) = self.choose_did().await?;

        let (sd_jwt, nonce) = self.create_vp_sd(&vc, &verifier_document).await?;
        self.verify_sd_jwt_presentation(&sd_jwt, &verifier_document, nonce.as_str())
            .await?;

        Ok(())
    }

    async fn create_vp_sd(
        &self,
        vc: &Vc,
        verifier_document: &IotaDocument,
    ) -> Result<(String, String)> {
        self.print_tile();

        let sd_jwt = SdJwt::parse(vc.vc())?;
        let disclosures: Vec<String> = self.handle_disclosures_selection(&sd_jwt.disclosures);
        let nonce = self.exchange_challenge();

        // // Print the disclosures from sd_jwt after decoding them
        // let disclosures = sd_jwt
        //     .disclosures
        //     .clone()
        //     .iter()
        //     .map(|disclosure| utils::decode_base64(disclosure).unwrap())
        //     .collect::<Vec<String>>();
        //
        // Output::print_options_vec_generic(&disclosures);
        //
        // println!("Selected Disclosures:");
        // Output::print_options_vec_generic(&_disclosures);

        print!("Holder is creating the KB-JWT...");
        // Optionally, the holder can add a Key Binding JWT (KB-JWT). This is dependent on the verifier's policy.
        // Issuing the KB-JWT is done by creating the claims set and setting the header `typ` value
        // with the help of `KeyBindingJwtClaims`.
        let binding_claims = KeyBindingJwtClaims::new(
            &Sha256Hasher::new(),
            sd_jwt.jwt.as_str().to_string(),
            disclosures.clone(),
            nonce.to_string(),
            verifier_document.id().to_string(),
            Timestamp::now_utc().to_unix(),
        )
        .to_json()?;
        println!("Ok!");

        print!("Holder is signing the JWT...");

        // Setting the `typ` in the header is required.
        let options = JwsSignatureOptions::new().typ(KeyBindingJwtClaims::KB_JWT_HEADER_TYP);
        let holder_document = vc
            .holder()
            .resolve_to_iota_document(&self.context.resolver)
            .await;
        // Create the KB-JWT.
        let kb_jwt: Jws = holder_document
            .create_jws(
                &self.context.storage,
                &vc.holder().fragment(),
                binding_claims.as_bytes(),
                &options,
            )
            .await?;
        // Create the final SD-JWT.
        let sd_jwt_obj = SdJwt::new(sd_jwt.jwt, disclosures, Some(kb_jwt.into()));
        println!("Ok!");

        print!("Sending presentation (as JWT) to the verifier...");
        let sd_jwt_presentation: String = sd_jwt_obj.presentation();
        println!("Ok!");

        Ok((sd_jwt_presentation, nonce.clone()))
    }

    async fn verify_sd_jwt_presentation(
        &self,
        sd_jwt_presentation: &String,
        verifier_document: &IotaDocument,
        nonce: &str,
    ) -> Result<()> {
        // //Print the SD-JWT-Presentation, The nonce, and the verifier's DID
        // println!("SD-JWT-Presentation: {}", sd_jwt_presentation);
        // println!("Nonce: {}", nonce);
        // println!("Verifier's DID: {}", verifier_document.id());

        print!("Verifier is parsing the JWT...");
        let sd_jwt = SdJwt::parse(&sd_jwt_presentation)?;
        let (issuer_document, holder_document) = self.get_issuer_and_holder(&sd_jwt.jwt).await?;
        println!("Ok!");

        print!("Verifier is validating the JWT...");
        let decoder = SdObjectDecoder::new_with_sha256();
        let validator =
            SdJwtCredentialValidator::with_signature_verifier(EdDSAJwsVerifier::default(), decoder);
        let validation = validator.validate_credential::<_, Object>(
            &sd_jwt,
            &issuer_document,
            &JwtCredentialValidationOptions::default(),
            FailFast::FirstError,
        )?;
        println!("Ok!");

        print!("Verifier is validating the KB-JWT...");
        let options = KeyBindingJWTValidationOptions::new()
            .nonce(nonce)
            .aud(&verifier_document.id().to_string());
        let _kb_validation =
            validator.validate_key_binding_jwt(&sd_jwt, &holder_document, &options)?;
        println!("Ok!");

        println!("JWT successfully validated");
        utils::pretty_print_json("Decoded Credential", &validation.credential.to_string());

        Ok(())
    }

    async fn get_issuer_and_holder(&self, jwt: &String) -> Result<(IotaDocument, IotaDocument)> {
        let (issuer, holder) = utils::get_entities_from_jwt(jwt)?;
        let issuer_document = self
            .context
            .resolver
            .resolve(&IotaDID::parse(&issuer)?)
            .await?;
        let holder_document = self
            .context
            .resolver
            .resolve(&IotaDID::parse(&holder)?)
            .await?;

        Ok((issuer_document, holder_document))
    }

    pub fn handle_disclosures_selection(&self, disclosures: &Vec<String>) -> Vec<String> {
        let mut selected_disclosures: HashSet<usize> = HashSet::new();
        let mut error: String = String::new();
        let disclosures_options =
            utils::extract_disclosure_keys(disclosures).unwrap_or(disclosures.clone()); // If the disclosures are not base64 encoded, use the original disclosures

        loop {
            self.print_tile();
            self.print_disclosures(&disclosures_options, &selected_disclosures);

            if !error.is_empty() {
                println!("{}", error.red());
            }

            let user_input = Input::wait_for_user_input(
                "Type the number to toggle a disclosure, or 'ok' to proceed:",
            );

            match self.handle_user_input(
                user_input.trim(),
                &disclosures_options,
                &mut selected_disclosures,
            ) {
                Ok(_) => error.clear(),
                Err(e) => error = e,
            }

            if user_input.trim() == "ok" {
                return selected_disclosures
                    .iter()
                    .filter_map(|&index| disclosures.get(index).cloned())
                    .collect();
            }
        }
    }

    fn print_disclosures<T: ToString>(
        &self,
        disclosures: &Vec<T>,
        selected_disclosures: &HashSet<usize>,
    ) {
        println!("Available disclosures:");
        Output::print_options_vec_generic(disclosures);

        println!("\nSelected disclosures:");
        if selected_disclosures.is_empty() {
            println!("None");
        } else {
            for &index in selected_disclosures {
                if let Some(disclosure) = disclosures.get(index) {
                    println!("{}: {}", index + 1, disclosure.to_string());
                }
            }
        }
    }

    fn handle_user_input<T: ToString>(
        &self,
        input: &str,
        disclosures: &[T],
        selected_disclosures: &mut HashSet<usize>,
    ) -> Result<(), String> {
        match input {
            "ok" => Ok(()),
            input => {
                if let Ok(num) = input.parse::<usize>() {
                    if num > 0 && num <= disclosures.len() {
                        let index = num - 1;
                        if selected_disclosures.contains(&index) {
                            selected_disclosures.remove(&index);
                        } else {
                            selected_disclosures.insert(index);
                        }
                        Ok(())
                    } else {
                        Err(String::from(
                            "Invalid number. Please select a valid option.",
                        ))
                    }
                } else {
                    Err(String::from(
                        "Invalid input. Please enter a number or 'ok'.",
                    ))
                }
            }
        }
    }

    async fn handle_normal_vp(&self, vc: &Vc) -> Result<()> {
        let (vp_jwt, challenge) = self.create_vp_normal(&vc).await?;
        self.verify_jwt_presentation_normal(challenge, &vp_jwt)
            .await?;

        Ok(())
    }

    async fn create_vp_normal(&self, vc: &Vc) -> Result<(Jwt, String)> {
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

    async fn verify_jwt_presentation_normal(
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

    fn _display_verifier_selection(&self, verifier_did: &Did) {
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

    async fn _confirm_verifier_selection(
        &self,
        verifier_did: &mut Did,
        verifier_document: &mut IotaDocument,
    ) {
        loop {
            self._display_verifier_selection(&verifier_did);
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
