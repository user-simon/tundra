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
pub trait State {
    type Error;

    /// Type of the application-defined global inside [`Context`]. This should be set to the same type as the
    /// one used when initializing the [`Context`]. If no global is used, this may be set to `()`. 
    type Global;

    /// Draw the state to a [`Frame`]. 
    fn draw(&self, frame: &mut Frame);
    
    /// Update the state with a key press input. 
    fn input(&mut self, key: KeyEvent, ctx: &mut Context<Self::Global>) -> Result<Signal, Self::Error>;

    /// The event loop. 
    /// 
    /// Calls [`State::draw`] and [`State::input`] until the latter returns [`Signal::Done`] or
    /// [`Signal::Cancelled`]. 
    /// 
    /// Should be called recursively for state transtitions; e.g. `<A as State>::input` may call
    /// `<B as State>::run` to transition to the `B` state, thereby preserving the state history on the call
    /// stack. 
    /// 
    /// For brevity, this may be wrapped by functions representing individual states, which provide a more
    /// bespoke interface. E.g. [`dialog::confirm`](crate::dialog::confirm), which creates a confirm dialog
    /// state, runs it, and then returns whether the user pressed `y` or `n`. 
    /// 
    /// # Returns
    /// - `Some(self)` if [`State::input`] returns [`Signal::Done`]. 
    /// - `None` if [`State::input`] returns [`Signal::Cancelled`]. 
    fn run(mut self, ctx: &mut Context<Self::Global>) -> Result<Option<Self>, Self::Error>
    where
        Self: Sized, 
        Self::Error: From<io::Error>, 
    {
        loop {
            ctx.draw_state(&self)?;

            if let Event::Key(key_event) = event::read()? {
                match self.input(key_event, ctx)? {
                    Signal::Done      => break Ok(Some(self)),
                    Signal::Cancelled => break Ok(None),
                    Signal::Running   => (),
                }
            }
        }
    }
}

/// Trivially implements a no-op [`State`] through `()`. 
/// 
/// This is useful when a state is expected but not used; e.g. if you want to display a
/// [`dialog`](crate::dialog) without a background. 
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
