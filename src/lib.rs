//! Tundra provides a framework and utilities for creating interactive terminal applications with
//! [Ratatui](ratatui). 
//! 
//! [Ratatui](ratatui) is a comprehensive library for creating singular user interfaces in the terminal, but
//! lacks features for organizing larger applications --- composed of several interfaces --- and for receiving user
//! data input. 
//! 
//! Tundra aims to extend the functionality of [Ratatui](ratatui) with utilities for: 
//! - Defining [application states](State). 
//! - Managing the terminal [environment and context](Context). 
//! - Displaying messages through [modal dialogs](dialog). 
//! - Receiving user input through [input forms](dialog::form!) and [fields](field). 
//! 
//! Tundra is also highly extensible with tools to easily define [your own dialogs](dialog::Dialog) and
//! [input fields](field::Field). 
//! 
//! It is **not** intended to be a replacement for or wrapper over [Ratatui](ratatui), nor the
//! [backend](ratatui::backend). [Ratatui](ratatui) is still required to draw the user interface of each
//! application state, and the [backend](ratatui::backend) is still required for low-level terminal
//! operations. 
//! 
//! 
//! # Getting Started
//! 
//! Add Tundra and Ratatui to the project dependencies: 
//! ```text
//! $ cargo add tundra ratatui
//! ```
//! 
//! Import the [Tundra prelude](prelude), exposing symbols required in virtually all applications: 
//! ```no_run
//! use tundra::prelude::*;
//! ```
//! 
//! 
//! # Basic Usage
//! 
//! First construct a [context](Context). This represents the underlying [terminal](ratatui::Terminal), but 
//! has added RAII to automatically initialize and reset the terminal environment. 
//! 
//! ```no_run
//! # use tundra::Context;
//! let ctx = Context::new()?;
//! # Ok::<(), std::io::Error>(())
//! ```
//! 
//! This [context](Context) gets passed around between application states --- allowing them to draw to the
//! terminal. 
//! 
//! Next, create an application state. For this example, we'll create a state with a counter that increases
//! when the user presses `up`. 
//! 
//! ```no_run
//! struct Counter {
//!     value: u32, 
//! }
//! ```
//! 
//! To define the event loop of the state, implement the [`State`] trait. 
//! 
//! ```no_run
//! use std::io;
//! use ratatui::widgets::Paragraph;
//! # use tundra::prelude::*;
//! # struct Counter{ value: u32 };
//! 
//! impl State for Counter {
//!     type Error = io::Error;
//!     type Global = ();
//! 
//!     fn draw(&self, frame: &mut Frame) {
//!         let widget = Paragraph::new(format!("{}", self.value));
//!         frame.render_widget(widget, frame.size());
//!     }
//! 
//!     fn input(&mut self, key: KeyEvent, _ctx: &mut Context) -> io::Result<Signal> {
//!         match key.code {
//!             KeyCode::Up    => self.value += 1, 
//!             KeyCode::Enter => return Ok(Signal::Done), 
//!             KeyCode::Esc   => return Ok(Signal::Cancelled), 
//!             _ => (), 
//!         }
//!         Ok(Signal::Running)
//!     }
//! }
//! ```
//! 
//! Some notes on the implementation: 
//! - `Error` is used to specify the type of error that can occur. 
//! - `Global` can be used to store a [global value inside the context](Context#application-defined-global). 
//! - `draw` is used to draw the user interface using [Ratatui](ratatui). 
//! - `input` is used to handle key press events. The [`Signal`] return value indicates to the event loop 
//! when and what to return. 
//! 
//! Our counter can now be ran using [`State::run`]. Let's also extract the entered value from the counter
//! after it's done running. 
//! 
//! ```no_run
//! # use std::io;
//! # use tundra::prelude::*;
//! # struct Counter{ value: u32 };
//! # impl Counter{
//! #   fn run(self, _: &mut Context) -> io::Result<Option<Self>> { Ok(Some(self)) }
//! # }
//! # let mut ctx = &mut Context::new().unwrap();
//! // returns when the user presses enter or escape (per our State::input)
//! let counter = Counter{ value: 0 }.run(ctx)?;
//! 
//! // the returned `counter` is `Some(Counter)` if enter was pressed, and `None` otherwise
//! let value = counter
//!     .map(|c| c.value)
//!     .unwrap_or(0);
//! # Ok::<(), std::io::Error>(())
//! ```
//! 
//! This is a fair amount of boiler-plate to write each time the `Counter` state is ran, so let's create a 
//! wrapper function that does this for us --- a common pattern! 
//! 
//! ```no_run
//! # use std::io;
//! # use tundra::prelude::*;
//! # struct Counter{ value: u32 };
//! # impl Counter{
//! #   fn run(self, _: &mut Context) -> io::Result<Option<Self>> { Ok(Some(self)) }
//! # }
//! pub fn counter(ctx: &mut Context) -> io::Result<u32> {
//!     let counter = Counter{ value: 0 }.run(ctx)?;
//!     let value = counter
//!         .map(|c| c.value)
//!         .unwrap_or(0);
//!     Ok(value)
//! }
//! ```
//! 
//! This interface can now be used from other states to "transition" to the `Counter` state! For the sake of
//! argument, let's create and run a new counter from within our existing counter whenever the user presses
//! `tab`. The value entered in the new counter (the "transitioned-to" state) will be added onto the current
//! counter. 
//! 
//! ```no_run
//! # use std::io;
//! # use tundra::prelude::*;
//! # fn counter(_: &mut Context) -> io::Result<u32> { Ok(0) }
//! # struct Counter{ value: u32 };
//! # impl Counter {
//! fn input(&mut self, key: KeyEvent, ctx: &mut Context) -> io::Result<Signal> {
//!     match key.code {
//!         KeyCode::Up    => self.value += 1, 
//!         KeyCode::Tab   => self.value += counter(ctx)?, 
//!         KeyCode::Enter => return Ok(Signal::Done), 
//!         KeyCode::Esc   => return Ok(Signal::Cancelled), 
//!         _ => (), 
//!     }
//!     Ok(Signal::Running)
//! }
//! # }
//! ```
//! 
//! See the [`counter` example](https://github.com/user-simon/tundra/blob/main/examples/counter.rs) for the
//! complete code. 
//! 
//! 
//! # Modal Dialogs
//! 
//! Modal dialogs are small pop-up "windows" displayed atop a background state. They contain messages or
//! prompt for user input and are shown simply by calling a function. Here is an example of showing an error
//! message in a dialog: 
//! ```no_run
//! # use tundra::prelude::*;
//! # let current_state = &();
//! # let ctx = &mut Context::new().unwrap();
//! // let current_state: &impl State
//! // let ctx: &mut Context<_>
//! dialog::error("Failed evicting tenant.", current_state, ctx)?;
//! # Ok::<(), std::io::Error>(())
//! ```
//! 
//! Here, `current_state` is a reference to whatever state the dialog should be drawn over. If the dialog is
//! being invoked from within a state, this would be `&self`. The error dialog --- and the [`dialog::error`]
//! function by extension --- returns once the user presses a key, acknowledging the error. Above some state
//! with a [Ratatui table](ratatui::widgets::Table), this shows as:  
//! 
//! ![dialog error demo](https://raw.githubusercontent.com/user-simon/tundra/main/img/dialog_error.png)
//! 
//! See the [dialog module](dialog) for a full list of the dialogs provided by Tundra, and for how to create
//! your own! 
//! 
//! Note that there is nothing magic about dialogs; they are implemented using the same machinery as any
//! other state, but have added logic to store a reference to the background state and to draw it before the
//! dialog. 
//! 
//! 
//! # User Input
//! 
//! User input is facilitated through the [form macro](dialog::form!), which displays a dialog containing a
//! specified set of [input fields](field). The return value of the macro invocation is a struct containing 
//! the values entered by the user for each field. Here is an example of showing a form, and once it's been
//! submitted, retrieving the entered values: 
//! ```no_run
//! # use tundra::{prelude::*, field::*};
//! # let current_state = &();
//! # let ctx = &mut Context::new().unwrap();
//! // let current_state: &impl State
//! // let ctx: &mut Context<_>
//! 
//! let values = dialog::form!{
//!     location: Textbox{ name: "Location" }, 
//!     rent: Slider<u32>{ name: "Monthly rent", range: 1..=5000, step: 50 }, 
//!     pets_allowed: Checkbox{ name: "Pets allowed" }, 
//!     [title]: "Register Rent Unit", 
//!     [context]: ctx, 
//!     [background]: current_state, 
//! }?;
//! 
//! if let Some(values) = values {
//!     let location: String = values.location;
//!     let rent: u32 = values.rent;
//!     let pets_allowed: bool = values.pets_allowed;
//! }
//! # Ok::<(), std::io::Error>(())
//! ```
//! 
//! Some notes about the example: 
//! - As with other dialogs, a context and background state must be provided. For forms, this is done with
//! the `[context]` and `[background]` "meta-fields". 
//! - The values (and the fields) are stored as members of unspellable structs created inside the macro --- 
//! no runtime lookup is required! Values are accessed using the same identifers that the corresponding
//! fields were declared with. 
//! - Forms can be cancelled by the user, which is represented by a `None` return value. 
//! - The type annotations for the field values are not required. 
//! 
//! Above some state with a [Ratatui table](ratatui::widgets::Table) (and with some example data entered),
//! this shows as: 
//! 
//! ![dialog form demo](https://raw.githubusercontent.com/user-simon/tundra/main/img/dialog_form.png)
//! 
//! See the [form macro](dialog::form!) for the complete macro specification, and see the
//! [field module](field) for a full list of the field types provided by Tundra, and for how to create your
//! own!
//! 
//! 
//! # A Note on the Backend
//! 
//! [Ratatui](ratatui) has support for several terminal [backends](ratatui::backend). If you don't know what
//! that means, this note holds no significance for you. 
//! 
//! Tundra currently only supports the [crossterm] backend. This is due to a lack of abstraction over the
//! different backends. Code --- particularly pertaining to [context](Context) and event handling --- would
//! have to be written and repeated for each backend. 
//! 
//! If you need another backend for your project, Tundra is not for you --- at least for the moment. 
//! 
//! 
//! # Examples
//! 
//! TODO

mod context;
pub mod dialog;
pub mod field;
mod state;

#[doc(no_inline)]
pub use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[doc(no_inline)]
pub use ratatui::Frame;

pub use crate::{
    state::*, 
    context::*, 
};

/// Exposes symbols required in virtually all applications. 
pub mod prelude {
    #[doc(no_inline)]
    pub use super::{
        dialog, 
        KeyCode, KeyEvent, KeyModifiers, Frame, 
        Signal, State, 
        Context, 
    };
}
