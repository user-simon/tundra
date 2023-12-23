//! TODO

mod context;
pub mod dialog;
pub mod input;
mod state;

#[doc(no_inline)]
pub use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[doc(no_inline)]
pub use ratatui::Frame;

pub use crate::{
    state::*, 
    context::*, 
};

pub mod prelude {
    #[doc(no_inline)]
    pub use super::{
        dialog, 
        KeyCode, KeyEvent, KeyModifiers, Frame, 
        Signal, State, 
        Context, 
    };
}
