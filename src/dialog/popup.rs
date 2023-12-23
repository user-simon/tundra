//! Defines simple, mainly informational dialogs. 
//! 
//! The following dialogs are defined in this module: 
//! - [`dialog::confirm`] asks the user to confirm an action before proceeding. 
//! - [`dialog::info`] displays a message. 
//! - [`dialog::warning`] displays a warning. 
//! - [`dialog::error`] displays an error. 
//! - [`dialog::fatal`] displays a fatal error. 

use std::io;
use super::*;

/// Displays a yellow dialog asking the user to confirm an action before proceeding. 
/// 
/// 
/// # Returns
/// 
/// - `true` if the user pressed `y`. 
/// - `false` if the user pressed `n` or `escape`. 
pub fn confirm<G>(msg: impl AsRef<str>, over: &impl State, ctx: &mut Context<G>) -> io::Result<bool> {
    Confirm{ msg: msg.as_ref() }
        .run_over(over, ctx)
        .map(|x| x.is_some())
}

/// Displays a blue dialog showing a message. 
pub fn info<G>(msg: impl AsRef<str>, over: &impl State, ctx: &mut Context<G>) -> io::Result<()> {
    message(msg.as_ref(), MessageLevel::Info, over, ctx)
}

/// Displays a yellow dialog showing a warning. 
pub fn warning<G>(msg: impl AsRef<str>, over: &impl State, ctx: &mut Context<G>) -> io::Result<()> {
    message(msg.as_ref(), MessageLevel::Warning, over, ctx)
}

/// Displays a red dialog showing an error message. 
pub fn error<G>(msg: impl AsRef<str>, over: &impl State, ctx: &mut Context<G>) -> io::Result<()> {
    message(msg.as_ref(), MessageLevel::Error, over, ctx)
}

/// Displays a red dialog showing a fatal error message. 
/// 
/// No background state is drawn upon displaying a fatal error message. 
pub fn fatal<G>(msg: impl AsRef<str>, ctx: &mut Context<G>) -> io::Result<()> {
    message(msg.as_ref(), MessageLevel::Fatal, &(), ctx)
}

/// Displays a dialog showing a message of specified [level](MessageLevel). 
fn message<G>(msg: &str, level: MessageLevel, over: &impl State, ctx: &mut Context<G>) -> io::Result<()> {
    Message{ msg, msg_level: level }
        .run_over(over, ctx)
        .map(|_| ())
}

/// Defines the title and colour of a [`Message`] dialog. 
enum MessageLevel {
    Info, 
    Warning, 
    Error, 
    Fatal, 
}

/// Dialog to simply show a message to the user. 
struct Message<'a> {
    msg: &'a str, 
    msg_level: MessageLevel, 
}

impl Dialog for Message<'_> {
    fn format(&self) -> DrawInfo {
        let (title, color) = match self.msg_level {
            MessageLevel::Info    => ("Info",        Color::Cyan),
            MessageLevel::Warning => ("Warning",     Color::Yellow),
            MessageLevel::Error   => ("Error",       Color::Red),
            MessageLevel::Fatal   => ("Fatal error", Color::Red),
        };
        DrawInfo {
            title: title.into(), 
            color, 
            body: self.msg.into(), 
            hint: "Press any key to continue...".into(),
        }
    }

    fn input(&mut self, _key: KeyEvent) -> Signal {
        Signal::Done
    }
}

/// Dialog to confirm an action before proceeding. 
struct Confirm<'a> {
    msg: &'a str, 
}

impl Dialog for Confirm<'_> {
    fn format(&self) -> DrawInfo {
        DrawInfo {
            title: "Confirm".into(),
            color: Color::Yellow,
            body: self.msg.into(),
            hint: "Press (y) to confirm, (n) or (esc) to cancel...".into(),
        }
    }

    fn input(&mut self, key: KeyEvent) -> Signal {
        match key.code {
            KeyCode::Char('y') |
            KeyCode::Char('Y') => Signal::Done,
            KeyCode::Esc       |
            KeyCode::Char('n') |
            KeyCode::Char('N') => Signal::Cancelled,
            _ => Signal::Running,
        }
    }
}
