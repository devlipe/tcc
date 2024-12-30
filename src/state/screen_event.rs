#[derive(Clone, Debug, PartialEq)]
pub enum ScreenEvent {
    CreateDID,
    ListDIDs,
    ListVCs,
    CreateVC,
    CreateNormalVC,
    CreateSDVC,
    VerifyVC,
    CreateVP,
    ListItems,
    Cancel,
    Success,
    Exit,
}
