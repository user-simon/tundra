use std::borrow::Cow;
use ratatui::text::Text;
use crate::prelude::*;
use super::*;

/// An [input field](super) for entering a boolean value. 
/// 
/// See [`checkbox::Builder`] for the methods available when constructing the field. 
/// 
/// 
/// # Key bindings
/// 
/// Any key toggles the value. 
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Checkbox {
    /// The user-visible name displayed by the input field. 
    pub name: Cow<'static, str>, 
    /// The current user-entered value. 
    pub value: bool, 
}

impl Field for Checkbox {
    type Value = bool;
    type Builder = Builder;

    fn name(&self) -> &str {
        &self.name
    }

    fn input(&mut self, key: KeyEvent) -> InputResult {
        if let KeyCode::Up | KeyCode::Down = key.code {
            InputResult::Ignored
        } else {
            self.value = !self.value;
            InputResult::Updated
        }
    }

    fn format(&self, _focused: bool) -> Text {
        match self.value {
            true => "✓", 
            false => "𐄂", 
        }.into()
    }

    fn value(&self) -> &Self::Value {
        &self.value
    }

    fn into_value(self) -> Self::Value {
        self.value
    }
}

/// Constructs a [`Checkbox`]. 
/// 
/// This is mainly used by the [form macro](crate::dialog::form!) when instantiating checkboxes, but may also
/// be used in application code for creating a stand-alone field. 
/// 
/// Requires that [`Builder::name`] is called before the field can be built. 
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Builder<const NAME: bool = false>(Checkbox);

impl Default for Builder {
    fn default() -> Self {
        Self(Checkbox {
            name: Default::default(), 
            value: false, 
        })
    }
}

impl<const NAME: bool> Builder<NAME> {
    /// The user-visible name displayed by the input field. 
    pub fn name(self, name: impl Into<Cow<'static, str>>) -> Builder<true> {
        let name = name.into();
        Builder(Checkbox{ name, ..self.0 })
    }

    /// The initial value. 
    pub fn value(self, value: bool) -> Self {
        Builder(Checkbox{ value, ..self.0 })
    }
}

impl Build for Builder<true> {
    type Field = Checkbox;

    fn build(self) -> Checkbox {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::{prelude::*, field::*};

    #[test]
    fn input() {
        let test = |key_code: KeyCode, expected: InputResult| {
            let mut checkbox = Checkbox::builder()
                .name("")
                .value(false)
                .build();
            let actual = checkbox.input(key_code.into());
            assert_eq!(actual, expected);
        };
        test(KeyCode::Char('a'), InputResult::Updated);
        test(KeyCode::Char('b'), InputResult::Updated);
        test(KeyCode::Char('1'), InputResult::Updated);
        test(KeyCode::Enter, InputResult::Updated);
        test(KeyCode::Esc, InputResult::Updated);

        // these two must be ignored for form navigation to work properly
        test(KeyCode::Up, InputResult::Ignored);
        test(KeyCode::Down, InputResult::Ignored);
    }
}
