use crate::{AppContext, Command, ScreenEvent};

pub struct CreateDIDCommand;

impl Command for CreateDIDCommand {
    fn execute(&mut self, context: &AppContext) -> ScreenEvent {
        println!("CreateDidCommand executed");
        ScreenEvent::Success

    }
}