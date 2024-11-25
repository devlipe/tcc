#[derive(Clone, Debug, PartialEq)]
pub enum ScreenEvent {
    SelectCreateDID,
    SelectCreateVC,
    SelectVerifyVC,
    Cancel,
    Success,
    Exit,
}
