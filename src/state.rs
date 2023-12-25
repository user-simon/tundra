use std::io;
use crossterm::event::{self, Event};
use crate::prelude::*;

/// Communicates when and what to return from [`State::run`] by a running state. 
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
/// The event loop is entered by calling [`State::run`]. 
/// 
/// Key input events are handled from [`State::input`], representing the most common use case. If other
/// events are needed, [`State::event`] may be implemented --- whose default implementation simply delegates
/// key input events to [`State::input`] and discards the rest. 
/// 
/// The state is drawn from [`State::draw`]. See [Ratatui's documentation](ratatui) on how to construct and
/// render widgets. 
/// 
/// The interface provided by [`State::run`] is fairly low-level. In most cases, a wrapper function may be
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
/// To create a state 
/// ```no_run
/// use std::io;
/// use ratatui::widgets::Paragraph;
/// use tundra::prelude::*;
/// 
/// struct MyState {
///     counter: i32, 
/// }
/// 
/// impl State for MyState {
///     type Error = io::Error;
///     type Global = ();
/// 
///     fn draw(&self, frame: &mut Frame) {
///         let counter_string = format!("{}", self.counter);
///         let widget = Paragraph::new(counter_string);
///         frame.render_widget(widget, frame.size());
///     }
/// 
///     fn input(&mut self, key: KeyEvent, ctx: &mut Context) -> io::Result<Signal> {
///         match key.code {
///             KeyCode::Up    => self.counter += 1, 
///             KeyCode::Down  => self.counter -= 1, 
///             KeyCode::Enter => return Ok(Signal::Done), 
///             KeyCode::Esc   => return Ok(Signal::Cancelled), 
///             _ => (), 
///         }
///         Ok(Signal::Running)
///     }
/// }
/// 
/// // wrapper function that constructs the state, runs it, and returns the entered value
/// pub fn run_my_state(ctx: &mut Context) -> io::Result<Option<i32>> {
///     let value = MyState{ counter: 0 }
///         .run(ctx)?
///         .map(|state| state.counter);
///     Ok(value)
/// }
/// ```
pub trait State: Sized {
    /// 
    type Error: From<io::Error>;

    /// Type of the application-defined global inside [`Context`]. This should be set to the same type as the
    /// one used when initializing the [`Context`]. If no global is used, this may be set to `()`. 
    type Global;

    /// Draw the state to a [`Frame`]. 
    /// 
    /// See [Ratatui's documentation](ratatui) for how to construct and render widgets. 
    fn draw(&self, frame: &mut Frame);
    
    /// Update the state with a key press input. 
    /// 
    /// This is called by the default implementation of [`State::event`] when a key input event is read. 
    fn input(&mut self, key: KeyEvent, ctx: &mut Context<Self::Global>) -> Result<Signal, Self::Error>;

    /// Update the state with an event. 
    /// 
    /// This is called by the default implementation of [`State::run`] when an event is read. 
    /// 
    /// The default implementation of this function simply delegates to [`State::input`], representing the
    /// most common use case. All other events are discarded. 
    fn event(&mut self, event: Event, ctx: &mut Context<Self::Global>) -> Result<Signal, Self::Error> {
        if let Event::Key(key_event) = event {
            self.input(key_event, ctx)
        } else {
            Ok(Signal::Running)
        }
    }

    /// The event loop. 
    /// 
    /// Calls [`State::draw`] and [`State::event`] until the latter returns [`Signal::Done`] or
    /// [`Signal::Cancelled`]. 
    fn run(mut self, ctx: &mut Context<Self::Global>) -> Result<Option<Self>, Self::Error> {
        loop {
            ctx.draw_state(&self)?;

            let event = event::read()?;
            
            match self.event(event, ctx)? {
                Signal::Done      => break Ok(Some(self)),
                Signal::Cancelled => break Ok(None),
                Signal::Running   => (),
            }
        }
    }
}

/// Implements a dummy (or no-op) [`State`] through `()`. 
/// 
/// This is useful when a state is expected but not used; e.g. if you want to display a [`dialog`] without a
/// background. 
impl State for () {
    type Error = io::Error;
    type Global = ();

    fn draw(&self, _frame: &mut Frame) {
        ()
    }

    fn input(&mut self, _key: KeyEvent, _ctx: &mut Context) -> io::Result<Signal> {
        Ok(Signal::Done)
    }
}
