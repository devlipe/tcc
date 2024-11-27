use crate::{AppContext, Command, Did, Output, ScreenEvent};
use prettytable::{row, Table};

pub struct ListDIDsCommand;

impl Command for ListDIDsCommand {
    fn execute(&mut self, context: &AppContext) -> ScreenEvent {
        self.print_tile();

        let dids = context.db.get_stored_dids();
        match dids {
            Ok(dids) => {
                self.display_dids_table(dids);
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

impl ListDIDsCommand {
    pub fn new() -> ListDIDsCommand {
        ListDIDsCommand
    }

    fn wait_for_user_input(&self) {
        println!("Press any key to return to the main menu");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
    }

    fn display_dids_table(&self, dids: Vec<Did>) {
        let mut table = Table::new();

        // Add a header row
        table.add_row(row!["Name", "Created", "DID",]);

        // Add rows for each DID, selecting only `id` and `name`
        for did in dids {
            table.add_row(row![did.name(), did.created_at(), did.did()]);
        }

        // Print the table to the terminal
        table.printstd();
    }
}
