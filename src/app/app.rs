use crate::{
    AppContext, Command, CreateDIDCommand, CreateVCMenu, CreateVCNormalCommand, CreateVCSDCommand,
    CreateVPCommand, ExitAppCommand, ListCreatedItems, ListDIDsCommand, ListVCsCommand,
    MainMenuCommand, ScreenEvent, ScreenFSM, ScreenState, VerifyVCCommand,
};
use rust_fsm::StateMachine;

pub struct App {
    fsm: StateMachine<ScreenFSM>,
    context: AppContext,
}

impl App {
    pub fn new(context: AppContext) -> Self {
        App {
            fsm: StateMachine::new(),
            context,
        }
    }

    pub fn run(&mut self) {
        loop {
            // Match the current state to choose the appropriate command
            let mut command: Box<dyn Command> = match self.fsm.state() {
                // Main Menu
                ScreenState::MainMenu => Box::new(MainMenuCommand::new()),

                // List Created Items
                ScreenState::ListItemsMenu => Box::new(ListCreatedItems::new()),
                ScreenState::ListDIDsWorkflow => Box::new(ListDIDsCommand::new(&self.context)),
                ScreenState::ListVCsWorkflow => Box::new(ListVCsCommand::new(&self.context)),

                // Create DID
                ScreenState::CreateDIDWorkflow => Box::new(CreateDIDCommand::new(&self.context)),

                // Create VC
                ScreenState::CreateVCMenu => Box::new(CreateVCMenu::new()),
                ScreenState::CreateSDVCWorkflow => Box::new(CreateVCSDCommand::new(&self.context)),
                ScreenState::CreateNormalVCWorkflow => {
                    Box::new(CreateVCNormalCommand::new(&self.context))
                }

                // Verify VC
                ScreenState::VerifyVCWorkflow => Box::new(VerifyVCCommand::new(&self.context)),

                // Create VP
                ScreenState::CreateVPWorkflow => Box::new(CreateVPCommand::new(&self.context)),

                // Exit App
                ScreenState::ExitAppWorkflow => Box::new(ExitAppCommand),
            };

            // Execute the command and get the resulting event
            let event = command.execute();

            // Check if the event is Exit and the State is ExitAppWorkflow, if so, break the loop
            if event == ScreenEvent::Exit && *self.fsm.state() == ScreenState::ExitAppWorkflow {
                break;
            }

            // Update state based on event
            let _output = self.fsm.consume(&event).unwrap();

            drop(command);
        }
    }
}
