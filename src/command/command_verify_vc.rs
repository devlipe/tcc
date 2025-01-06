use crate::{
    AppContext, Command, Did, Input, ListDIDsCommand, ListVCsCommand, Output, ScreenEvent,
    Vc,
};
use anyhow::Result;
use colored::Colorize;
use identity_eddsa_verifier::EdDSAJwsVerifier;
use identity_iota::core::Object;
use identity_iota::credential::SdJwtCredentialValidator;
use identity_iota::credential::{
    DecodedJwtCredential, FailFast, Jwt, JwtCredentialValidationOptions, JwtCredentialValidator,
};
use identity_iota::iota::IotaDocument;
use sd_jwt_payload::{SdJwt, SdObjectDecoder};

pub struct VerifyVCCommand<'a> {
    context: &'a AppContext,
}

impl Command for VerifyVCCommand<'_> {
    fn execute(&mut self) -> ScreenEvent {
        // Block on the async function using block_in_place
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.handle_verify_vc())
        })
        .unwrap_or_else(|e| {
            println!("Error: {}", e);
            ScreenEvent::Cancel
        })
    }

    fn print_tile(&self) {
        Output::clear_screen();
        Output::print_screen_title("Verify VC")
    }
}

impl VerifyVCCommand<'_> {
    pub fn new(context: &AppContext) -> VerifyVCCommand {
        VerifyVCCommand { context }
    }

    pub async fn handle_verify_vc(&self) -> Result<ScreenEvent> {
        let vc: Vc = self.choose_vc()?;

        // if vc.sd() {
        //     println!(
        //         "{}",
        //         "You choose a Selective Disclosure VC.".yellow().bold()
        //     );
        //     println!(
        //         "{}",
        //         "Please use the Create VP option to verify this Selective Disclosure VC."
        //             .yellow()
        //             .bold()
        //     );
        //     Input::wait_for_user_input("Press any key to continue...");
        //     return Ok(ScreenEvent::Success);
        // }

        // Print the VC to be verified
        println!("Verifying the following VC:");
        println!("{:?}", vc);

        let issuer_document = self.choose_did_document().await?;

        let decoded_vc = Self::verify_credential(&vc, &issuer_document);

        match decoded_vc {
            Ok(decoded_vc) => {
                println!("{}", "VC verified successfully:".green().bold());
                println!("{:?}", decoded_vc);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }

        Input::wait_for_user_input("Press any key to continue...");
        Ok(ScreenEvent::Success)
    }

    fn verify_credential(vc: &Vc, issuer_document: &IotaDocument) -> Result<DecodedJwtCredential> {
        let decoded_vc: DecodedJwtCredential<Object>;
        if vc.sd() {
            decoded_vc = Self::verify_sd_vc(vc, &issuer_document)?;
        } else {
            let credential_jwt = Jwt::from(vc.vc().to_string());
            decoded_vc = Self::verify_normal_vc(&credential_jwt, &issuer_document)?;
        }

        Ok(decoded_vc)
    }

    fn verify_sd_vc(vc: &Vc, issuer_document: &&IotaDocument) -> Result<DecodedJwtCredential> {
        let sd_jwt = SdJwt::parse(&vc.vc())?;
        let decoder = SdObjectDecoder::new_with_sha256();
        let validator =
            SdJwtCredentialValidator::with_signature_verifier(EdDSAJwsVerifier::default(), decoder);
        let validation = validator.validate_credential::<_, Object>(
            &sd_jwt,
            &issuer_document,
            &JwtCredentialValidationOptions::default(),
            FailFast::FirstError,
        )?;
        Ok(validation)
    }

    pub fn verify_normal_vc(
        credential_jwt: &Jwt,
        issuer_document: &IotaDocument,
    ) -> Result<DecodedJwtCredential> {
        let decoded_vc: DecodedJwtCredential<Object> =
            JwtCredentialValidator::with_signature_verifier(EdDSAJwsVerifier::default())
                .validate::<_, Object>(
                    &credential_jwt,
                    &issuer_document,
                    &JwtCredentialValidationOptions::default(),
                    FailFast::FirstError,
                )?;
        Ok(decoded_vc)
    }

    pub fn choose_vc(&self) -> Result<Vc> {
        let vcs: Vec<Vc> = self.context.db.get_stored_vcs()?;

        if vcs.is_empty() {
            println!("{}", "No VCs found. Please create a VC first.".red().bold());
            Input::wait_for_user_input("Press enter to continue");
            return Err(anyhow::anyhow!("No VCs found"));
        }
        let vc = self.get_vc(&vcs)?;
        Ok(vc)
    }

    fn get_vc(&self, vcs: &Vec<Vc>) -> Result<Vc> {
        self.print_tile();
        let index = Output::display_with_pagination(&vcs, Self::choose_vc_table, 2, true);
        let vc = vcs.get(index - 1);
        match vc {
            Some(vc) => Ok(vc.clone()),
            None => Err(anyhow::anyhow!("Invalid index")),
        }
    }

    fn choose_vc_table(vcs: &Vec<Vc>, first_row_index: usize) {
        ListVCsCommand::display_vcs_table(vcs, first_row_index);
        println!("Choose a VC to be verified by entering the row number:");
    }

    async fn choose_did_document(&self) -> Result<IotaDocument> {
        let dids: Vec<Did> = self.context.db.get_stored_dids()?;
        let did = self.get_did_document(&dids).await?;
        Ok(did)
    }

    async fn get_did_document(&self, dids: &Vec<Did>) -> Result<IotaDocument> {
        self.print_tile();
        let index = Output::display_with_pagination(&dids, Self::choose_did_table, 15, true);
        let did = dids.get(index - 1);
        match did {
            Some(did) => Ok(did.resolve_to_iota_document(&self.context.resolver).await),
            None => Err(anyhow::anyhow!("Invalid index")),
        }
    }

    fn choose_did_table(dids: &Vec<Did>, first_row_index: usize) {
        ListDIDsCommand::display_dids_table(dids, first_row_index);
        println!(
            "Choose a DID to verify as the issuer of the credential by entering the row number:"
        );
    }
}
