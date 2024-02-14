use std::convert::Infallible;
use crossterm::event::{self, Event};
use crate::prelude::*;

/// Short-hand for the type of error that can occur in a [`State`]. 
/// 
/// This is parameterised over the state `S` and the value type `T` (corresponding to the `Ok` type of a
/// result). 
type Error<S, T> = <<S as State>::Result<T> as ResultLike<T>>::Error;

/// Dictates when and what to return from a running [`State`]. 
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Signal<T: State> {
    /// The state should return with given value. 
    Return(T::Out), 
    /// The given state should continue running. 
    Continue(T), 
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
/// used to provide a more bespoke interface. 
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
/// # Signals
/// 
/// The event handler [`State::event`] (and [`State::input`] by extension) communicates when and what to
/// return from [`State::run`] using [`Signal`]. A value of [`Signal::Continue`] indicates that the state
/// should continue running, whereas [`Signal::Return`] indicates that the state should stop running, and
/// contains the value that should be returned. 
/// 
/// The return value can be whatever makes sense for the state, and the type of the value is defined by
/// [`State::Out`]. 
/// 
/// To allow the return value to be moved from the state (e.g., when the return value is a field of the state
/// struct), [`State::event`] consumes `self`. The consumed `self` is then yielded back to [`State::run`] via
/// [`Signal::Continue`], representing the "continuation" of the state. 
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
/// # Examples 
/// 
/// A state with a tally that increases when the user presses `up`: 
/// 
/// ```no_run
/// use ratatui::widgets::Paragraph;
/// use tundra::prelude::*;
/// 
/// struct Tally {
///     value: u32, 
/// }
/// 
/// impl State for Tally {
///     type Result<T> = T;
///     type Out = u32;
///     type Global = ();
///     
///     fn draw(&self, frame: &mut Frame) {
///         let widget = Paragraph::new(self.value.to_string());
///         frame.render_widget(widget, frame.size());
///     }
///     
///     fn input(mut self, key: KeyEvent, ctx: &mut Context) -> Signal<Self> {
///         match key.code {
///             KeyCode::Up    => self.value += 1, 
///             KeyCode::Tab   => self.value *= tally(ctx), 
///             KeyCode::Enter => return Signal::Return(self.value), 
///             _ => (), 
///         }
///         Signal::Continue(self)
///     }
/// }
/// 
/// // a wrapper for the state that constructs the tally and runs it -- a recommended pattern!
/// pub fn tally(ctx: &mut Context) -> u32 {
///     Tally{ value: 0 }.run(ctx)
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

    /// Type of the value to be returned from [`State::run`] once the state has finished running. The value
    /// being returned is given by [`Signal::Return`] from [`State::event`]. 
    type Out;

    /// Type of the application-defined global inside [`Context`]. This should be set to the same type as the
    /// one used when initializing the [`Context`]. If no global is used, this may be set to `()`. 
    type Global;

    /// Draw the state to a [`Frame`]. See [Ratatui's documentation](ratatui) for how to construct and render
    /// widgets. 
    fn draw(&self, frame: &mut Frame);
    
    /// Update the state with a key press input. This is called by the default implementation of
    /// [`State::event`] when a key input event is read. 
    /// 
    /// 
    /// # Default
    /// 
    /// Always returns `Signal::Continue(self)`. The default implementation is provided for states that
    /// instead choose to implement [`State::event`]. 
    #[allow(unused_variables)]
    fn input(self, key: KeyEvent, ctx: &mut Context<Self::Global>) -> Self::Result<Signal<Self>> {
        ResultLike::from_result(Ok(Signal::Continue(self)))
    }

    /// Update the state with an event. This is called by the default implementation of [`State::run`] when
    /// an event is read. 
    /// 
    /// 
    /// # Default
    /// 
    /// Simply delegates key press events to [`State::input`], representing the most common use case. All
    /// other events are discarded. States that only care about key press events should implement
    /// [`State::input`] instead. 
    fn event(self, event: Event, ctx: &mut Context<Self::Global>) -> Self::Result<Signal<Self>> {
        if let Event::Key(key_event) = event {
            self.input(key_event, ctx)
        } else {
            ResultLike::from_result(Ok(Signal::Continue(self)))
        }
    }

    /// Enters the event loop. 
    /// 
    /// 
    /// # Default
    /// 
    /// Calls [`State::draw`] and [`State::event`] until the latter returns [`Signal::Return`]. 
    /// 
    /// 
    /// # Panics
    /// 
    /// When [`ratatui::Terminal::draw`] or [`crossterm::event::read`](event::read()) fails. 
    fn run(mut self, ctx: &mut Context<Self::Global>) -> Self::Result<Self::Out>
    where
        Error<Self, Self::Out>: From<Error<Self, Signal<Self>>>
    {
        let result = loop {
            // we're intentionally panicking on `io::Error` here to simplify application code (we would
            // otherwise have to force the application-defined error to implement `From<io::Error>`). these
            // errors should be extremely rare and only occur in extraneous circumstances. applications that
            // wish to handle `io::Error` explicitly can override `State::run` to do so
            ctx.draw_state(&self).unwrap();
            let event = event::read().unwrap();

            // generalized version of `let signal = self.event(...)?`
            let result = self.event(event, ctx);
            let signal = match ResultLike::into_result(result) {
                Ok(signal) => signal, 
                Err(err) => break Err(err.into()), 
            };
            
            match signal {
                Signal::Return(out) => break Ok(out), 
                Signal::Continue(new_self) => self = new_self, 
            }
        };
        ResultLike::from_result(result)
    }
}

/// Implements a dummy (or no-op) [`State`] through `()`. It draws nothing and exits as soon as a key is
/// pressed. 
/// 
/// This is useful when a state is expected but not used; e.g. if you want to display a [`dialog`] without a
/// background. 
impl State for () {
    type Result<T> = T;
    type Out = ();
    type Global = ();

    fn draw(&self, _frame: &mut Frame) {
        ()
    }

    fn input(self, _key: KeyEvent, _ctx: &mut Context) -> Signal<Self> {
        Signal::Return(())
    }
}

/// Generalisation over data-carrying [`Result`]-like types. 
/// 
/// There are three significant implementors of this trait: 
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
/// the same error type regardless of its value type `T` (as is true for all three implementors listed above)
/// This means that to propogate an error from `State::Result<T>` to `State::Result<U>`, an explicit bound to
/// assert that the conversion between the two (ostensibly distinct) error types exists must be added. This
/// is cumbersome for generic code (like the default implementation of [`State::run`]), but has no bearing on
/// the concrete implementations of the states themselves. 
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
