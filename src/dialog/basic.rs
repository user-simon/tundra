//! Defines simple, mainly informational dialogs. 
//! 
//! The following dialogs are defined in this module: 
//! - [`dialog::confirm`] asks the user to confirm an action before proceeding. 
//! - [`dialog::select_index`] asks the user to select one item among a set. 
//! - [`dialog::select_value`] asks the user to select one value among a set. 
//! - [`dialog::select_action`] asks the user to select one action among a set. 
//! - [`dialog::select_action_mut`] asks the user to select one action among a set. 
//! - [`dialog::info`] displays a message. 
//! - [`dialog::warning`] displays a warning. 
//! - [`dialog::error`] displays an error. 
//! - [`dialog::fatal`] displays a fatal error. 
//! - [`dialog::message`] displays any kind of message. 

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
    Confirm{ msg: msg.as_ref() }.run_over(over, ctx)
}

/// Displays a blue dialog asking the user to select one item among a set. 
/// 
/// 
/// # Returns
/// 
/// The selected index. 
pub fn select_index<T: AsRef<str>, G>(
    msg: impl AsRef<str>, 
    items: impl AsRef<[T]>, 
    over: &impl State, 
    ctx: &mut Context<G>, 
) -> usize {
    let labels = items.as_ref();
    let dialog = Select {
        msg: msg.as_ref(), 
        get_label: |i: usize| labels[i].as_ref(), 
        get_value: std::convert::identity, 
        item_count: labels.len(), 
        selected: 0
    };
    dialog.run_over(over, ctx)
}

/// Displays a blue dialog asking the user to select one value among a set. 
/// 
/// The items are given as an array of `(user-visible label, value)`. 
/// 
/// 
/// # Returns
/// 
/// The value associated with the item. 
pub fn select_value<'a, T, G>(
    msg: impl AsRef<str>, 
    items: &'a [(impl AsRef<str>, T)], 
    over: &impl State, 
    ctx: &mut Context<G>, 
) -> &'a T {
    let items = items.as_ref();
    let dialog = Select {
        msg: msg.as_ref(), 
        get_label: |i: usize| items[i].0.as_ref(), 
        get_value: |i: usize| &items[i].1, 
        item_count: items.len(), 
        selected: 0, 
    };
    dialog.run_over(over, ctx)
}

/// Displays a blue dialog asking the user to select one action among a set. 
/// 
/// The items are given as an array of `(user-visible label, callback)`. 
/// 
/// 
/// # Returns
/// 
/// The value returned from the selected callback. 
pub fn select_action<T, U: State, G>(
    msg: impl AsRef<str>, 
    items: &[(impl AsRef<str>, fn(state: &U, ctx: &mut Context<G>) -> T)], 
    state: &U, 
    ctx: &mut Context<G>, 
) -> T {
    select_value(msg, items.as_ref(), state, ctx)(state, ctx)
}

/// Displays a blue dialog asking the user to select one action among a set. 
/// 
/// The items are given as an array of `(user-visible label, callback)`. 
/// 
/// 
/// # Returns
/// 
/// The value returned from the selected callback. 
pub fn select_action_mut<T, U: State, G>(
    msg: impl AsRef<str>, 
    items: &[(impl AsRef<str>, fn(state: &mut U, ctx: &mut Context<G>) -> T)], 
    state: &mut U, 
    ctx: &mut Context<G>, 
) -> T {
    select_value(msg, items.as_ref(), state, ctx)(state, ctx)
}

/// Displays a blue dialog showing a message. 
pub fn info<G>(msg: impl AsRef<str>, over: &impl State, ctx: &mut Context<G>) {
    message(msg.as_ref(), "Info", Color::Cyan, over, ctx)
}

/// Displays a yellow dialog showing a warning. 
pub fn warning<G>(msg: impl AsRef<str>, over: &impl State, ctx: &mut Context<G>) {
    message(msg.as_ref(), "Warning", Color::Yellow, over, ctx)
}

