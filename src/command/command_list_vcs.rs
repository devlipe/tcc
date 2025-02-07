use crate::{AppContext, Command, Output, ScreenEvent, VariablesConfig, Vc};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::{Cell, Table};

pub struct ListVCsCommand<'a> {
    context: &'a AppContext,
}

impl Command for ListVCsCommand<'_> {
    fn execute(&mut self) -> ScreenEvent {

        let vcs = self.context.db.get_stored_vcs();
        match vcs {
            Ok(vcs) => {
                Output::display_with_pagination(
                    &vcs,
                    Self::display_vcs_table,
                    VariablesConfig::get().vc_table_size(),
                    false,
                    Some(Box::new(|| self.print_tile())),
                );
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

impl ListVCsCommand<'_> {
    pub fn new(context: &AppContext) -> ListVCsCommand {
        ListVCsCommand { context }
    }

    pub fn display_vcs_table(vcs: &Vec<Vc>, fist_row_index: usize) {
        let mut table = Table::new();
        table.apply_modifier(UTF8_ROUND_CORNERS);
        table.set_content_arrangement(comfy_table::ContentArrangement::Dynamic);
        // Add header row
        table.set_header(vec![
            "Row", "Holder", "Issuer", "Type", "SD", "JWT", "Created", "Id",
        ]);

        // Add rows for each DID
        let mut row_number = fist_row_index;
        for vc in vcs {
            let vc_jwt = vc.vc();

            // Extract the first 10 and last 10 characters
            let short_vc_text = format!(
                "{} [.../{}] {}",
                &vc_jwt[..100],                // First 10 characters
                vc_jwt.len() - 200,            // Number of characters omitted
                &vc_jwt[vc_jwt.len() - 100..]  // Last 10 characters
            );
            table.add_row(vec![
                Cell::new(row_number),
                Cell::new(vc.holder().name()),
                Cell::new(vc.issuer().name()),
                Cell::new(vc.tp()),
                Cell::new(vc.sd().to_string()),
                Cell::new(short_vc_text),
                Cell::new(vc.created_at()),
                Cell::new(vc.id()),
            ]);
            row_number += 1;
        }

        println!("{table}");
    }
}
