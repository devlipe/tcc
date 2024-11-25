

// Create a class that will handle the output of the program.
// This class will be used to print the output of the program.

use colored::*;
use crossterm::execute;
use crossterm::terminal::ClearType;
use std::io::{stdout, Write};

use tokio::sync::watch;
use tokio::time::{sleep, Duration};
use users::{get_current_uid, get_user_by_uid};

pub struct Output;

impl Output {
    
    //Clear the screen
    pub fn clear_screen() {
        execute!(stdout(), crossterm::terminal::Clear(ClearType::All)).unwrap();
    }

    pub fn show_welcome_message() {
        // Clear the screen
        Self::clear_screen();

        // Get the current user or set to "fellow user" if not found
        let user_name: String = get_user_by_uid(get_current_uid())
            .map(|user| user.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| "fellow user".to_string());

        // Print the welcome message with colors
        println!("{}", format!("Welcome to the Petrus, {}!", user_name).bold().yellow());
        println!("This is a Proof Of Concept (POC) designed and created for the final project of the student Felipe Ferreira - 102017\n");
        println!("{}", "\tDeveloped by: https://github.com/devlipe".green());
        println!("{}", "\tOn October 7th, 2024".green());
        println!("{}", "\tVersion 1.0.0".green());

        stdout().flush().unwrap_or_default(); // Ensure prompt message is printed immediately

    }
    
    pub async fn loading_animation(rx: watch::Receiver<bool>) {
        let braille_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

        while *rx.borrow() {
            // Adjust the range to control how long the animation runs
            for &frame in &braille_frames {
                print!("\r{} Loading ", frame); // `\r` moves the cursor to the start of the line
                stdout().flush().unwrap();
                sleep(Duration::from_millis(100)).await;
            }
            
        }
    }
    pub fn print_screen_title(title: &str) {
        println!("\n{}", title.bold().blue());
        // Print 2 blank lines
        println!("\n");
    }
}