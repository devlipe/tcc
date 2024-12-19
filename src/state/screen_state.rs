#[derive(Clone, Debug, PartialEq)]
pub enum ScreenState {
    MainMenu,
    ListItemsMenu,
    CreateDIDWorkflow,
    ListDIDsWorkflow,
    CreateVCWorkflow,
    VerifyVCWorkflow,
    ExitAppWorkflow,
    
}
