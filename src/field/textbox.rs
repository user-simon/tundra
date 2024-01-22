use std::borrow::Cow;
use ratatui::prelude::*;
use crate::prelude::*;
use super::{*, builder::*};

/// An input [field](super) for entering single-line strings. 
/// 
/// 
/// # Hidden input
/// 
/// The entered value can be hidden with [`Textbox::hidden`] or [`Builder::hidden`]. When this is toggled,
/// all entered characters are replaced with `•` when the textbox is drawn. 
/// 
/// 
/// # Key bindings
/// 
/// [`KeyCode::Left`] and [`KeyCode::Right`] move the caret one character to the left and right, 
/// respectively. If [`KeyModifiers::CONTROL`] is held, the caret moves one word in the given direction. 
/// 
/// [`KeyCode::Home`] and [`KeyCode::End`] move the caret to the beginning and end of the input string,
/// respectively. 
/// 
/// [`KeyCode::Backspace`] and [`KeyCode::Delete`] remove one character from the left and right of the caret,
/// respectively. If [`KeyModifiers::CONTROL`] is held, one whole word is removed in the given direction. 
/// 
/// [`KeyCode::Char`] inputs are inserted into the input string directly after the caret. 
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Textbox {
    /// The user-visible name displayed by the input field. 
    pub name: Cow<'static, str>, 
    /// Whether the input should be hidden. See the [type-level](Textbox#hidden-input) documentation for more
    /// information.
    pub hidden: bool, 
    /// The current user-entered value. 
    value: String, 
    /// The *byte* index of the currently highlighted char. This may differ from the *char* index due to
    /// UTF-8. To maintain this invariance, `caret` and `value` are not directly modifiable by application
    /// code. 
    caret: usize, 
}

impl Textbox {
    /// Sets the current value. 
    pub fn set_value(&mut self, value: impl Into<String>) {
        self.value = value.into();
        self.caret = self.max_caret();
    }

    /// Gets the current value. 
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Splits the current value into three slices: before the caret, the caret itself, and after the caret. 
    fn split_caret(&self) -> [&str; 3] {
        let (a, b) = self.value.split_at(self.caret);
        let (b, c) = b.chars()
            .nth(0)
            .map(|first| b.split_at(first.len_utf8()))
            .unwrap_or(("", ""));
        [a, b, c]
    }

    /// The maximum possible index for the caret, given the current value. Defined for explicitness. Note
    /// that the caret can go one char out of bounds to the right where the next symbol is to be inserted. 
    fn max_caret(&self) -> usize {
        self.value.len()
    }

    /// Finds the byte index of the unicode char one step from the caret in the given direction. 
    fn step(&self, direction: Direction) -> usize {
        let [pre, caret, _] = self.split_caret();
        match direction {
            Direction::Left => pre.chars()
                .nth_back(0)
                .map(|last| self.caret - last.len_utf8())
                .unwrap_or(0),
            Direction::Right => self.caret + caret.len(),
        }
    }

    /// Finds the next word-boundary from the caret in the given direction. This is defined as the first
    /// occurence of a whitespace following a non-whitespace symbol. When `self.hidden == true`, all internal
    /// word-boundaries are ignored; either `0` or [`self.max_caret()`](Textbox::max_caret) is returned. 
    fn scan(&self, direction: Direction) -> usize {
        let [pre, caret, post] = self.split_caret();
        let (string, fallback) = match direction {
            Direction::Left  => (pre,  0), 
            Direction::Right => (post, self.max_caret()), 
        };
        
        if self.hidden {
            return fallback
        }

        // finds the next word-boundary in an iterator of char indices (which may be reversed for
        // Direction::Left) 
        fn iter(mut it: impl Iterator<Item = (usize, char)>, mut prev_ws: bool) -> Option<usize> {
            it.find_map(|(index, curr)| {
                let curr_ws = curr.is_whitespace();
                let valid = !prev_ws && curr_ws;
                prev_ws = curr_ws;
                valid.then_some(index)
            })
        }
        let chars = string.char_indices();
        let index = match direction {
            Direction::Left => iter(chars.rev(), true), 
            Direction::Right => iter(chars, caret
                    .chars()
                    .nth_back(0)
                    .map_or(false, char::is_whitespace)
                )
                .map(|index| index + self.caret + caret.len()), 
        };
        index.unwrap_or(fallback)
    }
}

