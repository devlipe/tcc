use crate::{AppContext, Command, CreateDIDCommand, CreateVCCommand, ExitAppCommand, MainMenuCommand, ScreenEvent, ScreenFSM, ScreenState, VerifyVCCommand};
use rust_fsm::StateMachine;

pub struct App {
    fsm: StateMachine<ScreenFSM>,
    context: AppContext,
}


impl App {
    pub fn new(context: AppContext) -> Self {
        App {
            fsm: StateMachine::new(),
            context
        }
    }

    pub fn run(&mut self) {
        
        loop {
            // Match the current state to choose the appropriate command
            let mut command: Box<dyn Command> = match self.fsm.state() {
                ScreenState::MainMenu => Box::new(MainMenuCommand),
                ScreenState::CreateDIDWorkflow => Box::new(CreateDIDCommand),
                ScreenState::CreateVCWorkflow => Box::new(CreateVCCommand),
                ScreenState::VerifyVCWorkflow => Box::new(VerifyVCCommand),
                ScreenState::ExitAppWorkflow => Box::new(ExitAppCommand),
            };

            // Execute the command and get the resulting event
            let event = command.execute(&self.context);

            // Check if the event is Exit and the State is ExitAppWorkflow, if so, break the loop
            if event == ScreenEvent::Exit && *self.fsm.state() == ScreenState::ExitAppWorkflow {
                break;
            }
           
           
            // Update state based on event
            let _output = self.fsm.consume(&event).unwrap();



        }
    
    }
}