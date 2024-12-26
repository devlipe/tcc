#[derive(Clone, Debug, PartialEq)]
pub enum ScreenEvent {
    SelectCreateDID,
    SelectListDIDs,
    SelectListVCs,
    SelectCreateVC,
    SelectVerifyVC,
    SelectCreateVP,
    SelectListItems,
    Cancel,
    Success,
    Exit,
}
