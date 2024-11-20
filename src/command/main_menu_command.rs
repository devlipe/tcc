use crate::{AppContext, Command, Output, ScreenEvent};

pub struct MainMenuCommand;

impl Command for MainMenuCommand {
    fn execute(&mut self, context: &AppContext) -> ScreenEvent {
        self.print_tile();
        println!("Main menu command executed");
        ScreenEvent::Cancel

    }

    fn print_tile(&self) {
        Output::clear_screen();
        Output::print_screen_title("Main Menu")
    }
}
