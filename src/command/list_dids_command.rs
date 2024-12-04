use crate::{AppContext, Command, Did, Output, ScreenEvent};
use prettytable::{row, Table};

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

        self.wait_for_user_input();

        ScreenEvent::Success
    }

    fn print_tile(&self) {
        Output::clear_screen();
        Output::print_screen_title("List DIDs")
    }
}

impl ListDIDsCommand<'_> {
    pub fn new(context: & AppContext) -> ListDIDsCommand {
        ListDIDsCommand { context }
    }

    fn wait_for_user_input(&self) {
        println!("Press any key to return to the main menu");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
    }

    pub fn display_dids_table( dids: &Vec<Did>) {
        let mut table = Table::new();

        // Add a header row
        table.add_row(row!["Row", "Id", "Name", "Created", "DID",]);

        // Add rows for each DID, selecting only `id` and `name`
        let mut row_number = 1;
        for did in dids {
            table.add_row(row![
                row_number,
                did.id(),
                did.name(),
                did.created_at(),
                did.did()
            ]);
            row_number += 1;
        }

        // Print the table to the terminal
        table.printstd();
    }
}
