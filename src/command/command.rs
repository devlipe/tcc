use crate::{AppContext, ScreenEvent};

pub trait Command{
    fn execute(&mut self, context: &AppContext) -> ScreenEvent;
    
    fn print_tile(&self);
}