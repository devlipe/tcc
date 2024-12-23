#[derive(Clone, Debug, PartialEq)]
pub enum ScreenState {
    MainMenu,
    ListItemsMenu,
    CreateDIDWorkflow,
    ListDIDsWorkflow,
    ListVCsWorkflow,
    CreateVCWorkflow,
    VerifyVCWorkflow,
    ExitAppWorkflow,
    
}
