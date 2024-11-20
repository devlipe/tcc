use crate::{AppContext, Command, Output, ScreenEvent};

pub struct CreateDIDCommand;

impl Command for CreateDIDCommand {
    fn execute(&mut self, context: &AppContext) -> ScreenEvent {
        println!("CreateDidCommand executed");
        ScreenEvent::Success

    }

    fn print_tile(&self) {
        Output::clear_screen();
        Output::print_screen_title("Create DID")
    }
}