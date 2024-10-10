use std::borrow::Cow;
use ratatui::{style::{Style, Stylize}, text::{Line, Span, Text}};
use crate::prelude::*;
use super::*;

/// An [input field](super) for selecting one item among a set. 
/// 
/// The value is the index of the selected item. See [`radio::Builder`] for the methods available when
/// constructing the field. 
/// 
/// 
/// # Key bindings
/// 
/// [`KeyCode::Up`] and [`KeyCode::Down`] move the focused item up and down, respectively. Any other key sets
/// the focused item to the selected one. 
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Radio {
    /// The user-visible name displayed by the input field. 
    pub name: Cow<'static, str>, 
    /// The user-visible names of the items that can chosen between. 
    pub items: Vec<Cow<'static, str>>, 
    /// Index of the currently selected item. 
    selected: usize, 
}

impl Radio {
    /// Maximum possible index of the selected item. Defined for explicitness. 
    fn max_selected(&self) -> usize {
       self.items.len() - 1 
    }
}

impl Field for Radio {
    type Value = usize;
    type Builder = Builder;

    fn name(&self) -> &str {
        &self.name
    }

    fn input(&mut self, key: KeyEvent) -> InputResult {
        match key.code {
            // move selected item left/right
            KeyCode::Left => {
                self.selected = self.selected
                    .checked_sub(1)
                    .unwrap_or(self.max_selected());
                InputResult::Updated
            }
            KeyCode::Right => {
                self.selected = if self.selected == self.max_selected() {
                    0
                } else {
                    self.selected + 1
                };
                InputResult::Updated
            }
            _ => InputResult::Ignored, 
        }
    }

    fn format(&self, focused: bool) -> Text {
        let value = self.items[self.selected].to_string();
        let style = match focused {
            true => Style::new().bold(), 
            false => Style::new(), 
        };
        Line::from(vec![
            Span::from("<"), 
            Span::styled(value, style), 
            Span::from(">"), 
        ]).into()
    }

    fn value(&self) -> &Self::Value {
        &self.selected
    }

    fn into_value(self) -> Self::Value {
        self.selected
    }
}

/// Constructs a [`Radio`]. 
/// 
/// This is mainly used by the [form macro](crate::dialog::form!) when instantiating radios, but may also
/// be used in application code for creating a stand-alone field. 
/// 
/// Requires that both [`Builder::name`] and [`Builder::items`] are called before the field can be built. 
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Builder<const NAME: bool = false, const ITEMS: bool = false>(Radio);

impl Default for Builder {
    fn default() -> Self {
        Self(Radio {
            name: Default::default(), 
            items: Default::default(), 
            selected: 0, 
        })
    }
}

impl<const NAME: bool, const ITEMS: bool> Builder<NAME, ITEMS> {
    /// The user-visible name displayed by the input field. 
    pub fn name(self, name: impl Into<Cow<'static, str>>) -> Builder<true, ITEMS> {
        let name = name.into();
        Builder(Radio{ name, ..self.0 })
    }

    /// The user-visible names of all items that can be chosen between. 
    /// 
    /// 
    /// # Panics
    /// 
    /// When the number of items is zero. 
    pub fn items<T>(self, items: impl IntoIterator<Item = T>) -> Builder<NAME, true>
    where
        T: Into<Cow<'static, str>>, 
    {
        let items: Vec<_> = items
            .into_iter()
            .map(Into::into)
            .collect();
        debug_assert!(!items.is_empty());

        Builder(Radio{ items, ..self.0 })
    }
}

impl<const NAME: bool> Builder<NAME, true> {
    /// The index of the currently selected item. 
    pub fn selected(self, index: usize) -> Self {
        let selected = index;
        Builder(Radio{ selected, ..self.0 })
    }
}

impl Build for Builder<true, true> {
    type Field = Radio;

    fn build(self) -> Self::Field {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::{prelude::*, field::*};

    #[test]
    fn input() {
        let input = |key: KeyCode, radio: &mut Radio, expected: InputResult| {
            let actual = radio.input(key.into());
            assert_eq!(actual, expected);
        };

        let radio = &mut Radio::builder()
            .name("")
            .items(["One", "Two", "Three", "Four"])
            .selected(0)
            .build();
        assert_eq!(radio.selected, 0);

        input(KeyCode::Left, radio, InputResult::Updated);
        assert_eq!(radio.selected, 3);

        input(KeyCode::Left, radio, InputResult::Updated);
        assert_eq!(radio.selected, 2);

        input(KeyCode::Left, radio, InputResult::Updated);
        assert_eq!(radio.selected, 1);

        for i in 2..=3 {
            input(KeyCode::Right, radio, InputResult::Updated);
            assert_eq!(radio.selected, i);
        }

        input(KeyCode::Right, radio, InputResult::Updated);
        assert_eq!(radio.selected, 0);
    }
}
