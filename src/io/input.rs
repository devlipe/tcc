pub struct Input;

impl Input {

    pub fn wait_for_user_input(msg: &str) {
        println!("{}", msg);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
    }
    pub fn get_number_input(min: usize , max: usize) -> usize {
        loop {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();

            let trimmed_input = input.trim();

            // Check if input is a blank line
            if trimmed_input.is_empty() {
                println!("Input cannot be blank. Please try again.");
                continue;
            }
            // Check if input is a number
            if !trimmed_input.chars().all(char::is_numeric) {
                println!("Invalid input. Please enter a number.");
                continue;
            }

            // Check if the input is within the valid range
            if let Ok(selection) = trimmed_input.parse::<usize>() {
                if selection >= min && selection <= max {
                    return selection;
                }
            }

            println!(
                "Invalid input. Please enter a number between {} and {}.",
                min, max
            );
        }
    }
}
