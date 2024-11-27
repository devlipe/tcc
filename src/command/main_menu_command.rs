use crate::{AppContext, Command, Output, ScreenEvent};

pub struct MainMenuCommand {
    // a vector of options that the user can select (String, ScreenEvent)
    options: Vec<(String, ScreenEvent)>,
}

impl Command for MainMenuCommand {
    fn execute(&mut self, _context: &AppContext) -> ScreenEvent {
        self.print_tile();
        self.print_options();
        let user_input = self.get_user_input();
        println!("User input: {}", user_input);
        self.handle_user_input(&user_input)
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

    fn get_user_input(&self) -> String {
        loop {
            println!("\nPlease select an option:");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();

            let trimmed_input = input.trim();

            // Check if input is a blank line
            if trimmed_input.is_empty() {
                println!("Input cannot be blank. Please try again.");
                continue;
            }
            // Check if input is a number
            if !trimmed_input.chars().all(char::is_numeric) {
                println!("Invalid input. Please enter a number.");
                continue;
            }

            // Check if the input is within the valid range
            if let Ok(selection) = trimmed_input.parse::<usize>() {
                if selection > 0 && selection <= self.options.len() {
                    return trimmed_input.to_string();
                }
            }

            println!(
                "Invalid input. Please enter a number between 1 and {}.",
                self.options.len()
            );
        }
    }

    fn handle_user_input(&self, input: &str) -> ScreenEvent {
        let index = input.parse::<usize>().unwrap();
        let option = self.options.get(index - 1).unwrap();
        option.1.clone()
    }
}
