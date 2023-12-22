use ratatui::{
    text::Line, 
    widgets::*, 
    prelude::{Rect, Buffer}, 
};
use crate::KeyEvent;

/// Interface for builder methods on input fields. 
/// 
/// The [form macro](macro@crate::dialog::form) uses this interface when instantiating fields. More
/// specifically, the [DSL](https://en.wikipedia.org/wiki/Domain-specific_language)
/// `MyField{ foo: bar, baz }` gets (loosely) translated as: 
/// 
/// ```text
/// MyField::Builder::default()
///     .foo(bar)
///     .baz()
///     .build()
/// ```
pub trait Build<T: Field> {
    /// Constructs and returns the field, consuming the builder. 
    /// 
    /// For compatibility with the [form macro](macro@crate::dialog::form), this must be infallible. 
    fn build(self) -> T;
}

/// Interface for user-input fields. 
/// 
/// Fields are mainly designed for usage in [forms](macro@crate::dialog::form), but may be used on their own
/// by feeding key-presses with [`Field::input`] and drawing them using [`FieldWidget`], which implements
/// [`Widget`]. 
pub trait Field: Sized {
    /// The type of value entered by the user. 
    type Value;
    /// Points toward the builder type that may be used by the [form macro](macro@crate::dialog::form) for
    /// instantiating fields. 
    type Builder: Build<Self>;

    /// Retrieves the user-visible name displayed by the input field. 
    fn name(&self) -> &str;
    /// Passes a key input event. 
    fn input(&mut self, key: KeyEvent);
    /// Renders the field to a single line. 
    fn format(&self, selected: bool) -> Line;
    /// Borrows the currently entered value. 
    fn value(&self) -> &Self::Value;
    /// Consumes the field and returns the currently entered value. 
    fn into_value(self) -> Self::Value;
}

/// TODO
#[derive(Clone, Copy, Debug)]
pub struct FieldWidget<'a, T>(pub &'a T);

impl<T: Field> Widget for FieldWidget<'_, T> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let field = self.0;
        let content = field.format(true);
        let block = Block::default()
            .title(format!(" {} ", field.name().to_uppercase()))
            .borders(Borders::ALL);
        Paragraph::new(content)
            .block(block)
            .render(area, buf);
    }
}
