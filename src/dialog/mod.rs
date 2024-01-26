//! Modal dialogs displayed in the middle of the screen, covering some background [state](crate::State). 
//! 
//! The following dialogs are defined in this module: 
//! - [`dialog::confirm`] asks the user to confirm an action before proceeding. 
//! - [`dialog::select`] asks the user to select one action among a set. 
//! - [`dialog::info`] displays a message. 
//! - [`dialog::warning`] displays a warning. 
//! - [`dialog::error`] displays an error. 
//! - [`dialog::fatal`] displays a fatal error. 
//! - [`dialog::form!`] allows the user to enter information through a set of input fields. 
//! 
//! 
//! # Custom dialogs
//! 
//! Custom dialogs may be created by implementing the [`Dialog`] trait. See its documentation for more
//! information. 
//! 
//! 
//! # Examples
//! 
//! To show a dialog without any background, provide the [dummy state](crate::State#dummy-state) `()`: 
//! ```no_run
//! # use tundra::prelude::*;
//! # let ctx = &mut Context::new().unwrap();
//! // let ctx: &mut Context<_>
//! dialog::info("Shown without a background!", &(), ctx);
//! ```

mod basic;
pub mod form;

use std::borrow::Cow;
use ratatui::{
    Frame, 
    style::{Color, Stylize}, 
    text::Text, 
    widgets::{*, block::Title}, 
    layout::{Rect, Layout, Constraint, Margin}, 
};
use crate::{prelude::*, Never};

pub use basic::*;
pub use form::form;

/// Interface for content displayed inside a dialog. 
/// 
/// For most applications, the [library provided dialogs](dialog) should suffice, but custom dialogs may be
/// created by implementing this trait. 
/// 
/// This essentially serves as a wrapper over [`State`] to provide the drawing of the dialog box and 
/// background state. 
/// 
/// 
/// # Examples
/// 
/// Creating a custom confirmation dialog (this is more or less the same as the one provided through 
/// [`dialog::confirm`]): 
/// ```no_run
/// use ratatui::style::Color;
/// use tundra::{prelude::*, dialog::{Dialog, DrawInfo}};
/// 
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
///             ..Default::default()
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
/// fn confirm(msg: String, background: &impl State, ctx: &mut Context) -> bool {
///     Confirm{ msg }
///         .run_over(background, ctx)
///         .is_some()
/// }
/// 
/// # let current_state = &();
/// # let ctx = &mut Context::new().unwrap();
/// // let current_state: &impl State
/// // let ctx: &mut Context<_>
/// 
/// let msg = "Please confirm before proceeding";
/// let confirmed: bool = confirm(msg.into(), current_state, ctx);
/// ```
pub trait Dialog: Sized {
    /// Defines the information needed to draw the dialog. See [`DrawInfo`] for the required fields. 
    fn format(&self) -> DrawInfo;
    
    /// Update the dialog with a key press input. 
    fn input(&mut self, key: KeyEvent) -> Signal;

    /// Runs the dialog to fruition over some background state. 
    /// 
    /// This is a wrapper over [`State::run`] with added logic to draw the dialog box and background
    /// state. 
    fn run_over<G>(self, background: &impl State, ctx: &mut Context<G>) -> Option<Self> {
        Container{ content: self, background }
            .run(&mut ctx.chain_without_global())
            .map(|container| container.content)
    }
}

/// Defines how to draw a dialog and its contents. 
/// 
/// This is returned from [`Dialog::format`] and is interpreted by the dialog state when drawing. 
/// 
/// Note that most (though not all) variables used when drawing dialogs are factored out in this struct for
/// flexibility --- many of which are likely not relevant for most dialogs. In these cases, set the required
/// variables and defer to the default implementation for the remainder. 
/// 
/// 
/// # Examples
/// 
/// To draw a red dialog with title "Attention!", body "You are an ugly boy.", and hint "Press any key to
/// accept...": 
/// ```no_run
/// # use ratatui::style::Color;
/// # use tundra::dialog::DrawInfo;
/// # let _ = 
/// DrawInfo {
///     title: "Attention!".into(), 
///     color: Color::Red, 
///     body: "You are an ugly boy.".into(), 
///     hint: "Press any key to accept...".into(), 
///     ..Default::default()
/// }
/// # ;
/// ```
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DrawInfo<'a> {
    /// User-visible title of the dialog box. Default: `""`. 
    pub title: Cow<'a, str>, 
    /// Colour of the entire dialog. Default: `Color::Cyan`. 
    pub color: Color, 
    /// Dialog payload. Default: `""`. 
    pub body: Text<'a>, 
    /// String displayed at the bottom in italics, for example for displaying the dialog key binds. Default: 
    /// `""`. 
    pub hint: Cow<'a, str>, 
    /// Margin `[horizontal, vertical]` between the border and the body. Default: `[3, 1]`. 
    pub inner_margin: [u16; 2], 
    /// Width of the dialog as a percentage (between `0` and `100`) of the total width of the terminal. 
    /// Default: `50`. 
    pub width_percentage: u8, 
    /// Settings used to wrap the body [`Paragraph`]. Set to `None` to disable wrapping. Default: uses
    /// wrapping with [`Wrap::trim`] set to true. 
    pub wrap: Option<Wrap>, 
    /// Function constructing a [`Title`] from a string. Default: turns the title uppercase and inserts a
    /// space on either side of it. 
    pub create_title: fn(Cow<'a, str>) -> Title<'a>, 
    /// Function constructing the [`Block`], which represents the dialog box. Note that two properties are
    /// later overriden: 
    /// - `Block::fg()`, which is set to [`color`](DrawInfo::color). 
    /// - `Block::title()`, which is set to the output of [`create_title`](DrawInfo::create_title). 
    /// 
    /// Default: uses `Borders::ALL` and `BorderType::Thick`. 
    pub create_block: fn() -> Block<'a>, 
}

