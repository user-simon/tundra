use std::{
    borrow::Cow, 
    fmt::Display, 
    ops::{Sub, Add}, 
};
use num_traits::{Bounded, One, Zero};
use ratatui::{
    text::{Line, Span}, 
    style::{Style, Stylize}, 
};
use crate::prelude::*;
use super::field::*;

/// An input [field](super::Field) for entering a numerical value. 
#[derive(Clone, Debug)]
pub struct Slider<T> {
    pub name: Cow<'static, str>,
    pub value: T, 
    pub min: T, 
    pub max: T, 
    pub step: T, 
    pub default: T, 
}

impl<T> Default for Slider<T>
where
    T: Zero + One + Bounded, 
{
    fn default() -> Slider<T> {
        Slider {
            name: Default::default(), 
            value: T::zero(), 
            min: T::min_value(), 
            max: T::max_value(), 
            step: T::one(), 
            default: T::zero(), 
        }
    }
}

impl<T> Field for Slider<T>
where
    T: Clone + Display + PartialOrd,
    for<'a> &'a T: Add<Output = T> + Sub<Output = T>, 
{
    type Value = T;
    type Builder = Builder<T>;

    fn name(&self) -> &str {
        &self.name
    }

    fn input(&mut self, key: KeyEvent) {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        self.value = match (key.code, ctrl) {
            // move slider one step
            (KeyCode::Left, false) => {
                if self.value >= &self.min + &self.step {
                    &self.value - &self.step
                } else {
                    self.min.clone()
                }
            }
            (KeyCode::Right, false) => {
                if self.value <= &self.max - &self.step {
                    &self.value + &self.step
                } else {
                    self.max.clone()
                }
            }

            // move slider to nearest anchor (min, default, max)
            (KeyCode::Left, true) => {
                if self.value > self.default {
                    self.default.clone()
                } else {
                    self.min.clone()
                }
            }
            (KeyCode::Right, true) => {
                if self.value < self.default {
                    self.default.clone()
                } else {
                    self.max.clone()
                }
            }
            _ => return, 
        }
    }

    fn format(&self, selected: bool) -> Line {
        let val = format!("{}", self.value);
        let style = |cond| match cond {
            true => Style::new().bold(), 
            false => Style::new()
        };
        Line::from(vec![
            Span::styled("<", style(self.value != self.min)), 
            Span::styled(val, style(selected)), 
            Span::styled(">", style(self.value != self.max))
        ])
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
/// This is used by the [form macro](macro@crate::dialog::form) when instantiating [sliders](Slider), but may
/// be used in application code as well. 
#[derive(Clone, Debug)]
pub struct Builder<T>(pub Slider<T>);

impl<T> Default for Builder<T>
where
    Slider<T>: Default, 
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T> Builder<T> {
    pub fn name(self, name: impl Into<Cow<'static, str>>) -> Self {
        let name = name.into();
        Builder(Slider{ name, ..self.0 })
    }

    pub fn value(self, value: T) -> Self
    where
        T: Clone
    {
        let default = value.clone();
        Builder(Slider{ value, default, ..self.0 })
    }

    pub fn min(self, min: T) -> Self {
        Builder(Slider{ min, ..self.0 })
    }

    pub fn max(self, max: T) -> Self {
        Builder(Slider{ max, ..self.0 })
    }

    pub fn step(self, step: T) -> Self {
        Builder(Slider{ step, ..self.0 })
    }
}

impl<T> Build<Slider<T>> for Builder<T>
where
    Slider<T>: Field, 
{
    fn build(self) -> Slider<T> {
        self.0
    }
}
