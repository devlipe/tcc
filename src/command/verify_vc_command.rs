use crate::{Command, ScreenEvent};

pub struct VerifyVCCommand;

impl Command for VerifyVCCommand {
    fn execute(&mut self) -> ScreenEvent {
        println!("VerifyVCCommand executed");
        ScreenEvent::Success
    }
}