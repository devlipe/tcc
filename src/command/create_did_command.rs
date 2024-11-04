use crate::{Command, ScreenEvent};

pub struct CreateDIDCommand;

impl Command for CreateDIDCommand {
    fn execute(&mut self) -> ScreenEvent {
        println!("CreateDidCommand executed");
        ScreenEvent::Success

    }
}