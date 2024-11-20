use crate::{AppContext, Command, Output, ScreenEvent};

pub struct CreateVCCommand;

impl Command for CreateVCCommand {
    fn execute(&mut self, context: &AppContext) -> ScreenEvent {
        println!("CreateVCCommand executed");
        ScreenEvent::Success

    }

    fn print_tile(&self) {
        Output::clear_screen();
        Output::print_screen_title("Create VC")
    }
}
