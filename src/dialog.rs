//! TODO

use std::{
    io, 
    borrow::Cow, 
};
use ratatui::{
    style::{Color, Stylize, Style}, 
    text::{Text, StyledGrapheme}, 
    widgets::*, 
    prelude::{Rect, Layout, Constraint}
};
use crate::prelude::*;

/// Content displayed inside a dialog pop-up covering the middle of some background [state](crate::State). 
/// 
/// For most applications, the [library provided dialogs](dialog) should suffice, but it is possible to 
/// create custom dialogs as well. 
/// 
/// This essentially serves as a wrapper over [`State`] to provide the rendering of the dialog box and 
/// background state. 
/// 
/// # Examples
/// Creating a custom confirmation dialog (this is more or less the same as the one provided through 
/// [`dialog::confirm`]): 
/// ```
/// # use std::io;
/// # use tundra::{prelude::*, dialog::*};
/// # use ratatui::style::Color;
/// struct Confirm {
///     msg: String, 
/// }
/// 
/// impl Dialog for Confirm {
///     fn format(&self) -> DrawInfo {
///         DrawInfo {
///             title: "Confirm".into(), 
///             color: Color::Yellow, 
///             body: self.msg.clone().into(), 
///             hint: "Press (y) to confirm, (n) to cancel...".into(), 
///         }
///     }
/// 
///     fn input(&mut self, key: KeyEvent) -> Signal {
///         match key.code {
///             KeyCode::Char('y') => Signal::Done,
///             KeyCode::Char('n') => Signal::Cancelled,
///             _ => Signal::Running,
///         }
///     }
/// }
/// 
/// // convenience wrapper over `Dialog::run_over`, providing a more bespoke interface
/// fn confirm(msg: String, background: &impl State, ctx: &mut Context) -> io::Result<bool> {
///     Confirm{ msg }
///         .run_over(background, ctx)
///         .map(|x| x.is_some())
/// }
/// 
/// # let current_state = ();
/// # let ctx = &mut Context::new().unwrap();
/// 
/// // let current_state = ...
/// // let ctx = ...
/// 
/// let msg = "Please confirm before proceeding";
/// let confirmed = confirm(msg.into(), &current_state, ctx).unwrap();
/// 
/// match confirmed {
///     true => { /* ... */ }, 
///     false => { /* ... */ }, 
/// }
/// ```
pub trait Dialog: Sized {
    /// Defines the information needed to render. See [`DrawInfo`] for the required fields. 
    fn format(&self) -> DrawInfo;
    
    /// Update the dialog with a key press input. 
    fn input(&mut self, key: KeyEvent) -> Signal;

    /// Runs the dialog to fruition over some background state. 
    /// 
    /// This is a wrapper over [`State::run`] with added logic to render the dialog box and background
    /// state. 
    fn run_over<G>(self, background: &impl State, ctx: &mut Context<G>) -> io::Result<Option<Self>> {
        Container{ content: self, background }
            .run(&mut ctx.chain_without_global())
            .map(|x| x.map(|x| x.content))
    }
}

/// Defines the content displayed within a dialog, and associated meta-data. 
/// 
/// This is interpreted by the dialog state when rendering. 
#[derive(Clone, Debug)]
pub struct DrawInfo<'a> {
    /// User-visible title of the dialog box. 
    pub title: Cow<'a, str>, 
    /// Colour of the entire dialog. 
    pub color: Color, 
    /// Dialog payload. 
    pub body: Text<'a>, 
    /// String displayed at the bottom in italics, for example for displaying the dialog key binds. 
    pub hint: Cow<'a, str>,
}

/// Displays a dialog asking the user to confirm an action before proceeding. 
/// 
/// # Returns
/// `true` if the user pressed `y`. 
/// `false` if the user pressed `n` or `escape`. 
pub fn confirm<G>(msg: impl AsRef<str>, over: &impl State, ctx: &mut Context<G>) -> io::Result<bool> {
    Confirm{ msg: msg.as_ref() }
        .run_over(over, ctx)
        .map(|x| x.is_some())
}

/// Displays a dialog showing a message. 
pub fn info<G>(msg: impl AsRef<str>, over: &impl State, ctx: &mut Context<G>) -> io::Result<()> {
    message(msg.as_ref(), MessageLevel::Info, over, ctx)
}

