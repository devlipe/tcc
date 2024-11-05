use crate::{AppContext, Command, ScreenEvent};

pub struct VerifyVCCommand;

impl Command for VerifyVCCommand {
    fn execute(&mut self, context: &AppContext) -> ScreenEvent {
        println!("VerifyVCCommand executed");
        ScreenEvent::Success
    }
}