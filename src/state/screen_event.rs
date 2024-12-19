#[derive(Clone, Debug, PartialEq)]
pub enum ScreenEvent {
    SelectCreateDID,
    SelectListDIDs,
    SelectCreateVC,
    SelectVerifyVC,
    SelectListItems,
    Cancel,
    Success,
    Exit,
}
