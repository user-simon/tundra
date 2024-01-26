use std::convert::Infallible;

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
/// # Error Handling
/// 
/// Arbitrary application-defined errors are supported through the [`State::Result`] type. Errors can be
/// returned from [`State::input`] or [`State::event`], and are propogated through [`State::run`]. 
/// 
/// Requiring a result type as opposed to an error type (which is generally standard practice) allows states
/// to accept types that aren't results, but can *behave* like results. Most prominently: `Option<T>` and
/// `T` itself. The latter case is especially interesting since it allows states for which no error can occur
/// to be implemented without any mention of [`Result`] or [`Infallible`] trickery --- all return values are
/// implicitly `Ok`. 
/// 
/// 
/// # Dummy state
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
///     type Result<T> = T;
///     type Global = ();
///     
///     fn draw(&self, frame: &mut Frame) {
///         let widget = Paragraph::new(format!("{}", self.value));
///         frame.render_widget(widget, frame.size());
///     }
///     
///     fn input(&mut self, key: KeyEvent, ctx: &mut Context) -> Signal {
///         match key.code {
///             KeyCode::Up    => self.value += 1, 
///             KeyCode::Enter => return Signal::Done, 
///             KeyCode::Esc   => return Signal::Cancelled, 
///             _ => (), 
///         }
///         Signal::Running
///     }
/// }
/// 
/// // wrapper over `State::run` to return the entered value; a common pattern
/// pub fn counter(ctx: &mut Context) -> u32 {
///     Counter{ value: 0 }
///         .run(ctx)
///         .map(|counter| counter.value)
///         .unwrap_or(0)
/// }
/// ```
pub trait State: Sized {
    /// The result type, encoding what kinds of errors can occur when running the state: 
    /// - `Result<T, E>` in cases where an exact error `E` can occur. 
    /// - `Option<T>` in cases where the exact error is not important. 
    /// - `T` in cases where no error can occur. 
    /// 
    /// See the [trait-level](State#error-handling) documentation for more information. 
    type Result<T>: ResultLike<T>;

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
    fn input(&mut self, key: KeyEvent, ctx: &mut Context<Self::Global>) -> Self::Result<Signal> {
        ResultLike::from_result(Ok(Signal::Running))
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
    fn event(&mut self, event: Event, ctx: &mut Context<Self::Global>) -> Self::Result<Signal> {
        if let Event::Key(key_event) = event {
            self.input(key_event, ctx)
        } else {
            ResultLike::from_result(Ok(Signal::Running))
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
    fn run(mut self, ctx: &mut Context<Self::Global>) -> Self::Result<Option<Self>>
    where
        StateError<Self, Option<Self>>: From<StateError<Self, Signal>>
    {
        let result = loop {
            // we're intentionally panicking on `io::Error`s here to simplify application code (we would
            // otherwise have to force the application-defined error to implement `From<io::Error>`). 
            // applications that wish to handle `io::Error` explicitly can override this function
            ctx.draw_state(&self).unwrap();
            let event = event::read().unwrap();

            // generalized version of `let signal = self.event(...)?`
            let signal = match ResultLike::into_result(self.event(event, ctx)) {
                Ok(signal) => signal, 
                Err(err) => break Err(err.into()), 
            };
            
            match signal {
                Signal::Done      => break Ok(Some(self)), 
                Signal::Cancelled => break Ok(None), 
                Signal::Running   => (), 
            }
        };
        ResultLike::from_result(result)
    }
}

/// Short-hand for the type of error that can occur in a [`State`]. 
/// 
/// This is parameterised over the state `S` and the value type `T` (corresponding to the `Ok` type of a
/// result). 
pub type StateError<S, T> = <<S as State>::Result<T> as ResultLike<T>>::Error;

/// Implements a dummy (or no-op) [`State`] through `()`. It draws nothing and exits as soon as a key is
/// pressed. 
/// 
/// This is useful when a state is expected but not used; e.g. if you want to display a [`dialog`] without a
/// background. 
impl State for () {
    type Result<T> = T;
    type Global = ();

    fn draw(&self, _frame: &mut Frame) {
        ()
    }

    fn input(&mut self, _key: KeyEvent, _ctx: &mut Context) -> Signal {
        Signal::Done
    }
}

/// Generalization over data-carrying [`Result`]-like types. 
/// 
/// There are three significant implementations of this trait: 
/// - `Result<T, E>` itself, which has error type `E`. 
/// - `Option<T>`, which has error type `()`. 
/// - `T`, which has error type [`Infallible`] (or `!` once stabilised). 
/// 
/// Either three of these can be used in place of an explicit [`Result`] where a [`ResultLike`] type is
/// expected. This allows [`State`] to accept not only any error type (through `Result<T, E>`), but also the
/// absence of an error type (through `Option<T>`), and the absence of an error altogether (through `T`). 
/// 
/// 
/// # Limitations
/// 
/// There are limitations to this approach. Namely, it is very difficult to assert that [`State::Result`] has
/// the same error type regardless of its value type `T` (as is true for all three implementations listed
/// above). This means that to propogate an error from `State::Result<T>` to `State::Result<U>`, an explicit 
/// bound to assert that the conversion between the two (ostensibly distinct) error types exists must be
/// added. This is cumbersome for generic code (like the default implementation of [`State::run`]), but has
/// no bearing on the concrete implementations of the states themselves. 
pub trait ResultLike<T> {
    type Error;

    fn from_result(result: Result<T, Self::Error>) -> Self;
    fn into_result(self) -> Result<T, Self::Error>;
}

impl<T> ResultLike<T> for T {
    type Error = Infallible;

    fn from_result(result: Result<T, Infallible>) -> T {
        match result {
            Ok(x) => x, 
            Err(x) => match x {}
        }
    }

    fn into_result(self) -> Result<T, Infallible> {
        Ok(self)
    }
}

impl<T, E> ResultLike<T> for Result<T, E> {
    type Error = E;

    fn from_result(result: Result<T, E>) -> Result<T, E> {
        result
    }

    fn into_result(self) -> Result<T, E> {
        self
    }
}

impl<T> ResultLike<T> for Option<T> {
    type Error = ();

    fn from_result(result: Result<T, ()>) -> Option<T> {
        result.ok()
    }

    fn into_result(self) -> Result<T, ()> {
        self.ok_or(())
    }
}

/// A type with no valid values. 
/// 
/// Defined to be used as [`State::Error`] type for infallible states. To be replaced with the `!` primitive
/// once stabilised. 
#[derive(Clone, Copy, Debug)]
pub enum Never {}
