use crate::{Command, Input, Output, ScreenEvent};

pub struct MainMenuCommand {
    // a vector of options that the user can select (String, ScreenEvent)
    options: Vec<(String, ScreenEvent)>,
}

impl Command for MainMenuCommand {
    fn execute(&mut self) -> ScreenEvent {
        self.print_tile();
        Output::print_options_vec(&self.options);
        println!("\nPlease select an option:");

        let user_input = Input::get_number_input(1, self.options.len());
        println!("User input: {}", user_input);
        self.options[user_input - 1].1.clone()
    }

    fn print_tile(&self) {
        Output::clear_screen();
        Output::print_screen_title("Main Menu")
    }
}

impl MainMenuCommand {
    pub fn new() -> MainMenuCommand {
        let options = vec![
            ("List Created Items".to_string(), ScreenEvent::ListItems),
            ("Create a new DID".to_string(), ScreenEvent::CreateDID),
            ("Create a new VC".to_string(), ScreenEvent::CreateVC),
            ("Create a new VP".to_string(), ScreenEvent::CreateVP),
            ("Verify a VC".to_string(), ScreenEvent::VerifyVC),
            ("Exit".to_string(), ScreenEvent::Cancel),
        ];

        MainMenuCommand { options }
    }
}