/// Displays a red dialog showing an error message. 
pub fn error<G>(msg: impl AsRef<str>, over: &impl State, ctx: &mut Context<G>) {
    message(msg.as_ref(), "Error", Color::Red, over, ctx)
}

/// Displays a red dialog showing a fatal error message. 
/// 
/// No background state is drawn upon displaying a fatal error message. 
pub fn fatal<G>(msg: impl AsRef<str>, ctx: &mut Context<G>) {
    message(msg.as_ref(), "Fatal error", Color::Red, &(), ctx)
}

/// Displays a dialog showing a generic message. 
/// 
/// This is lower level than the other message dialog functions. Prefer the more specialised 
/// [`dialog::info`], [`dialog::warning`], [`dialog::error`], or [`dialog:fatal`] unless you need the 
/// customisation. 
pub fn message<G>(msg: &str, title: &str, color: Color, over: &impl State, ctx: &mut Context<G>) {
    Message{ msg, title, color }.run_over(over, ctx)
}

/// Dialog to confirm an action before proceeding. 
struct Confirm<'a> {
    msg: &'a str, 
}

impl Dialog for Confirm<'_> {
    type Out = bool;

    fn format(&self) -> DrawInfo {
        DrawInfo {
            title: "Confirm".into(), 
            color: Color::Yellow, 
            body: self.msg.into(), 
            hint: "Press (y) to confirm, (n) or (esc) to cancel...".into(), 
            ..Default::default()
        }
    }

    fn input(self, key: KeyEvent) -> Signal<Self> {
        match key.code {
            KeyCode::Char('y') |
            KeyCode::Char('Y') => Signal::Return(true), 
            KeyCode::Esc       |
            KeyCode::Char('n') |
            KeyCode::Char('N') => Signal::Return(false), 
            _ => Signal::Continue(self), 
        }
    }
}

/// Dialog to select one item among a set. 
struct Select<'a, T, U> {
    msg: &'a str, 
    get_label: T, 
    get_value: U, 
    item_count: usize, 
    selected: usize, 
}

impl<'a, T: Fn(usize) -> &'a str, U: Fn(usize) -> V, V> Dialog for Select<'a, T, U> {
    type Out = V;

    fn format(&self) -> DrawInfo {
        let format_action = |(i, action)| {
            let prefix = match i == self.selected {
                true => '→', 
                false => '·', 
            };
            format!("{prefix} {action}").into()
        };
        let labels = (0..self.item_count)
            .map(&self.get_label)
            .enumerate()
            .map(format_action);
        let body: Vec<Line> = [self.msg.into(), Line::default()]
            .into_iter()
            .chain(labels)
            .collect();
        DrawInfo {
            title: "Select".into(), 
            color: Color::Cyan, 
            body: body.into(), 
            hint: "Press (enter) to select item...".into(), 
            wrap: Some(Wrap{ trim: false }), 
            ..Default::default()
        }
    }

    fn input(mut self, key: KeyEvent) -> Signal<Self> {
        match key.code {
            KeyCode::Up => {
                self.selected = self.selected.saturating_sub(1);
            } 
            KeyCode::Down => {
                self.selected = usize::min(self.selected + 1, self.item_count - 1);
            }
            KeyCode::Enter => return Signal::Return((self.get_value)(self.selected)), 
            _ => (), 
        };
        Signal::Continue(self)
    }
}

/// Dialog to simply show a message to the user. 
struct Message<'a> {
    msg: &'a str, 
    title: &'a str, 
    color: Color, 
}

impl Dialog for Message<'_> {
    type Out = ();

    fn format(&self) -> DrawInfo {
        DrawInfo {
            title: self.title.into(), 
            color: self.color, 
            body: self.msg.into(), 
            hint: "Press any key to close...".into(), 
            ..Default::default()
        }
    }

    fn input(self, _key: KeyEvent) -> Signal<Self> {
        Signal::Return(())
    }
}
