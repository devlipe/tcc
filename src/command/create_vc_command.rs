use crate::{AppContext, Command, ScreenEvent};

pub struct CreateVCCommand;

impl Command for CreateVCCommand {
    fn execute(&mut self, context: &AppContext) -> ScreenEvent {
        println!("CreateVCCommand executed");
        ScreenEvent::Success

    }
}
