use std::borrow::Cow;
use ratatui::text::{Text, Line};
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
    pub selected: usize, 
    /// Index of the currently focused item. 
    focus: usize, 
}

impl Field for Radio {
    type Value = usize;
    type Builder = Builder;

    fn name(&self) -> &str {
        &self.name
    }

    fn input(&mut self, key: KeyEvent) -> InputResult {
        match key.code {
            // move focused item up/down
            KeyCode::Up if self.focus > 0 => {
                self.focus -= 1;
                InputResult::Consumed
            }
            KeyCode::Down if self.focus < (self.items.len() - 1) => {
                self.focus += 1;
                InputResult::Consumed
            }

            // we are the top/bottom of the items, no change
            KeyCode::Up | KeyCode::Down => InputResult::Ignored, 

            // selected the focused item
            _ => {
                self.selected = self.focus;
                InputResult::Updated
            }
        }
    }

    fn format(&self, focused: bool) -> Text {
        let format_item = |(i, item)| {
            let symbol = match i == self.selected {
                true  => '●', 
                false => ' ', 
            };
            match focused && i == self.focus {
                true => format!("[{symbol}] {item}"), 
                false => format!("({symbol}) {item}"), 
            }
        };
        self.items
            .iter()
            .enumerate()
            .map(format_item)
            .map(Line::from)
            .collect::<Vec<_>>()
            .into()
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
            focus: 0,
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

        input(KeyCode::Char('a'), radio, InputResult::Updated);
        assert_eq!(radio.selected, 0);

        input(KeyCode::Down, radio, InputResult::Consumed);
        assert_eq!(radio.selected, 0);

        input(KeyCode::Enter, radio, InputResult::Updated);
        assert_eq!(radio.selected, 1);

        for i in 1..=2 {
            assert_eq!(radio.focus, i);
            input(KeyCode::Down, radio, InputResult::Consumed);
        }
        assert_eq!(radio.focus, 3);

        input(KeyCode::Down, radio, InputResult::Ignored);
        assert_eq!(radio.focus, 3);

        input(KeyCode::Backspace, radio, InputResult::Updated);
        assert_eq!(radio.selected, 3);

        for i in (1..=3).rev() {
            assert_eq!(radio.focus, i);
            input(KeyCode::Up, radio, InputResult::Consumed);
        }
        assert_eq!(radio.focus, 0);

        input(KeyCode::Up, radio, InputResult::Ignored);
        assert_eq!(radio.focus, 0);

        input(KeyCode::F(1), radio, InputResult::Updated);
        assert_eq!(radio.selected, 0);
    }
}