impl<'a> Default for DrawInfo<'a> {
    fn default() -> DrawInfo<'a> {
        DrawInfo {
            title: "".into(), 
            color: Color::Cyan, 
            body: "".into(), 
            hint: "".into(), 
            inner_margin: [3, 1], 
            width_percentage: 50, 
            wrap: Some(Wrap{ trim: true }), 
            create_title: |title| match title.is_empty() {
                true => "".into(), 
                false => format!(" {title} ").to_uppercase().into(), 
            }, 
            create_block: || Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Thick), 
        }
    }
}

/// This represents the dialog box and serves as the common [`State`] implementation for all
/// [dialogs](Dialog). 
/// 
/// It is responsible for rendering the dialog box, dialog contents, and background state. 
struct Container<'a, T, U> {
    /// Dialog contents. 
    content: T, 
    /// Background state. 
    background: &'a U, 
}

impl<T: Dialog, U: State> State for Container<'_, T, U> {
    type Result<V> = V;
    type Global = ();

    fn draw(&self, frame: &mut Frame) {
        self.background.draw(frame);
        let draw_info = self.content.format();

        // factored out non-generic code to reduce code generation
        draw_dialog(draw_info, frame)
    }

    fn input(&mut self, key: KeyEvent, _ctx: &mut Context) -> Signal {
        self.content.input(key)
    }
}

#[inline(never)]
fn draw_dialog<'a>(info: DrawInfo<'a>, frame: &mut Frame) {
    let DrawInfo {
        title, 
        body, 
        color, 
        hint, 
        inner_margin: [inner_margin_x, inner_margin_y], 
        width_percentage, 
        wrap, 
        create_title, 
        create_block, 
    } = info;

    // create body and hint paragraphs
    let body = match (wrap, Paragraph::new(body)) {
        (Some(wrap), body) => body.wrap(wrap), 
        (None, body) => body, 
    };
    let hint = Paragraph::new(hint)
        .wrap(Wrap{ trim: true })
        .italic();

    // compute the required inner dimensions
    let frame_size = frame.size();
    let inner_width = (frame_size.width * width_percentage as u16) / 100;
    let [hint_height, body_height] = [&hint, &body].map(|x|
        x.line_count(inner_width) as u16
    );
    let inner_height = body_height + 2 + hint_height; // 2 spaces between body and hint

    // draw box and compute its actual inner area
    let inner_area = {
        let title = create_title(title);
        let block = create_block()
            .title(title)
            .fg(color);
        let [outer_width, outer_height] = outer_size(
            &block, 
            inner_width + inner_margin_x * 2, 
            inner_height + inner_margin_y * 2, 
        );
        let [delta_width, delta_height] = [
            frame_size.width.saturating_sub(outer_width), 
            frame_size.height.saturating_sub(outer_height), 
        ];
        let mut outer_area = frame_size.inner(&Margin {
            horizontal: delta_width / 2,
            vertical: delta_height / 2,
        });

        // if the delta height is odd, the margin will be 0.5 too small on both the top and bottom. to
        // account for this, we remove 1 from the dialog height -- basically rounding the top margin down and
        // the bottom margin up
        outer_area.height -= delta_height & 1;

        let inner_area = block.inner(outer_area);

        frame.render_widget(Clear, outer_area);
        frame.render_widget(block, outer_area);

        inner_area
    };

    // draw body and hint inside the inner area
    {
        let layout = Layout::default()
            .horizontal_margin(inner_margin_x)
            .vertical_margin(inner_margin_y)
            .constraints([
                Constraint::Length(body_height), 
                Constraint::Min(0), 
                Constraint::Length(hint_height), 
            ])
            .split(inner_area);
    
        frame.render_widget(body, layout[0]);
        frame.render_widget(hint, layout[2]);
    }
}

fn outer_size(block: &Block, inner_width: u16, inner_height: u16) -> [u16; 2] {
    let dummy = Rect::new(0, 0, u16::MAX, u16::MAX);
    let Rect{ width, height, .. } = block.inner(dummy);
    let dx = dummy.width - width;
    let dy = dummy.height - height;
    [inner_width + dx, inner_height + dy]
}
