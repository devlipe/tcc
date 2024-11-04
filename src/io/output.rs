

// Create a class that will handle the output of the program.
// This class will be used to print the output of the program.

use std::io;
use std::io::{stdout, Write};
use crossterm::execute;
use crossterm::terminal::ClearType;
use users::{get_current_uid, get_user_by_uid};
use colored::*;

pub struct Output;

impl Output {
    
    //Clear the screen
    pub fn clear_screen(&self) {
        execute!(stdout(), crossterm::terminal::Clear(ClearType::All)).unwrap();
    }

    pub fn show_welcome_message(&self) {
        // Clear the screen
        self.clear_screen();

        // Get the current user or set to "fellow user" if not found
        let user_name = get_user_by_uid(get_current_uid())
            .map(|user| user.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| "fellow user".to_string());

        // Print the welcome message with colors
        println!("{}", format!("Welcome to the Petrus, {}!", user_name).bold().yellow());
        println!("This is a Proof Of Concept (POC) designed and created for the final project of the student Felipe Ferreira - 102017\n");
        println!("{}", "\tDeveloped by: https://github.com/devlipe".green());
        println!("{}", "\tOn October 7th, 2024".green());
        println!("{}", "\tVersion 1.0.0".green());

        // Prompt to continue
        print!("Press enter to continue...");
        stdout().flush().unwrap_or_default(); // Ensure prompt message is printed immediately

        // Wait for user input to continue
        let mut trash = String::new();
        io::stdin().read_line(&mut trash).unwrap_or_default();
    }
}