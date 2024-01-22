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
/// # use tundra::field::{Field, textbox::{Textbox, Builder}};
/// # let _ = 
/// <Textbox as Field>::builder()
///     .name("Password")
///     .value("admin")
///     .hidden()
///     .build()
/// # ;
/// ```
/// 
/// Three restrictions are placed on field builder types: 
/// 1. They must implement [`Default`]. 
/// 2. All methods can take at most one argument. 
/// 3. They must have a method with the signature `fn build(self) -> F`, where `F` is the field being
/// instantiated. 
/// 
/// For maximal flexibility in the design of the builder types, the third restriction is not enforced by the
/// trait system. This allows the `build` function to require compile-time predicates, such as requiring that
/// a specific builder method was called. 
/// 
/// All library-provided fields require that at least the [`Field::name`] is defined. To facilitate such
/// requirements, some syntax utilities are provided in this module, which may be used for custom field
/// builders as well. 
/// 
/// 
/// # Example
/// 
/// A simple builder with no type state:
/// ```
/// struct MyField {
///     name: String, 
///     // ...
/// }
/// 
/// struct Builder(MyField);
/// 
/// impl Builder {
///     pub fn name(self, name: String) -> Self {
///         Builder(MyField{ name, ..self.0 })
///     }
/// 
///     pub fn build(self) -> MyField {
///         self.0
///     }
/// }
/// ```
/// 
/// A builder requiring that a name was supplied: 
/// ```no_run
/// use tundra::field::builder::*;
/// 
/// struct MyField {
///     name: String, 
///     // ...
/// }
/// 
/// // type state parameter `NAME`, indicating whether the name has yet been supplied
/// struct Builder<const NAME: bool>(MyField);
/// 
/// impl<const NAME: bool> Builder<NAME> {
///     pub fn name(self, name: String) -> Builder<true>
///     where
///         Defined<NAME>: False, 
///     {
///         Builder(MyField{ name, ..self.0 })
///     }
/// 
///     // only callable if `Builder::name` was called first
///     pub fn build(self) -> MyField
///     where
///         Defined<NAME>: True, 
///     {
///         self.0
///     }
/// }
/// ```
pub mod builder {
    /// Indicates with a flag whether some property has been defined by the builder instance. 
    pub struct Defined<const B: bool>;

    /// Implemented for `Defined<true>`; allows the syntax `where Defined<true>: True`. 
    pub trait True {}
    impl True for Defined<true> {}

    /// Implemented for `Defined<false>`; allows the syntax `where Defined<false>: False`. 
    pub trait False {}
    impl False for Defined<false> {}
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
    /// instantiating the field. See the [`builder`] module for more information. 
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
    /// Constructs the [field builder](builder) using [`Default`]. 
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
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum InputResult {
    /// The key press was ignored. 
    Ignored, 
    /// The key press was consumed, but did not change the [`value`](Field::value) of the field (e.g., it may
    /// have affected internal focus). 
    Consumed, 
    /// The key press was used to update the [`value`](Field::value) of the field. 
    Updated, 
}
