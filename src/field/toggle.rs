use std::{borrow::Cow, ops::RangeInclusive, iter};
use ratatui::text::{Text, Line};
use crate::prelude::*;
use super::{*, builder::*};

/// An input [field](super) for toggling a set of items on/off. 
/// 
/// The value is an array of booleans --- one for each item --- indicating whether the item corresponding to
/// each index is toggled. 
/// 
/// 
/// # Limiting the number of toggled items
/// 
/// The allowed range of the number items to be toggled at any given time can be customised with
/// [`Toggle::set_range`] or [`Builder::range`]. E.g., a range of `2..=4` allows there to be
/// up-to-and-including 4, but at least 2, toggled items. If the limit is reached and another item is
/// toggled, the oldest toggled item is toggled off. The default range allows any number of items to be
/// toggled. 
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
    /// The allowed range of the number of items to be toggled at any given time. 
    range: RangeInclusive<usize>, 
    /// Index of the currently focused item. 
    focus: usize, 
    /// The user-visible names of the items that can be toggled. 
    items: Vec<Cow<'static, str>>, 
    /// Whether the item corresponding to each index is toggled. 
    values: Vec<bool>, 
    /// If the item corresponding to each index is toggled, contains the [`time`](Toggle::time) it was
    /// toggled. It's split up in different fields like this to allow [`Toggle::values`] to be returned from
    /// [`Field::value`]. 
    log: Vec<Option<usize>>, 
    /// The current "time". This is incremented each time an item is toggled, allowing its value to indicate
    /// the age of a toggled item. 
    time: usize, 
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
        debug_assert!(!self.items.is_empty());

        let min_toggled = self.range
            .start()
            .clone();
        let values = iter::repeat(true)
            .take(min_toggled)
            .chain(iter::repeat(false))
            .take(self.items.len());
        self.set_values(values);
    }

    /// Sets the values of all items. Does not verify whether the values are valid according to the allowed
    /// range (this is only an issue if a custom range is used). 
    /// 
    /// 
    /// # Panics
    /// 
    /// When the number of values is not equal to the number of items. 
    pub fn set_values(&mut self, values: impl IntoIterator<Item = bool>) {
        let (values, log) = values
            .into_iter()
            .map(|b| (b, b.then_some(self.time)))
            .unzip();
        self.values = values;
        self.log = log;

        debug_assert!(self.values.len() == self.items.len());
    }

    /// Sets the allowed range of toggled values. See the
    /// [type-level](Toggle#limiting-the-number-of-toggled-items) documentation for more information. This
    /// also ensures that at least `range.start()` items are toggled. 
    pub fn set_range(&mut self, range: RangeInclusive<usize>) {
        self.range = range;
        let min = self.range
            .start()
            .clone();
        let difference = min.saturating_sub(self.pop_count());
        let free = iter::zip(self.values.iter_mut(), self.log.iter_mut())
            .filter(|(&mut b, _)| !b)
            .take(difference);

        for (value, log) in free {
            *value = true;
            *log = Some(self.time)
        }
    }

    /// Gets the names of the items that can be toggled. 
    pub fn items(&self) -> &[Cow<'static, str>] {
        &self.items
    }
    
    /// Toggles the item corresponding to an index on. 
    /// 
    /// 
    /// # Returns
    /// 
    /// - `Ok(())` if the index could be toggled on. 
    /// - `Err(())` if the [pop count](Toggle::pop_count) is equal to the
    /// [upper limit](Toggle#limiting-the-number-of-toggled-items) and no items could be toggled off. 
    pub fn toggle_on(&mut self, index: usize) -> Result<(), ()> {
        if &self.pop_count() >= self.range.end() {
            let oldest = self.log
                .iter()
                .enumerate()
                .filter_map(|(i, time)| time.map(|time| (i, time)))
                .min_by_key(|&(_, time)| time)
                .map(|(i, _)| i);
            let Some(oldest) = oldest else {
                return Err(())
            };
            self.values[oldest] = false;
            self.log[oldest] = None;
        }
        self.time += 1;
        self.values[index] = true;
        self.log[index] = Some(self.time);
        Ok(())
    }

    /// Toggles the item corresponding to an index off. 
    /// 
    /// 
    /// # Returns
    /// 
    /// - `Ok(())` if the index could be toggled off. 
    /// - `Err(())` if the [pop count](Toggle::pop_count) is equal to the
    /// [lower limit](Toggle#limiting-the-number-of-toggled-items). 
    pub fn toggle_off(&mut self, index: usize) -> Result<(), ()> {
        if &self.pop_count() > self.range.start() {
            self.values[index] = false;
            self.log[index] = None;
            Ok(())
        } else {
            Err(())
        }
    }

    /// The current number of toggled items. 
    pub fn pop_count(&self) -> usize {
        self.values
            .iter()
            .filter(|&&t| t)
            .count()
    }
}

