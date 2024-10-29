

// Create a class that will handle the output of the program.
// This class will be used to print the output of the program.

use std::io;
use std::io::{stdout, Write};
use crossterm::execute;
use crossterm::style::{Color, Stylize};
use crossterm::terminal::ClearType;
use users::{get_current_uid, get_user_by_uid};

pub struct Output;

impl Output {
    
    // Print the output of the program.
    pub fn print_output(&self, output: String) {
        println!("{}", output);
    }
    
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
        println!("{} {}", "Hello, {}!\n".yellow(), user_name);
        println!("This is a simple stocks selector. It works by sorting and ranking stocks by ROA, EV/Edit, and P/L.\n");
        println!("{:?}\tDeveloped by: https://github.com/devlipe", Color::Green);
        println!("\tOn April 10th, 2022");
        println!("\tVersion 1.0.0{:?}", Color::Reset);

        // Prompt to continue
        print!("{:?}Press enter to continue...{:?}", Color::Red, Color::Reset);
        stdout().flush().unwrap_or_default(); // Ensure prompt message is printed immediately

        // Wait for user input to continue
        let mut trash = String::new();
        io::stdin().read_line(&mut trash).unwrap_or_default();
        
    }
}