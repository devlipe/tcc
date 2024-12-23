use crate::{AppContext, Command, Did, Output, ScreenEvent};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::{Cell, Table};

pub struct ListDIDsCommand<'a> {
    context: &'a AppContext,
}

impl Command for ListDIDsCommand<'_> {
    fn execute(&mut self) -> ScreenEvent {
        self.print_tile();

        let dids = self.context.db.get_stored_dids();
        match dids {
            Ok(dids) => {
                Output::display_with_pagination(&dids, Self::display_dids_table, 15, false);
                ScreenEvent::Success
            }
            Err(e) => {
                println!("Error: {}", e);
                ScreenEvent::Cancel
            }
        }
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

    pub fn display_dids_table(dids: &Vec<Did>, first_row_index: usize) {
        let mut table = Table::new();
        table.apply_modifier(UTF8_ROUND_CORNERS);
        table.set_content_arrangement(comfy_table::ContentArrangement::Dynamic);

        // Add a header row
        table.set_header(vec!["Row", "Name", "Created", "DID", "Id"]);

        // Add rows for each DID, selecting only `id` and `name`
        let mut row_number = first_row_index;
        for did in dids {
            table.add_row(vec![
                Cell::new(row_number),
                Cell::new(did.name()),
                Cell::new(did.created_at()),
                Cell::new(did.did()),
                Cell::new(did.id()),
            ]);

            row_number += 1;
        }

        // Print the table to the terminal
        println!("{table}");
    }
}
