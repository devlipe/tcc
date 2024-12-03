use crate::{Command, Output, ScreenEvent};

pub struct VerifyVCCommand;

impl Command for VerifyVCCommand {
    fn execute(&mut self) -> ScreenEvent {
        println!("VerifyVCCommand executed");
        ScreenEvent::Success
    }

    fn print_tile(&self) {
        Output::clear_screen();
        Output::print_screen_title("Verify VC")
    }
}