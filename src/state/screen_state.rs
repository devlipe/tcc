#[derive(Clone, Debug, PartialEq)]
pub enum ScreenState {
    MainMenu,
    CreateDIDWorkflow,
    ListDIDsWorkflow,
    CreateVCWorkflow,
    VerifyVCWorkflow,
    ExitAppWorkflow,
    
}
