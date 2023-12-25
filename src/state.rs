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

/// Trivially implements a no-op [`State`] through `()`. 
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