impl Field for Toggle {
    type Value = Vec<bool>;
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
                let result = match self.values[self.focus] {
                    true  => self.toggle_off(self.focus), 
                    false => self.toggle_on(self.focus), 
                };
                match result {
                    Ok(_) => InputResult::Updated, 
                    Err(_) => InputResult::Ignored, 
                }
            }
        }
    }

    fn format(&self, focused: bool) -> Text {
        let format_item = |i, item, toggled| {
            let symbol = match toggled {
                true  => '◼', 
                false => ' ', 
            };
            match focused && i == self.focus {
                true => format!("[{symbol}] {item}"), 
                false => format!("({symbol}) {item}"), 
            }
        };
        iter::zip(self.items.iter(), self.values.iter())
            .enumerate()
            .map(|(i, (item, &value))| format_item(i, item, value))
            .map(Line::from)
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

/// Constructs a [`Toggle`]. 
/// 
/// This is used by the [form macro](crate::dialog::form!) when instantiating [toggles](Toggle), but may be
/// used in application code as well. 
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Builder<const NAME: bool = false, const ITEMS: bool = false>(Toggle);

impl Default for Builder {
    fn default() -> Self {
        Self(Toggle {
            name: Default::default(), 
            range: 0..=usize::MAX, 
            focus: 0, 
            items: Default::default(), 
            values: Default::default(), 
            log: Default::default(), 
            time: 0, 
        })
    }
}

impl<const NAME: bool, const ITEMS: bool> Builder<NAME, ITEMS> {
    /// The user-visible name displayed by the input field. 
    pub fn name(self, name: impl Into<Cow<'static, str>>) -> Builder<true, ITEMS>
    where
        Defined<NAME>: False, 
    {
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
        Defined<ITEMS>: False, 
        T: Into<Cow<'static, str>>, 
    {
        self.0.set_items(items);
        Builder(self.0)
    }

    /// The initial values of all items. 
    /// 
    /// 
    /// # Panics
    /// 
    /// When the number of values is not equal to the number of items. 
    pub fn values(mut self, values: impl IntoIterator<Item = bool>) -> Self
    where
        Defined<ITEMS>: True, 
    {
        self.0.set_values(values);
        Builder(self.0)
    }

    /// The allowed range of toggled values. See the
    /// [type-level](Toggle#limiting-the-number-of-toggled-items) documentation for more information. This
    /// also ensures that at least `range.start()` items are toggled. 
    pub fn range(mut self, range: RangeInclusive<usize>) -> Self
    where
        Defined<ITEMS>: True, 
    {
        self.0.set_range(range);
        Builder(self.0)
    }

    /// If the name has been defined with [`Builder::name`] and the items have been defined with
    /// [`Builder::items`], consumes the builder and returns the constructed [`Toggle`]. 
    pub fn build(self) -> Toggle
    where
        Defined<NAME>: True, 
        Defined<ITEMS>: True, 
    {
        self.0
    }
}
