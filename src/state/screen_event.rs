#[derive(Clone, Debug, PartialEq)]
pub enum ScreenEvent {
    SelectCreateDID,
    SelectListDIDs,
    SelectListVCs,
    SelectCreateVC,
    SelectVerifyVC,
    SelectListItems,
    Cancel,
    Success,
    Exit,
}
