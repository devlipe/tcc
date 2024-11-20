use std::io;
use termion::event::Key;
use termion::input::TermRead;
use crate::{AppContext, Command, Output, ScreenEvent};

pub struct  ExitAppCommand;

impl Command for ExitAppCommand {
    fn execute(&mut self, _context: &AppContext) -> ScreenEvent {
        self.print_tile();
        ExitAppCommand::show_exit_message();
        ScreenEvent::Exit
    }

    fn print_tile(&self) {
        Output::clear_screen();
        Output::print_screen_title("Exit App")
    }
}

impl  ExitAppCommand {
    fn show_exit_message() {
        println!("It is a shame that we have to part ways. Goodbye!");
        println!("Press any key to exit (ESC to cancel):");

        // Create a stdin lock to read key events
        let stdin = io::stdin();

        // Loop to handle key presses
        for key in stdin.keys() {
            match key.unwrap() {
                Key::Esc => {
                    println!("Exit cancelled");
                    return; // Cancel exit if ESC is pressed
                }
                _ => {
                    println!("Exiting...");
                    break; // Exit on any other key press
                }
            }
        }
    }
}
