use crate::{Command, Input, Output, ScreenEvent};

pub struct CreateVCMenu{
    options: Vec<(String, ScreenEvent)>,
}

impl Command for CreateVCMenu {
    fn execute(&mut self) -> ScreenEvent {
        self.print_tile();
        Output::print_options_vec(&self.options);
        println!("\nPlease select an option:");
        let user_input = Input::get_number_input(1, self.options.len());
        self.options[user_input - 1].1.to_owned()
    }

    fn print_tile(&self) {
        Output::clear_screen();
        Output::print_screen_title("Create Verifiable Credential")
    }
}

impl CreateVCMenu {
    
    pub fn new() -> CreateVCMenu {
        let mut options = Vec::new();
        options.push(("Create Verifiable Credential".to_string(), ScreenEvent::CreateNormalVC));
        options.push(("Create Verifiable Credential with Selective Disclosure".to_string(), ScreenEvent::CreateSDVC));
        options.push(("Back".to_string(), ScreenEvent::Cancel));
        CreateVCMenu { options }
    }
    
}