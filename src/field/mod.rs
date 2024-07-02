//! Input fields for allowing the user to enter various kinds of data. 
//! 
//! The following input fields are defined in this module: 
//! - [`Checkbox`] for entering booleans. 
//! - [`Radio`] for selecting one item among a set. 
//! - [`Slider`] for entering a number in a range. 
//! - [`Textbox`] for entering single-line strings. 
//! - [`Toggle`] for toggling a set of items on/off. 
//! 
//! Fields are mainly designed to be used in [forms](crate::dialog::form!), but can be used on their own by
//! feeding key-presses with [`Field::input`] and drawing them using the [`Text`] returned from
//! [`Field::format`]. 
//! 
//! 
//! # Custom Fields
//! 
//! Custom fields may be created by implementing the [`Field`] trait. See its documentation for more
//! information. 

pub mod checkbox;
pub mod radio;
pub mod slider;
pub mod textbox;
pub mod toggle;

use ratatui::text::Text;
use crate::KeyEvent;

#[doc(inline)]
pub use {
    checkbox::Checkbox, 
    radio::Radio, 
    slider::Slider, 
    textbox::Textbox, 
    toggle::Toggle, 
};

/// Field builder specification. 
/// 
/// Builders are responsible for providing the methods available when instantiating fields inside a
/// [form](crate::dialog::form!). More specifically, the DSL
/// `Textbox{ name: "Password", value: "admin", hidden }` gets (loosely) translated as: 
/// ```no_run
/// # use tundra::field::{Field, Build, textbox::{Textbox, Builder}};
/// # let _ = 
/// Textbox::builder()
///     .name("Password")
///     .value("admin")
///     .hidden()
///     .build()
/// # ;
/// ```
/// 
/// Three restrictions are placed on field builder types: 
/// 1. They must implement [`Default`]. 
/// 2. They must implement [`Build`]. 
/// 3. All methods can take at most one argument. 
/// 
/// For maximal flexibility, the second restriction is not added as a bound to [`Field::Builder`]. This
/// allows the [`Build`] trait implementation to be predicated on type-state, such as requiring that a
/// specific builder method was called. 
/// 
/// All library-provided fields require that at least the [`Field::name`] is defined. 
/// 
/// 
/// # Example
/// 
/// A simple builder with no type state:
/// ```no_run
/// # use tundra::{KeyEvent, field::InputResult};
/// # use ratatui::text::Text;
/// use tundra::field::{Field, Build};
/// 
/// #[derive(Default)]
/// struct MyField {
///     name: String, 
///     // ...
/// }
/// 
/// impl Field for MyField {
///     type Builder = Builder;
///     
///     // ...
///     # type Value = ();
///     # fn name(&self) -> &str { todo!() }
///     # fn input(&mut self, _: KeyEvent) -> InputResult { todo!() }
///     # fn format(&self, _: bool) -> Text { todo!() }
///     # fn value(&self) -> &() { todo!() }
///     # fn into_value(self) -> Self::Value { todo!() }
/// }
/// 
/// #[derive(Default)]
/// struct Builder(MyField);
/// 
/// impl Builder {
///     pub fn name(self, name: String) -> Self {
///         Builder(MyField{ name, ..self.0 })
///     }
/// }
/// 
/// impl Build for Builder {
///     type Field = MyField;
/// 
///     fn build(self) -> MyField {
///         self.0
///     }
/// }
/// ```
/// 
/// A builder requiring that a name was supplied: 
/// ```no_run
/// # use tundra::{KeyEvent, field::InputResult};
/// # use ratatui::text::Text;
/// use tundra::field::{Field, Build};
/// 
/// #[derive(Default)]
/// struct MyField {
///     name: String, 
///     // ...
/// }
/// 
/// impl Field for MyField {
///     type Builder = Builder<false>;
///     
///     // ...
///     # type Value = ();
///     # fn name(&self) -> &str { todo!() }
///     # fn input(&mut self, _: KeyEvent) -> InputResult { todo!() }
///     # fn format(&self, _: bool) -> Text { todo!() }
///     # fn value(&self) -> &() { todo!() }
///     # fn into_value(self) -> Self::Value { todo!() }
/// }
/// 
/// // note the type state parameter `NAME`, indicating whether the name has yet been supplied
/// #[derive(Default)]
/// struct Builder<const NAME: bool>(MyField);
/// 
/// impl Builder<false> {
///     // only callable if name has not yet been given
///     pub fn name(self, name: String) -> Builder<true> {
///         Builder(MyField{ name, ..self.0 })
///     }
/// }
/// 
/// impl Build for Builder<true> {
///     type Field = MyField;
/// 
///     // only callable if name has been given
///     fn build(self) -> MyField {
///         self.0
///     }
/// }
pub trait Build: Sized {
    type Field: Field;

    fn build(self) -> Self::Field;
}

/// Interface for user input fields. 
/// 
/// For most applications, the [library provided fields](self) should suffice, but custom fields may be
/// created by implementing this trait. 
/// 
/// Fields are mainly designed to be used in [forms](crate::dialog::form!), but can be used on their own by
/// feeding key-presses with [`Field::input`] and drawing them using the [`Text`] returned from
/// [`Field::format`]. 
pub trait Field: Sized {
    /// The type of value entered by the user. 
    type Value;
    /// Points toward the builder type that may be used by the [form macro](crate::dialog::form!) for
    /// instantiating the field. The type should implement [`Build`], but this is not added as a bound for
    /// maximal flexibility. See the [`Build`] trait for more information. 
    type Builder: Default;

    /// Retrieves the user-visible name displayed by the input field. 
    fn name(&self) -> &str;
    /// Passes a key input event. 
    fn input(&mut self, key: KeyEvent) -> InputResult;
    /// Renders the field. 
    fn format(&self, focused: bool) -> Text;
    /// Borrows the current user-entered value.
    fn value(&self) -> &Self::Value;
    /// Consumes the field and returns the current user-entered value. 
    fn into_value(self) -> Self::Value;
    /// Constructs the [field builder](Build) using [`Default`]. 
    fn builder() -> Self::Builder {
        Default::default()
    }
}

/// Indicates the result of a call to [`Field::input`]. 
/// 
/// 
/// # Custom fields
/// 
/// Note that care should be taken when and when not to return [`Consumed`](InputResult::Consumed), since it
/// blocks [forms](crate::dialog::form!) from responding to [`KeyCode::Up`](crate::prelude::KeyCode::Up) and
/// [`KeyCode::Down`](crate::prelude::KeyCode::Down) inputs. 
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum InputResult {
    /// The key press was ignored. 
    Ignored, 
    /// The key press was consumed, but did not change the [`value`](Field::value) of the field (e.g., it may
    /// have affected internal focus). 
    Consumed, 
    /// The key press was used to update the [`value`](Field::value) of the field. 
    Updated, 
}
