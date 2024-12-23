use crate::{AppContext, Command, Input, Output, ScreenEvent, Vc};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::Table;

pub struct ListVCsCommand<'a> {
    context: &'a AppContext,
}

impl Command for ListVCsCommand<'_> {
    fn execute(&mut self) -> ScreenEvent {
        self.print_tile();

        let vcs = self.context.db.get_stored_vcs();
        match vcs {
            Ok(vcs) => {
                ListVCsCommand::display_vcs_table(&vcs);
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

impl ListVCsCommand<'_> {
    pub fn new(context: &AppContext) -> ListVCsCommand {
        ListVCsCommand { context }
    }

    pub fn display_vcs_table(vcs: &Vec<Vc>) {
        let mut table = Table::new();
        table.apply_modifier(UTF8_ROUND_CORNERS);
        table.set_content_arrangement(comfy_table::ContentArrangement::Dynamic);
        // Add header row
        table.set_header(vec![
            "Row", "Holder", "Issuer", "Type", "JWT", "Created", "Id",
        ]);

        // Add rows for each DID
        let mut row_number = 1;
        for vc in vcs {
            table.add_row(vec![
                row_number.to_string(),
                vc.holder().name().to_string(),
                vc.issuer().name().to_string(),
                vc.tp().to_string(),
                vc.vc().to_string(),
                vc.created_at().to_string(),
                vc.id().to_string(),
            ]);
            row_number += 1;
        }

        println!("{table}");
    }
}
