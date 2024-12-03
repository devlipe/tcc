use crate::{AppContext, Command, Did, Input, ListDIDsCommand, Output, ScreenEvent};
use identity_iota::iota::IotaDocument;

pub struct CreateVCCommand {
    context: &'static AppContext,
}

impl Command for CreateVCCommand {
    fn execute(&mut self) -> ScreenEvent {
        self.print_tile();
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

impl CreateVCCommand {
    pub fn new(context: &'static AppContext) -> CreateVCCommand {
        CreateVCCommand { context }
    }

    async fn handle_vc_creation(&self) -> anyhow::Result<ScreenEvent> {
        let _issuer_did: IotaDocument = self.get_issuer_did().await;

        Ok(ScreenEvent::Success)
    }

    pub async fn get_issuer_did(&self) -> IotaDocument {
        let did: Did = self.get_did();
        did.resolve_to_iota_document(&self.context.resolver).await
    }

    pub fn get_did(&self) -> Did {
        let dids = self.context.db.get_stored_dids().unwrap();

        ListDIDsCommand::display_dids_table(&dids);

        println!("Select the DID row to use as the issuer:");
        let user_input = Input::get_number_input(1, dids.len());
        let selected_did = dids.get(user_input - 1).unwrap();
        selected_did.clone()
    }
}
