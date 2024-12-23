use crate::{AppContext, Command, Did, Input, Output, ScreenEvent};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::Table;

pub struct ListDIDsCommand<'a> {
    context: &'a AppContext,
}

impl Command for ListDIDsCommand<'_> {
    fn execute(&mut self) -> ScreenEvent {
        self.print_tile();

        let dids = self.context.db.get_stored_dids();
        match dids {
            Ok(dids) => {
                ListDIDsCommand::display_dids_table(&dids);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }

        Input::wait_for_user_input("Press any key to return to the main menu");

        ScreenEvent::Success
    }

    fn print_tile(&self) {
        Output::clear_screen();
        Output::print_screen_title("List DIDs")
    }
}

impl ListDIDsCommand<'_> {
    pub fn new(context: &AppContext) -> ListDIDsCommand {
        ListDIDsCommand { context }
    }

    pub fn display_dids_table(dids: &Vec<Did>) {
        let mut table = Table::new();
        table.apply_modifier(UTF8_ROUND_CORNERS);
        table.set_content_arrangement(comfy_table::ContentArrangement::Dynamic);

        // Add a header row
        table.set_header(vec!["Row", "Name", "Created", "DID", "Id"]);

        // Add rows for each DID, selecting only `id` and `name`
        let mut row_number = 1;
        for did in dids {
            table.add_row(vec![
                row_number.to_string(),
                did.name().to_string(),
                did.created_at().to_string(),
                did.did().to_string(),
                did.id().to_string(),
            ]);

            row_number += 1;
        }

        // Print the table to the terminal
        println!("{table}");
    }
}
