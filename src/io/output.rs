// Create a class that will handle the output of the program.
// This class will be used to print the output of the program.

use crate::ScreenEvent;
use colored::*;
use crossterm::execute;
use crossterm::terminal::ClearType;
use std::io::{stdout, Write};
use std::iter::Map;
use std::str::Split;
use tokio::sync::watch;
use tokio::time::{sleep, Duration};
use users::{get_current_uid, get_user_by_uid};

pub struct Output;

impl Output {
    //Clear the screen
    pub fn clear_screen() {
        execute!(stdout(), crossterm::terminal::Clear(ClearType::All)).unwrap();
        // execute!(stdout(), crossterm::terminal::Clear(ClearType::Purge)).unwrap();
    }

    pub fn show_welcome_message() {
        // Clear the screen
        Self::clear_screen();

        // Get the current user or set to "fellow user" if not found
        let user_name: String = get_user_by_uid(get_current_uid())
            .map(|user| user.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| "fellow user".to_string());

        // Print the welcome message with colors
        println!(
            "{}",
            format!("Welcome to the Petrus, {}!", user_name)
                .bold()
                .yellow()
        );
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

    pub fn print_during_loading(message: &str) {
        println!("\r{}", message);
    }
    pub fn print_screen_title(title: &str) {
        println!("\n{}", title.bold().blue());
        // Print 2 blank lines
        println!("\n");
    }

    pub async fn cooldown() {
        sleep(Duration::from_secs(2)).await;
    }

    pub fn snake_to_title_case(input: &str) -> String {
        // remove the file extension
        let input = input.split('.').next().unwrap();

        Self::uppercase_first_letter(input)
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn snake_to_camel_case(input: &str) -> String {
        // remove the file extension
        let input = input.split('.').next().unwrap();

        Self::uppercase_first_letter(input).collect::<String>()
    }

    fn uppercase_first_letter(input: &str) -> Map<Split<char>, fn(&str) -> String> {
        input.split('_').map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
    }

    pub fn display_with_pagination<T: Clone>(
        items: &Vec<T>,
        display_fn: fn(&Vec<T>, usize),
        page_size: usize,
        selectable: bool,
    ) -> usize {
        let total_pages = (items.len() + page_size - 1) / page_size;
        let mut current_page = 0;
        let mut error = String::new();

        loop {
            // Calculate the start and end indices for the current page
            let start_index = current_page * page_size;
            let end_index = usize::min(start_index + page_size, items.len());

            display_fn(&items[start_index..end_index].to_vec(), start_index + 1);

            println!("Page {}/{}", current_page + 1, total_pages);

            // Navigation options
            Self::print_navigation_options(selectable);

            if !error.is_empty() {
                println!("Error: {}", error.clone());
                error.clear();
            }

            // Wait for user input
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();

            Self::clear_screen();

            match input {
                "" => {
                    if current_page + 1 < total_pages {
                        current_page += 1;
                    } else {
                        error = "You are already on the last page.".to_string();
                    }
                }
                "p" => {
                    if current_page > 0 {
                        current_page -= 1;
                    } else {
                        error = "You are already on the first page.".to_string();
                    }
                }
                "q" => {
                    if !selectable {
                        println!("Exiting navigation.");
                        return 0;
                    } else {
                        error = "Please own up to your choices and end this stream".to_string();
                    }
                }
                _ => {
                    if selectable {
                        // Check if input is a number
                        if !input.chars().all(char::is_numeric) {
                            println!("Invalid input. Please enter a number.");
                            continue;
                        }

                        // Check if the input is within the valid range
                        if let Ok(selection) = input.parse::<usize>() {
                            if selection >= start_index + 1 && selection <= end_index {
                                return selection;
                            }
                        }

                        error = format!(
                            "Invalid input. Please enter a number between {} and {}.",
                            start_index + 1,
                            end_index
                        );
                    } else {
                        error =
                            "Invalid input. Use 'n' for next, 'p' for previous, or 'q' to quit."
                                .to_string();
                    }
                }
            }
        }
    }

    fn print_navigation_options(selectable: bool) {
        if selectable {
            println!(
                "{} {} {}",
                "Navigation:",
                "enter - Next page".green().bold(),
                "p - Previous page".bold().red()
            );
        } else {
            println!(
                "{} {} {} {}",
                "Navigation:",
                "enter - Next page".green().bold(),
                "p - Previous page".bold().red(),
                "q - Quit".bold().blue()
            );
        }
    }
    
    pub fn print_options_vec(ops: &Vec<(String, ScreenEvent)>) {
        for (index, option) in ops.iter().enumerate() {
            println!("{}. {}", index + 1, option.0);
        }
    }

    pub fn print_options_vec_generic<T: ToString>(ops: &Vec<T>) {
        for (index, option) in ops.iter().enumerate() {
            println!("{}. {}", index + 1, option.to_string());
        }
    }
}
