use crate::{AppContext, Command, Output, ScreenEvent};

pub struct CreateDIDCommand;

impl Command for CreateDIDCommand {
    fn  execute(&mut self, _context: &AppContext) -> ScreenEvent {
        self.print_tile();
        
        ScreenEvent::Success

    }

    fn print_tile(&self) {
        Output::clear_screen();
        Output::print_screen_title("Create DID")
    }
}

impl CreateDIDCommand {
    
    fn print_options(&self) {
        println!("1. Create a new DID");
        println!("2. Create a new VC");
        println!("3. Exit");
    }
    

    
}