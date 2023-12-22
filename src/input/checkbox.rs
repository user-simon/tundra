use std::borrow::Cow;
use ratatui::text::Line;
use crate::prelude::*;
use super::field::*;

/// An input [field](super::Field) for entering a boolean value. 
#[derive(Clone, Debug, Default)]
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

    fn input(&mut self, _key: KeyEvent) {
        self.value = !self.value;
    }

    fn format(&self, _selected: bool) -> Line {
        let value = match self.value {
            true => "✔", 
            false => "✖", 
        };
        value.into()
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
/// This is used by the [form macro](macro@crate::dialog::form) when instantiating [checkboxes](Checkbox),
/// but may be used in application code as well. 
#[derive(Clone, Debug, Default)]
pub struct Builder(pub Checkbox);

impl Builder {
    /// The user-visible name displayed by the input field. 
    pub fn name(self, name: impl Into<Cow<'static, str>>) -> Self {
        let name = name.into();
        Builder(Checkbox{ name, ..self.0 })
    }

    /// The initial value. 
    pub fn value(self, value: bool) -> Self {
        Builder(Checkbox{ value, ..self.0 })
    }
}

impl Build<Checkbox> for Builder {
    fn build(self) -> Checkbox {
        self.0
    }
}
