#[derive(Clone, Debug, PartialEq)]
pub enum ScreenState {
    MainMenu,
    CreateDIDWorkflow,
    CreateVCWorkflow,
    VerifyVCWorkflow,
    ExitAppWorkflow,
}
