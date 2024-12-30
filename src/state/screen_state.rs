#[derive(Clone, Debug, PartialEq)]
pub enum ScreenState {
    MainMenu,
    ListItemsMenu,
    CreateDIDWorkflow,
    ListDIDsWorkflow,
    ListVCsWorkflow,
    CreateVCMenu,
    CreateNormalVCWorkflow,
    CreateSDVCWorkflow,
    VerifyVCWorkflow,
    CreateVPWorkflow,
    ExitAppWorkflow,
}
