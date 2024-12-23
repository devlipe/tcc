use crate::{Command, Output, ScreenEvent};
use identity_eddsa_verifier::EdDSAJwsVerifier;
use identity_iota::core::Object;
use identity_iota::credential::{DecodedJwtCredential, FailFast, Jwt, JwtCredentialValidationOptions, JwtCredentialValidator};
use identity_iota::iota::IotaDocument;

pub struct VerifyVCCommand;

impl Command for VerifyVCCommand {
    fn execute(&mut self) -> ScreenEvent {
        println!("VerifyVCCommand executed");
        ScreenEvent::Success
    }

    fn print_tile(&self) {
        Output::clear_screen();
        Output::print_screen_title("Verify VC")
    }
}

impl VerifyVCCommand {
    pub fn verify_credential(credential_jwt: &Jwt, issuer_document : &IotaDocument) -> anyhow::Result<DecodedJwtCredential> {
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
}