/// Displays a dialog showing a warning. 
pub fn warning<G>(msg: impl AsRef<str>, over: &impl State, ctx: &mut Context<G>) -> io::Result<()> {
    message(msg.as_ref(), MessageLevel::Warning, over, ctx)
}

/// Displays a dialog showing an error message. 
pub fn error<G>(msg: impl AsRef<str>, over: &impl State, ctx: &mut Context<G>) -> io::Result<()> {
    message(msg.as_ref(), MessageLevel::Error, over, ctx)
}

/// Displays a dialog showing a fatal error message. 
pub fn fatal<G>(msg: impl AsRef<str>, ctx: &mut Context<G>) -> io::Result<()> {
    message(msg.as_ref(), MessageLevel::Fatal, &(), ctx)
}

/// Displays a dialog showing a message of specified [level](MessageLevel). 
fn message<G>(msg: &str, level: MessageLevel, over: &impl State, ctx: &mut Context<G>) -> io::Result<()> {
    Message{ msg, msg_level: level }
        .run_over(over, ctx)
        .map(|_| ())
}

/// This represents the dialog box and serves as the common [`State`] implementation for all
/// [dialogs](Dialog). 
/// 
/// It is responsible for rendering the dialog box, dialog contents, and background state. 
struct Container<'a, T, U> {
    /// The [dialog](Dialog), proper. 
    content: T, 
    /// Background state. 
    background: &'a U, 
}

impl<T: Dialog, U: State> State for Container<'_, T, U> {
    type Error = io::Error;
    type Global = ();

    fn draw(&self, frame: &mut Frame) {
        self.background.draw(frame);
        let draw_info = self.content.format();

        // factored out non-generic code to reduce code generation
        draw_dialog(draw_info, frame)
    }

    fn input(&mut self, key: KeyEvent, _ctx: &mut Context) -> io::Result<Signal> {
        Ok(self.content.input(key))
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

fn draw_dialog(info: DrawInfo, frame: &mut Frame) {
    const HORIZONTAL_MARGIN: u16 = 3;
    const VERTICAL_MARGIN: u16 = 1;
    const WIDTH_FACTOR: f32 = 0.5;

    let DrawInfo{ title, body, color, hint } = info;

    let wrap = Wrap{ trim: false };
    let hint = Paragraph::new(hint).wrap(wrap);
    let body = Paragraph::new(body).wrap(wrap);

    let frame_size = frame.size();
    let inner_width = (frame_size.width as f32 * WIDTH_FACTOR) as u16;

    let [hint_height, body_height] = [&hint, &body].map(|x|
        x.line_count(inner_width) as u16
    );

    // draw box and compute its inner area
    let inner_area = {
        let inner_height = body_height + hint_height + 1;
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", title.to_uppercase()))
            .border_type(BorderType::Thick)
            .fg(color);
        let [outer_width, outer_height] = outer_size(
            &block, 
            inner_width + HORIZONTAL_MARGIN * 2, 
            inner_height + VERTICAL_MARGIN * 2, 
        );

        let Rect{ width: frame_width, height: frame_height, .. } = frame_size;
        let outer_area = Layout::default()
            .constraints([Constraint::Min(0)])
            .horizontal_margin(frame_width.saturating_sub(outer_width) / 2)
            .vertical_margin(frame_height.saturating_sub(outer_height) / 2)
            .split(frame_size)[0];
        let inner_area = block.inner(outer_area);

        frame.render_widget(Clear, outer_area);
        frame.render_widget(block, outer_area);

        inner_area
    };

    // draw body and hint inside the inner area
    {
        let [body_area, hint_area] = {
            let layout = Layout::default()
                .horizontal_margin(HORIZONTAL_MARGIN)
                .vertical_margin(VERTICAL_MARGIN)
                .constraints([
                    Constraint::Length(body_height), 
                    Constraint::Min(0), 
                    Constraint::Length(hint_height), 
                ])
                .split(inner_area);
            [layout[0], layout[2]]
        };
    
        frame.render_widget(body, body_area);
        frame.render_widget(hint, hint_area);
    }
}

fn outer_size(block: &Block, inner_width: u16, inner_height: u16) -> [u16; 2] {
    let dummy = Rect::new(0, 0, u16::MAX, u16::MAX);
    let Rect{ width, height, .. } = block.inner(dummy);
    let [dx, dy] = [
        dummy.width - width, 
        dummy.height - height, 
    ];
    [inner_width + dx, inner_height + dy]
}
