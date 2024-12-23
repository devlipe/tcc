
use rust_fsm::{StateMachineImpl};
use crate::{ScreenEvent, ScreenState};

pub struct ScreenFSM;

impl StateMachineImpl for ScreenFSM{
    type Input = ScreenEvent;
    type State = ScreenState;
    type Output = ();
    const INITIAL_STATE: Self::State = ScreenState::MainMenu;


    fn transition(state: &ScreenState, event: &ScreenEvent) -> Option<ScreenState> {
        match (state, event) {
            //
            (ScreenState::MainMenu, ScreenEvent::SelectCreateDID) => Some(ScreenState::CreateDIDWorkflow),
            (ScreenState::MainMenu, ScreenEvent::SelectCreateVC) => Some(ScreenState::CreateVCWorkflow),
            (ScreenState::MainMenu, ScreenEvent::SelectVerifyVC) => Some(ScreenState::VerifyVCWorkflow),
            (ScreenState::MainMenu, ScreenEvent::SelectListItems) => Some(ScreenState::ListItemsMenu),

            (ScreenState::ListItemsMenu, ScreenEvent::Cancel) => Some(ScreenState::MainMenu),
            (ScreenState::ListItemsMenu, ScreenEvent::SelectListDIDs) => Some(ScreenState::ListDIDsWorkflow),
            (ScreenState::ListItemsMenu, ScreenEvent::SelectListVCs) => Some(ScreenState::ListVCsWorkflow),
            
             // Exit the program
            (ScreenState::MainMenu, ScreenEvent::Cancel) => Some(ScreenState::ExitAppWorkflow),
            
            
            // In case of cancel, return to the main menu
            (_, ScreenEvent::Cancel) => Some(ScreenState::MainMenu),
            // For the default case of success, return to the main menu
            (_, ScreenEvent::Success) => Some(ScreenState::MainMenu),
            
            _ => None,
        }
    }
    
    fn output(_state: &Self::State, _input: &Self::Input) -> Option<Self::Output> {
        None
    }
}

// state_machine! {
//     #[state_machine(input(crate::ScreenEvent), state(crate::ScreenState))]
//     screen_fsm(MainMenu)
// 
//     MainMenu => {
//         SelectCreateDID => CreateDIDWorkflow,
//         SelectCreateVC => CreateVCWorkflow,
//         SelectVerifyVC => VerifyVCWorkflow,
//     },
//     CreateDIDWorkflow(Cancel) => MainMenu,
//     CreateVCWorkflow(Cancel) => MainMenu,
//     VerifyVCWorkflow(Cancel) => MainMenu
// }

// fn test(){
//     let mut fsm = screen_fsm::StateMachine::new();
//     let event = ScreenEvent::SelectCreateDID;
//     let new_state =fsm.consume(&event).unwrap();
//     println!("{:?}", new_state);
// }
