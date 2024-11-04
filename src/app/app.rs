use rust_fsm::StateMachine;
use crate::{Command, ScreenState, MainMenuCommand, CreateDIDCommand, CreateVCCommand, VerifyVCCommand, ScreenFSM};

pub struct App {
    fsm: StateMachine<ScreenFSM>,
}

impl App {
    pub fn new() -> Self {
        App {
            fsm: StateMachine::new(),
        }
    }

    pub fn run(&mut self) {
        
        let mut count = 0;
        loop {
            // Match the current state to choose the appropriate command
            let mut command: Box<dyn Command> = match self.fsm.state() {
                ScreenState::MainMenu => Box::new(MainMenuCommand),
                ScreenState::CreateDIDWorkflow => Box::new(CreateDIDCommand),
                ScreenState::CreateVCWorkflow => Box::new(CreateVCCommand),
                ScreenState::VerifyVCWorkflow => Box::new(VerifyVCCommand),
            };

            // Execute the command and get the resulting event
            let event = command.execute();

            // Update state based on event
            let _output = self.fsm.consume(&event).unwrap();
            count += 1;
            if count > 10 {
                break;
            }

        }
    
    }
}