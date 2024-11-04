use crate::{Command, ScreenEvent};

pub struct CreateVCCommand;

impl Command for CreateVCCommand {
    fn execute(&mut self) -> ScreenEvent {
        println!("CreateVCCommand executed");
        ScreenEvent::Success

    }
}
