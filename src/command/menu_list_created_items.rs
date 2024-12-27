use crate::{Command, Input, Output, ScreenEvent};

pub struct ListCreatedItems {
    // a vector of options that the user can select (String, ScreenEvent)
    options: Vec<(String, ScreenEvent)>,
}

impl Command for ListCreatedItems {
    fn execute(&mut self) -> ScreenEvent {
        self.print_tile();
        Output::print_options_vec(&self.options);
        println!("\nPlease select an option:");

        let user_input = Input::get_number_input(1, self.options.len());
        self.options[user_input - 1].1.clone()
    }

    fn print_tile(&self) {
        Output::clear_screen();
        Output::print_screen_title("List Items")
    }
}

impl ListCreatedItems {
    pub fn new() -> ListCreatedItems {
        let mut options = Vec::new();
        options.push(("List DIDs".to_string(), ScreenEvent::SelectListDIDs));
        options.push(("List VCs".to_string(), ScreenEvent::SelectListVCs));
        options.push(("Back".to_string(), ScreenEvent::Cancel));
        ListCreatedItems { options }
    }
    
}
