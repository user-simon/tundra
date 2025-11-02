use std::borrow::Cow;
use bitvec::{bitbox, boxed::BitBox, slice::BitSlice};
use ratatui::{
    style::{Style, Stylize}, 
    text::{Line, Span, Text}, 
};
use crate::prelude::*;
use super::*;

/// An [input field](super) for toggling a set of items on/off. 
/// 
/// The value is a [`BitBox`] --- one bit for each item --- indicating whether the item corresponding to each
/// index is toggled. See [`toggle::Builder`] for the methods available when constructing the field. 
/// 
/// 
/// # Limiting the number of toggled items
/// 
/// Limits on the allowed number of toggled items can be introduced in [forms](dialog::form!) using field
/// validation. To aid this, the following error conditions are defined in the [toggle] module: [`exactly`], 
/// [`not_exactly`], [`less_than`], [`more_than`], [`outside_range`]. 
/// 
/// 
/// # Key bindings
/// 
/// [`KeyCode::Up`] and [`KeyCode::Down`] move the focused item up and down, respectively. Any other key
/// toggles the focused item. 
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Toggle {
    /// The user-visible name displayed by the input field. 
    pub name: Cow<'static, str>, 
    /// Index of the currently focused item. 
    focus: usize, 
    /// The user-visible names of the items that can be toggled. 
    items: Vec<Cow<'static, str>>, 
    /// Whether the item corresponding to each index is toggled. 
    values: BitBox, 
}

impl Toggle {
    /// Sets the user-visible names of all items that can be toggled. All existing values are discarded. 
    /// 
    /// 
    /// # Panics
    /// 
    /// When the number of items is zero. 
    pub fn set_items<T>(&mut self, items: impl IntoIterator<Item = T>)
    where
        T: Into<Cow<'static, str>>, 
    {
        // set items
        self.items = items
            .into_iter()
            .map(Into::into)
            .collect();
        assert!(!self.items.is_empty());

        // set all values to 0
        self.values = bitbox![0; self.items.len()];
    }

    /// Sets the values at given indices. 
    /// 
    /// 
    /// # Panics
    /// 
    /// When any given index is out of bounds. 
    pub fn set_indices(&mut self, indices: impl IntoIterator<Item = usize>) {
        for i in indices {
            self.values.set(i, true);
        }
    }

    /// Gets the names of the items that can be toggled. 
    pub fn items(&self) -> &[Cow<'static, str>] {
        &self.items
    }
}

impl Field for Toggle {
    type Value = BitBox;
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

            // toggle focused item on/off
            _ => {
                let mut bit = self.values
                    .get_mut(self.focus)
                    .expect("Focus is in range");
                *bit = !*bit;
                InputResult::Updated
            }
        }
    }

    fn format(&self, focused: bool) -> Text {
        std::iter::zip(self.items.iter(), self.values.iter())
            .enumerate()
            .map(|(i, (item, value))| {
                let value = *value;
                let symbol = match value {
                    true => "✓", 
                    false => " ", 
                };
                let style = Style::new().bold();
                match focused && i == self.focus {
                    true => Line::from(vec![
                        Span::styled("<", style), 
                        Span::from(symbol), 
                        Span::styled("> ", style), 
                        Span::from(item.as_ref()), 
                    ]), 
                    false => Line::from(format!("({symbol}) {item}")), 
                }
            })
            .collect::<Vec<_>>()
            .into()
    }

    fn value(&self) -> &Self::Value {
        &self.values
    }

    fn into_value(self) -> Self::Value {
        self.values
    }
}

/// Check whether number of toggled items is exactly `N`. 
/// 
/// Defined for use in field validation for [`Toggle`]. 
pub fn exactly(n: usize) -> impl Fn(&BitSlice) -> bool {
    move |bits| bits.count_ones() == n
}

/// Check whether number of toggled items is not exactly `N`. 
/// 
/// Defined for use in field validation for [`Toggle`]. 
pub fn not_exactly(n: usize) -> impl Fn(&BitSlice) -> bool {
    move |bits| bits.count_ones() != n
}

/// Check whether number of toggled items is less than `N`. 
/// 
/// Defined for use in field validation for [`Toggle`]. 
pub fn less_than(n: usize) -> impl Fn(&BitSlice) -> bool {
    move |bits| bits.count_ones() < n
}

/// Check whether number of toggled items is more than `N`. 
/// 
/// Defined for use in field validation for [`Toggle`]. 
pub fn more_than(n: usize) -> impl Fn(&BitSlice) -> bool {
    move |bits| bits.count_ones() > n
}

/// Check whether number of toggled items is less than `LOW` or more than `HIGH_INCLUSIVE`. 
/// 
/// Defined for use in field validation for [`Toggle`]. 
pub fn outside_range(low: usize, high_inclusive: usize) -> impl Fn(&BitSlice) -> bool {
    move |bits| {
        let count = bits.count_ones();
        count < low || count > high_inclusive
    }
}

/// Constructs a [`Toggle`]. 
/// 
/// This is used by the [form macro](crate::dialog::form!) when instantiating [toggles](Toggle), but may be
/// used in application code as well. 
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Builder<const NAME: bool = false, const ITEMS: bool = false>(Toggle);

impl Default for Builder {
    fn default() -> Self {
        Self(Toggle {
            name: Cow::default(), 
            focus: 0, 
            items: Vec::default(), 
            values: BitBox::default(), 
        })
    }
}

impl<const NAME: bool, const ITEMS: bool> Builder<NAME, ITEMS> {
    /// The user-visible name displayed by the input field. 
    pub fn name(self, name: impl Into<Cow<'static, str>>) -> Builder<true, ITEMS> {
        let name = name.into();
        Builder(Toggle{ name, ..self.0 })
    }

    /// The user-visible names of all items that can be toggled. 
    /// 
    /// 
    /// # Panics
    /// 
    /// When the number of items is zero. 
    pub fn items<T>(mut self, items: impl IntoIterator<Item = T>) -> Builder<NAME, true>
    where
        T: Into<Cow<'static, str>>, 
    {
        self.0.set_items(items);
        Builder(self.0)
    }
}

impl<const NAME: bool> Builder<NAME, true> {
    /// Sets the values at given indices. 
    /// 
    /// 
    /// # Panics
    /// 
    /// When any given index is out of bounds. 
    pub fn set(mut self, indices: impl IntoIterator<Item = usize>) -> Self {
        self.0.set_indices(indices);
        Builder(self.0)
    }
}

impl Build for Builder<true, true> {
    type Field = Toggle;

    /// If the name has been defined with [`Builder::name`] and the items have been defined with
    /// [`Builder::items`], consumes the builder and returns the constructed [`Toggle`]. 
    fn build(self) -> Toggle {
        self.0
    }
}