impl Field for Textbox {
    type Value = String;
    type Builder = Builder<false>;

    fn name(&self) -> &str {
        &self.name
    }

    fn input(&mut self, key: KeyEvent) -> InputResult {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let (new_caret, result) = match (key.code, ctrl) {
            // move caret one char
            (KeyCode::Left,  false) => (self.step(Direction::Left), InputResult::Consumed), 
            (KeyCode::Right, false) => (self.step(Direction::Right), InputResult::Consumed), 

            // move caret one word
            (KeyCode::Left,  true) => (self.scan(Direction::Left), InputResult::Consumed), 
            (KeyCode::Right, true) => (self.scan(Direction::Right), InputResult::Consumed), 

            // move caret to beginning/end of input
            (KeyCode::Home, _) => (0, InputResult::Consumed), 
            (KeyCode::End,  _) => (self.max_caret(), InputResult::Consumed), 

            // remove char
            (KeyCode::Backspace, false) if self.caret > 0 => {
                let new = self.step(Direction::Left);
                self.value.remove(new);
                (new, InputResult::Updated)
            }
            (KeyCode::Delete, false) if self.caret < self.max_caret() => {
                self.value.remove(self.caret);
                (self.caret, InputResult::Updated)
            }

            // remove word
            (KeyCode::Backspace | KeyCode::Char('w'), true) if self.caret > 0 => {
                let end = self.scan(Direction::Left);
                self.value.drain(end..self.caret);
                (end, InputResult::Updated)
            }
            (KeyCode::Delete | KeyCode::Char('d'), true) if self.caret < self.max_caret() => {
                let end = self.scan(Direction::Right);
                self.value.drain(self.caret..end);
                (self.caret, InputResult::Updated)
            }

            // insert char
            (KeyCode::Char(c), false) => {
                self.value.insert(self.caret, c);
                (self.caret + c.len_utf8(), InputResult::Updated)
            }
            _ => (self.caret, InputResult::Ignored), 
        };
        self.caret = new_caret;
        result
    }

    fn format(&self, focused: bool) -> Text {
        // hides the contents if `self.hidden == true`; clones them otherwise
        let visibility = match self.hidden {
            true => |s: &str| s.chars()
                .map(|_| '•')
                .collect(),
            false => ToOwned::to_owned, 
        };

        match focused {
            true => {
                let [pre, caret, post] = self.split_caret().map(visibility);
                let caret = match caret.is_empty() {
                    true => " ".to_owned(),
                    false => caret,
                };
                Line::from(vec![
                    Span::raw(pre), 
                    Span::styled(caret, Style::new().reversed()), 
                    Span::raw(post), 
                ]).into()
            }
            false => {
                visibility(&self.value).into()
            }
        }
    }

    fn value(&self) -> &String {
        &self.value
    }

    fn into_value(self) -> String {
        self.value
    }
}

/// Constructs a [`Textbox`]. 
/// 
/// This is mainly used by the [form macro](crate::dialog::form!) when instantiating textboxes, but may also
/// be used in application code for creating a stand-alone field. 
/// 
/// Requires that [`Builder::name`] is called before the field can be built. 
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Builder<const NAME: bool>(Textbox);

impl Default for Builder<false> {
    fn default() -> Self {
        Self(Textbox {
            name: Default::default(),
            value: Default::default(),
            hidden: false,
            caret: 0,
        })
    }
}

impl<const NAME: bool> Builder<NAME> {
    /// The user-visible name displayed by the input field. 
    pub fn name(self, name: impl Into<Cow<'static, str>>) -> Builder<true>
    where
        Defined<NAME>: False, 
    {
        let name = name.into();
        Builder(Textbox{ name, ..self.0 })
    }

    /// The initial value. 
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.0.set_value(value);
        self
    }

    /// Hides the input. See the [type-level](Textbox#hidden-input) documentation for more information.
    pub fn hidden(self) -> Self {
        Builder(Textbox{ hidden: true, ..self.0 })
    }

    /// If the name has been defined with [`Builder::name`], consumes the builder and returns the constructed
    /// [`Textbox`]. 
    pub fn build(self) -> Textbox
    where
        Defined<NAME>: True, 
    {
        self.0
    }
}

/// Used to specify the direction of a movement relative to the caret. 
enum Direction {
    Left, 
    Right, 
}
