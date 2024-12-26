use crate::{AppContext, Command, Did, Input, ListDIDsCommand, ListVCsCommand, Output, ScreenEvent, Vc};
use anyhow::Result;
use colored::*;
use identity_eddsa_verifier::EdDSAJwsVerifier;
use identity_iota::core::Object;
use identity_iota::credential::{
    DecodedJwtCredential, FailFast, Jwt, JwtCredentialValidationOptions, JwtCredentialValidator,
};
use identity_iota::iota::IotaDocument;

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

    pub async fn handle_verify_vc(&self) -> anyhow::Result<ScreenEvent> {
        let vc: Vc = self.choose_vc()?;

        // Print the VC to be verified
        println!("Verifying the following VC:");
        println!("{:?}", vc);

        let issuer_document = self.choose_did_document().await?;
        
        let jwt_token = Jwt::from(vc.vc().to_string());
        
        let decoded_vc = Self::verify_credential(&jwt_token, &issuer_document);
        
        match decoded_vc {
            Ok(decoded_vc) => {
                println!("{}","VC verified successfully:".green().bold());
                println!("{:?}", decoded_vc);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }

        Input::wait_for_user_input("Press any key to continue...");
        Ok(ScreenEvent::Success)
    }

    pub fn verify_credential(
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
        println!("Choose a DID to verify the credential by entering the row number:");
    }
}
