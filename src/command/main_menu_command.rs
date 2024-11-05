use crate::{AppContext, Command, ScreenEvent};

pub struct MainMenuCommand;

impl Command for MainMenuCommand {
    fn execute(&mut self, context: &AppContext) -> ScreenEvent {
        println!("Main menu command executed");
        ScreenEvent::SelectCreateDID

    }
}