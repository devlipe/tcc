use crate::{Command, Output, ScreenEvent};
use std::io;
use std::io::stdin;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

pub struct ExitAppCommand;

impl Command for ExitAppCommand {
    fn execute(&mut self) -> ScreenEvent {
        self.print_tile();
        ExitAppCommand::show_exit_message()
    }

    fn print_tile(&self) {
        Output::clear_screen();
        Output::print_screen_title("Exit App")
    }
}

impl ExitAppCommand {
    fn show_exit_message() -> ScreenEvent {
        println!("It is a shame that we have to part ways. Goodbye!");
        println!("Press any key to exit (ESC to cancel):");

        let stdin = stdin();
        let mut keys = stdin.keys();

        // Enable raw mode
        let _stdout = io::stdout().into_raw_mode().unwrap();

        while let Some(Ok(key)) = keys.next() {
            return match key {
                Key::Esc => {
                    ScreenEvent::Cancel // Cancel exit if ESC is pressed
                }
                _ => {
                    println!("Exiting...");
                    ScreenEvent::Exit // Exit if any other key is pressed
                }
            };
        }

        // Default case
        ScreenEvent::Exit
    }
}
