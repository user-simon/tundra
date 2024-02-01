//! Defines simple, mainly informational dialogs. 
//! 
//! The following dialogs are defined in this module: 
//! - [`dialog::confirm`] asks the user to confirm an action before proceeding. 
//! - [`dialog::select`] asks the user to select one action among a set. 
//! - [`dialog::select`] asks the user to select one action among a set. 
//! - [`dialog::info`] displays a message. 
//! - [`dialog::warning`] displays a warning. 
//! - [`dialog::error`] displays an error. 
//! - [`dialog::fatal`] displays a fatal error. 

use ratatui::text::Line;
use super::*;

/// Displays a yellow dialog asking the user to confirm an action before proceeding. 
/// 
/// 
/// # Returns
/// 
/// - `true` if the user pressed `y`. 
/// - `false` if the user pressed `n` or `escape`. 
pub fn confirm<G>(msg: impl AsRef<str>, over: &impl State, ctx: &mut Context<G>) -> bool {
    Confirm{ msg: msg.as_ref() }
        .run_over(over, ctx)
        .is_some()
}

/// Displays a blue dialog asking the user to select one action among a set. 
/// 
/// 
/// # Returns
/// 
/// - The selected index if the user pressed `enter`. 
/// - `None` if the user pressed `escape`. 
pub fn select<'a, T, U, G>(actions: T, over: &impl State, ctx: &mut Context<G>) -> Option<usize>
where
    T: AsRef<[U]>, 
    U: AsRef<str>, 
{
    Select{ actions: actions.as_ref(), selected: 0 }
        .run_over(over, ctx)
        .map(|x| x.selected)
}

/// Displays a blue dialog showing a message. 
pub fn info<G>(msg: impl AsRef<str>, over: &impl State, ctx: &mut Context<G>) {
    message(msg.as_ref(), MessageLevel::Info, over, ctx)
}

/// Displays a yellow dialog showing a warning. 
pub fn warning<G>(msg: impl AsRef<str>, over: &impl State, ctx: &mut Context<G>) {
    message(msg.as_ref(), MessageLevel::Warning, over, ctx)
}

/// Displays a red dialog showing an error message. 
pub fn error<G>(msg: impl AsRef<str>, over: &impl State, ctx: &mut Context<G>) {
    message(msg.as_ref(), MessageLevel::Error, over, ctx)
}

/// Displays a red dialog showing a fatal error message. 
/// 
/// No background state is drawn upon displaying a fatal error message. 
pub fn fatal<G>(msg: impl AsRef<str>, ctx: &mut Context<G>) {
    message(msg.as_ref(), MessageLevel::Fatal, &(), ctx)
}

/// Displays a dialog showing a message of specified [level](MessageLevel). 
fn message<G>(msg: &str, level: MessageLevel, over: &impl State, ctx: &mut Context<G>) {
    Message{ msg, level }.run_over(over, ctx);
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
            ..Default::default()
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

/// Dialog to select one action among a set. 
struct Select<'a, T> {
    actions: &'a [T], 
    selected: usize, 
}

impl<'a, T: AsRef<str>> Dialog for Select<'a, T> {
    fn format(&self) -> DrawInfo {
        let format_action = |(i, action)| {
            let prefix = match i == self.selected {
                true => '→', 
                false => ' ', 
            };
            format!("{prefix} {action}").into()
        };
        let body: Vec<Line> = self.actions
            .iter()
            .map(AsRef::as_ref)
            .enumerate()
            .map(format_action)
            .collect();
        DrawInfo {
            title: "Select".into(), 
            color: Color::Cyan, 
            body: body.into(), 
            hint: "Press (enter) to select action, (esc) to cancel...".into(), 
            wrap: Some(Wrap{ trim: false }), 
            ..Default::default()
        }
    }

    fn input(&mut self, key: KeyEvent) -> Signal {
        match key.code {
            KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
                Signal::Running
            } 
            KeyCode::Down => {
                self.selected = usize::min(self.selected + 1, self.actions.len() - 1);
                Signal::Running
            } 
            KeyCode::Esc => Signal::Cancelled, 
            KeyCode::Enter => Signal::Done, 
            _ => Signal::Running, 
        }
    }
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
    level: MessageLevel, 
}

impl Dialog for Message<'_> {
    fn format(&self) -> DrawInfo {
        let (title, color) = match self.level {
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
            ..Default::default()
        }
    }

    fn input(&mut self, _key: KeyEvent) -> Signal {
        Signal::Done
    }
}
