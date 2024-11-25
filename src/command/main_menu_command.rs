use crate::{AppContext, Command, Output, ScreenEvent};

pub struct MainMenuCommand;

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
    
    fn print_options(&self) {
        println!("1. Create a new DID");
        println!("2. Create a new VC");
        println!("3. Exit");
    }
    
    fn get_user_input(&self) -> String {
        println!("\nPlease select an option:");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        input.trim().to_string()
    }
    
    fn handle_user_input(&self, input: &str) -> ScreenEvent {
        match input {
            "1" => ScreenEvent::SelectCreateDID,
            "2" => ScreenEvent::SelectCreateVC,
            "3" => ScreenEvent::Cancel,
            _ => ScreenEvent::Success
        }
    }
    
}
