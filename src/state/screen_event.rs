#[derive(Clone, Debug, PartialEq)]
pub enum ScreenEvent {
    SelectCreateDID,
    SelectListDIDs,
    SelectCreateVC,
    SelectVerifyVC,
    Cancel,
    Success,
    Exit,
}
