use crossterm::event::{self, Event};
use crate::prelude::*;

/// Communicates when and what to return from [`State::run`] by a running state. 
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Signal {
    /// The state should exit and be returned. 
    Done, 
    /// The state should exit and not be returned. 
    Cancelled, 
    /// The state should continue running. 
    Running,
}

/// Defines the event loop of an application state. 
/// 
/// 
/// # Usage
/// 
/// For most applications, implementing [`State::draw`] and [`State::input`] will suffice. 
/// - [`State::draw`] draws the user interface using [Ratatui](ratatui). 
/// - [`State::input`] handles key press events. 
/// 
/// Afterward, [`State::run`] may be called to enter the event loop. 
/// 
/// If [events](Event) other than key press events are required in an application, it may implement
/// [`State::event`], which handles any and all events read from the backend. Its default implementation
/// simply delegates key press events to [`State::input`] and discards the rest. 
/// 
/// The interface provided by [`State::run`] is fairly low-level. In most cases, a wrapper function should be
/// used to provide a more bespoke interface. E.g., [`dialog::confirm`], which creates a confirm dialog
/// state, runs it, and then returns whether the user pressed `y` or `n`. 
/// 
/// 
/// # Dummy State
/// 
/// A dummy (or no-nop) state is implemented through `()`. This is useful when a state is expected but not
/// used; e.g. to display a [`dialog`] without a background. 
/// 
/// The dummy state draws nothing and exits as soon as a key is pressed. 
/// 
/// 
/// # Returns
/// 
/// A state "returns" when [`State::run`] returns:  
/// 
/// - `Some(self)` if the state exits with [`Signal::Done`]. 
/// - `None` if the state exits with [`Signal::Cancelled`]. 
/// 
/// 
/// # Examples 
/// 
/// A state with a counter that increases when the user presses `up`: 
/// 
/// ```no_run
/// use std::io;
/// use ratatui::widgets::Paragraph;
/// use tundra::prelude::*;
/// 
/// struct Counter {
///     value: u32, 
/// }
/// 
/// impl State for Counter {
///     type Error = io::Error;
///     type Global = ();
///     
///     fn draw(&self, frame: &mut Frame) {
///         let widget = Paragraph::new(format!("{}", self.value));
///         frame.render_widget(widget, frame.size());
///     }
///     
///     fn input(&mut self, key: KeyEvent, ctx: &mut Context) -> io::Result<Signal> {
///         match key.code {
///             KeyCode::Up    => self.value += 1, 
///             KeyCode::Enter => return Ok(Signal::Done), 
///             KeyCode::Esc   => return Ok(Signal::Cancelled), 
///             _ => (), 
///         }
///         Ok(Signal::Running)
///     }
/// }
/// 
/// // wrapper over `State::run` to return the entered value; a common pattern
/// pub fn counter(ctx: &mut Context) -> io::Result<u32> {
///     let counter = Counter{ value: 0 }.run(ctx)?;
///     let value = counter
///         .map(|c| c.value)
///         .unwrap_or(0);
///     Ok(value)
/// }
/// ```
pub trait State: Sized {
    /// Type of error that can occur. If running the state is infallible, set this to the [`Never`] type. 
    type Error;

    /// Type of the application-defined global inside [`Context`]. This should be set to the same type as the
    /// one used when initializing the [`Context`]. If no global is used, this may be set to `()`. 
    type Global;

    /// Draw the state to a [`Frame`] using [Ratatui](ratatui). 
    /// 
    /// See [Ratatui's documentation](ratatui) for how to construct and render widgets. 
    fn draw(&self, frame: &mut Frame);
    
    /// Update the state with a key press input. 
    /// 
    /// This is called by the default implementation of [`State::event`] when a key input event is read. 
    /// 
    /// 
    /// # Default
    /// 
    /// Always returns [`Signal::Running`]. 
    #[allow(unused_variables)]
    fn input(&mut self, key: KeyEvent, ctx: &mut Context<Self::Global>) -> Result<Signal, Self::Error> {
        Ok(Signal::Running)
    }

    /// Update the state with an event. 
    /// 
    /// This is called by the default implementation of [`State::run`] when an event is read. 
    /// 
    /// 
    /// # Default
    /// 
    /// Simply delegates to [`State::input`], representing the most common use case. All other events are
    /// discarded. 
    fn event(&mut self, event: Event, ctx: &mut Context<Self::Global>) -> Result<Signal, Self::Error> {
        if let Event::Key(key_event) = event {
            self.input(key_event, ctx)
        } else {
            Ok(Signal::Running)
        }
    }

    /// Enters the event loop. 
    /// 
    /// 
    /// # Default
    /// 
    /// Calls [`State::draw`] and [`State::event`] until the latter returns [`Signal::Done`] or
    /// [`Signal::Cancelled`]. 
    /// 
    /// 
    /// # Panics
    /// 
    /// When [`ratatui::Terminal::draw`] or [`crossterm::event::read`](event::read()) fails. 
    fn run(mut self, ctx: &mut Context<Self::Global>) -> Result<Option<Self>, Self::Error> {
        loop {
            ctx.draw_state(&self).unwrap();
            let event = event::read().unwrap();
            
            match self.event(event, ctx)? {
                Signal::Done      => break Ok(Some(self)),
                Signal::Cancelled => break Ok(None),
                Signal::Running   => (),
            }
        }
    }
}

/// Implements a dummy (or no-op) [`State`] through `()`. It draws nothing and exits as soon as a key is
/// pressed. 
/// 
/// This is useful when a state is expected but not used; e.g. if you want to display a [`dialog`] without a
/// background. 
impl State for () {
    type Error = Never;
    type Global = ();

    fn draw(&self, _frame: &mut Frame) {
        ()
    }

    fn input(&mut self, _key: KeyEvent, _ctx: &mut Context) -> Result<Signal, Never> {
        Ok(Signal::Done)
    }
}

/// A type with no valid values. 
/// 
/// Defined to be used as [`State::Error`] type for infallible states. To be replaced with the `!` primitive
/// once stabilised. 
#[derive(Clone, Copy, Debug)]
pub enum Never {}
