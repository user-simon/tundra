use std::{
    borrow::Cow, 
    fmt::Display, 
    ops::{Sub, Add, RangeInclusive}, 
};
use num_traits::{Bounded, One, Zero};
use ratatui::{
    text::{Line, Span, Text}, 
    style::{Style, Stylize}, 
};
use crate::prelude::*;
use super::{*, builder::*};

/// An [input field](super) for entering a numerical value. 
/// 
/// The type parameter `T` is the type of the value being entered. The following bounds are placed on `T`: 
/// ```text
///  T: Clone + Display + PartialOrd + num_traits::Zero + num_traits::One + num_traits::Bounded, 
/// &T: Add<Output = T> + Sub<Output = T>, 
/// ```
/// Those bounds hold for all primitive numerical types (e.g., `i8`, `usize`, `f64`), but the design allows
/// for other types as well. 
/// 
/// See [`slider::Builder`] for the methods available when constructing the field. 
/// 
/// 
/// # Key bindings
/// 
/// [`KeyCode::Left`] and [`KeyCode::Right`] move the value one step to the left and right, respectively. If
/// a modifier key is held, the value is "snapped" to the nearest anchor in the given direction, where the
/// anchors are `self.range.start()`, `self.default`, and `self.range.end()` (in order). 
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Slider<T> {
    /// The user-visible name displayed by the input field. 
    pub name: Cow<'static, str>,
    /// The current user-entered value. 
    pub value: T, 
    /// The allowed range of the value that can be entered. 
    pub range: RangeInclusive<T>, 
    /// The step-size. The value is incremented/decremented by this amount. 
    pub step: T, 
    /// The default value. 
    pub default: T, 
}

impl<T> Field for Slider<T>
where
    T: Clone + Display + PartialOrd, 
    Builder<T>: Default, 
    for<'a> &'a T: Add<Output = T> + Sub<Output = T>, 
{
    type Value = T;
    type Builder = Builder<T>;

    fn name(&self) -> &str {
        &self.name
    }

    fn input(&mut self, key: KeyEvent) -> InputResult {
        let modifier = !key.modifiers.is_empty();
        self.value = match (key.code, modifier) {
            // move slider one step
            (KeyCode::Left, false) if &self.value > self.range.start() => {
                if self.value >= self.range.start() + &self.step {
                    &self.value - &self.step
                } else {
                    self.range.start().clone()
                }
            }
            (KeyCode::Right, false) if &self.value < self.range.end() => {
                if self.value <= self.range.end() - &self.step {
                    &self.value + &self.step
                } else {
                    self.range.end().clone()
                }
            }

            // move slider to nearest anchor (min, default, max)
            (KeyCode::Left, true) if &self.value > self.range.start() => {
                if self.value > self.default {
                    self.default.clone()
                } else {
                    self.range.start().clone()
                }
            }
            (KeyCode::Right, true) if &self.value < self.range.end() => {
                if self.value < self.default {
                    self.default.clone()
                } else {
                    self.range.end().clone()
                }
            }
            _ => return InputResult::Ignored, 
        };
        InputResult::Updated
    }

    fn format(&self, focused: bool) -> Text {
        let val = format!("{}", self.value);
        let style = |cond| match cond {
            true => Style::new().bold(), 
            false => Style::new(), 
        };
        Line::from(vec![
            Span::styled("<", style(&self.value != self.range.start())), 
            Span::styled(val, style(focused)), 
            Span::styled(">", style(&self.value != self.range.end()))
        ]).into()
    }

    fn value(&self) -> &T {
        &self.value
    }

    fn into_value(self) -> T {
        self.value
    }
}

/// Constructs a [`Slider`]. 
/// 
/// This is mainly used by the [form macro](crate::dialog::form!) when instantiating sliders, but may also
/// be used in application code for creating a stand-alone field. 
/// 
/// Requires that [`Builder::name`] is called before the field can be built. 
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Builder<T, const NAME: bool = false>(Slider<T>);

impl<T> Default for Builder<T>
where
    T: Zero + One + Bounded, 
{
    fn default() -> Self {
        Self(Slider {
            name: Default::default(), 
            value: T::zero(), 
            range: T::min_value()..=T::max_value(), 
            step: T::one(), 
            default: T::zero(), 
        })
    }
}

impl<T, const NAME: bool> Builder<T, NAME> {
    /// The user-visible name displayed by the input field. 
    pub fn name(self, name: impl Into<Cow<'static, str>>) -> Builder<T, true>
    where
        Defined<NAME>: False, 
    {
        let name = name.into();
        Builder(Slider{ name, ..self.0 })
    }

    /// The initial and default value. 
    pub fn value(self, value: T) -> Self
    where
        T: Clone, 
    {
        let default = value.clone();
        Builder(Slider{ value, default, ..self.0 })
    }

    /// The allowed range of the value that can be entered. Clamps the value to the range. 
    pub fn range(self, range: RangeInclusive<T>) -> Self
    where
        T: Clone + PartialOrd, 
    {
        let (min, max) = range.clone().into_inner();
        let value = self.0.value.clone();
        let value = match (value < min, value > max) {
            (true, _) => min, 
            (_, true) => max, 
            (_, _) => value, 
        };
        Builder(Slider{ range, ..self.0 }).value(value)
    }

    /// The amount that is added to or subtracted from the value. 
    pub fn step(self, step: T) -> Self {
        Builder(Slider{ step, ..self.0 })
    }

    /// If the name has been defined with [`Builder::name`], consumes the builder and returns the constructed
    /// [`Slider`]. 
    pub fn build(self) -> Slider<T>
    where
        Defined<NAME>: True, 
    {
        self.0
    }
}
