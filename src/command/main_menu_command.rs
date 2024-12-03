use crate::{Command, Input, Output, ScreenEvent};

pub struct MainMenuCommand {
    // a vector of options that the user can select (String, ScreenEvent)
    options: Vec<(String, ScreenEvent)>,
}

impl Command for MainMenuCommand {
    fn execute(&mut self) -> ScreenEvent {
        self.print_tile();
        self.print_options();
        println!("\nPlease select an option:");

        let user_input = Input::get_number_input(1, self.options.len());
        println!("User input: {}", user_input);
        self.handle_user_input(user_input)
    }

    fn print_tile(&self) {
        Output::clear_screen();
        Output::print_screen_title("Main Menu")
    }
}

impl MainMenuCommand {
    pub fn new() -> MainMenuCommand {
        let mut options = Vec::new();
        options.push(("Create a new DID".to_string(), ScreenEvent::SelectCreateDID));
        options.push(("List created DIDs".to_string(), ScreenEvent::SelectListDIDs));
        options.push(("Create a new VC".to_string(), ScreenEvent::SelectCreateVC));
        options.push(("Exit".to_string(), ScreenEvent::Cancel));

        MainMenuCommand { options }
    }

    fn print_options(&self) {
        for (index, option) in self.options.iter().enumerate() {
            println!("{}. {}", index + 1, option.0);
        }
    }

    fn handle_user_input(&self, input: usize) -> ScreenEvent {
        let option = self.options.get(input - 1).unwrap();
        option.1.clone()
    }
}
