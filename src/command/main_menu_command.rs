use crate::{Command, ScreenEvent};

pub struct MainMenuCommand;

impl Command for MainMenuCommand {
    fn execute(&mut self) -> ScreenEvent {
        println!("Main menu command executed");
        ScreenEvent::SelectCreateDID

    }
}