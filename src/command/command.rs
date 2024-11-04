use crate::ScreenEvent;

pub trait Command{
    fn execute(&mut self) -> ScreenEvent;
}