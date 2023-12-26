#![doc(hidden)]

/// Displays a user input form with specified [fields](crate::input) in a [dialog](crate::dialog) to the
/// user. 
/// 
/// The easiest way to get a hang of how to use the macro is to just examine the [examples](#examples), but a
/// (somewhat) more formal description of the syntax is provided as well. 
/// 
/// The syntax expects first a set of fields, followed by a set of required metadata. The items should be
/// comma-separated (trailing commas are optional). 
/// 
/// 
/// # Fields
/// 
/// A field consists of:
/// - An identifier; used to reference the entered value after the form has been submitted by the user. 
/// - A field type; any type that implements [`Field`](crate::input::Field). 
/// - A set of parameters used when instantiating the field; these are translated into methods on the
/// [field builder](crate::input::Build). There are two kinds of parameters: those with one argument and
/// those with none. Those with one argument are specified as `IDENTIFIER: VALUE`. Those with no argument are
/// specified simply as `IDENTIFIER`. 
/// 
/// The syntax for declaring a field follows the form: `IDENTIFIER: TYPE{ PARAMS }`. For example, to declare
/// a textbox with identifier `password`, and with parameters `name = "Password"` (one argument) and `hidden`
/// (no argument): 
/// 
/// ```text
/// password: Textbox{ name: "Password", hidden }, 
/// ```
/// 
/// The DSL `Textbox{ name: "Password", hidden }` gets (loosely) translated as: 
/// 
/// ```text
/// Textbox::Builder::default()
///     .name("Password")
///     .hidden()
///     .build()
/// ```
/// 
/// 
/// # Required Metadata
/// 
/// In addition to the fields, the following metadata is required (in order): 
/// 1. `title`; the user-visible title of the dialog box. Should be `impl Into<Cow<str>>`. 
/// 2. `context`; the current [context](crate::Context). Should be `&mut Context<_>`. 
/// 3. `background`; the state shown underneath the dialog box. Should be `&impl State`. 
/// 
/// The syntax for providing a piece of meta-data follows the form `[IDENTIFIER]: VALUE`. For example, to
/// provide the title of the form as `"My form"`: 
/// 
/// ```text
/// [title]: "My form", 
/// ```
/// 
/// 
/// # (Optional) Validation Function
/// 
/// Optionally, a validation function may be specified as the last piece of metadata. The entered user input
/// is validated using this function before the form may be submitted. If the input fails to validate, a
/// given error message is shown before the user is prompted to retry. 
/// 
/// The validation function accepts as argument a struct containing a reference to the values of all fields. 
/// Since this struct is unspellable by application code, the function must be a closure. 
/// 
/// The validation function should return a value of `Result<(), impl AsRef<str>>`. On succeess, `Ok(())`
/// should be returned, indicating that the entered values were valid. On failure, `Err` should be returned
/// with the error message that is to be displayed to the user. 
/// 
/// The syntax for declaring a validation function follows the form `[validate]: |IDENTIFIER| EXPRESSION`. 
/// For example, to validate that a slider `foo`'s value is non-zero: 
/// 
/// ```text
/// [validate]: |form| if *form.foo == 0 {
///     Err("Foo must be non-zero!")
/// } else {
///     Ok(())
/// }
/// ```
/// 
/// 
/// # Returns
/// 
/// The return value of the macro is an [`Option`]: 
/// - `Some` if the form was submitted. Contains the values of all fields as members of an
/// unspellable struct. The identifiers of the values are the same as the corresponding fields. 
/// - `None` if the form was cancelled. 
/// 
/// 
/// # Examples
/// 
/// ```no_run
/// use tundra::{prelude::*, input::*};
/// 
/// # let current_state = &();
/// # let ctx = &mut Context::new().unwrap();
/// // let current_state: &impl State
/// // let ctx: &mut Context<_>
/// 
/// let form = dialog::form!{
///     foo: Textbox{ name: "Foo" }, 
///     bar: Slider<f32>{ name: "Bar", step: 0.25, min: 0.0, max: 2.0 }, 
///     baz: Checkbox{ name: "Baz" }, 
///     [title]: "Here's a form!", 
///     [context]: ctx, 
///     [background]: current_state, 
/// }?;
/// if let Some(form) = form {
///     let foo: String = form.foo;
///     let bar: f32 = form.bar;
///     let baz: bool = form.baz;
/// }
/// # Ok::<(), std::io::Error>(())
/// ```
/// 
/// 
/// To show a login prompt, checking the credentials before proceeding: 
/// 
/// ```no_run
/// use tundra::{prelude::*, input::*};
/// 
/// # let current_state = &();
/// # let ctx = &mut Context::new().unwrap();
/// // let current_state: &impl State
/// // let ctx: &mut Context<_>
/// 
/// let form = dialog::form!{
///     username: Textbox{ name: "Username" }, 
///     password: Textbox{ name: "Password", hidden }, 
///     [title]: "Login", 
///     [context]: ctx, 
///     [background]: current_state, 
///     [validate]: |form| if form.username == "admin" && form.password == "password1" {
///         Ok(())
///     } else {
///         Err("Invalid credentials. Try again.")
///     }
/// }?;
/// match form {
///     Some(_) => { /* form submitted -> login success */ }
///     None => { /* form cancelled -> login failure */ }
/// }
/// # Ok::<(), std::io::Error>(())
/// ```
#[macro_export]
macro_rules! form {
    {
        // A comma-separated list of fields
        $(
            $id:ident: $type:ty {
                // Parameters for each field using builder pattern methods
                $(
                    $arg_id:ident $(: $arg_val:expr)?
                ),+
                $(,)?
            }
        ),+, 
        // User-visible title of the dialog box. Should be `impl Into<Cow<str>>`
        [title]: $title:expr, 
        // Current context. Should be `&mut Context<_>`
        [context]: $ctx:expr, 
        // State shown underneath the dialog box. Should be `&impl State`
        [background]: $bg:expr
        // Function to validate the entered values. Should return `Result<(), impl AsRef<str>>`
        $(, [validate]: |$form_id:ident| $validate:expr)?
        $(,)?
    } => {{
        struct __Form<'a> {
            pub __selected: usize, 
            pub __title: std::borrow::Cow<'a, str>, 
            $(
                pub $id: $type, 
            )*
        }

        #[allow(dead_code)]
        struct __Values {$(
            pub $id: <$type as $crate::input::Field>::Value,
        )*}
        
        #[allow(dead_code)]
        struct __BorrowedValues<'a> {$(
            pub $id: &'a <$type as $crate::input::Field>::Value,
        )*}

        #[allow(unused_variables)]
        const __FIELDS: usize = 0 $(
            + {
                let $id = ();
                1
            }
        )*;
        const __MAX_FIELD: usize = __FIELDS - 1;

        impl __Form<'_> {
            fn format_fields(&self) -> ratatui::text::Text {
                use std::vec::Vec;
                use ratatui::text::Line;
                use $crate::{input::Field, dialog::form::internal};

                type Dispatch = for<'a> fn(&'a __Form<'a>, bool, usize) -> Line<'a>;

                let name_lengths = [$(
                    Field::name(&self.$id).len(), 
                )*];
                let max_name = name_lengths
                    .iter()
                    .max()
                    .copied()
                    .unwrap_or(0);

                const DISPATCHES: [Dispatch; __FIELDS] = [$(
                    |form, selected, align| internal::format_field(&form.$id, selected, align), 
                )*];
                
                DISPATCHES.iter()
                    .enumerate()
                    .map(|(i, f)| f(self, i == self.__selected, max_name))
                    .collect::<Vec<_>>()
                    .into()
            }
        }

        impl $crate::dialog::Dialog for __Form<'_> {
            fn input(&mut self, key: $crate::KeyEvent) -> $crate::Signal {
                use $crate::{
                    input::Field, 
                    KeyCode, KeyEvent, Signal, 
                };
                type Dispatch = fn(&mut __Form, KeyEvent);

                match key.code {
                    KeyCode::Esc => Signal::Cancelled, 
                    KeyCode::Enter => Signal::Done, 
                    KeyCode::Up => {
                        self.__selected = self.__selected.saturating_sub(1);
                        Signal::Running
                    }
                    KeyCode::Down => {
                        self.__selected = match self.__selected {
                            __MAX_FIELD => __MAX_FIELD, 
                            c => c + 1, 
                        };
                        Signal::Running
                    }
                    _ => {
                        const JUMP_TABLE: [Dispatch; __FIELDS] = [$(
                            |form, key| Field::input(&mut form.$id, key)
                        ),*];
                        JUMP_TABLE[self.__selected](self, key);
                        Signal::Running
                    }
                }
            }

            fn format(&self) -> $crate::dialog::DrawInfo {
                use ratatui::style::Color;
                use $crate::dialog::DrawInfo;

                DrawInfo {
                    title: self.__title.as_ref().into(), 
                    body: self.format_fields(), 
                    hint: "Press (enter) to submit, (esc) to cancel...".into(), 
                    ..Default::default()
                }
            }
        }

        impl $crate::dialog::form::internal::Form for __Form<'_> {
            type Values = __Values;
            type BorrowedValues<'a> = __BorrowedValues<'a> where Self: 'a;
    
            fn into_values(self) -> Self::Values {
                use $crate::input::Field;

                __Values {$(
                    $id: Field::into_value(self.$id), 
                )*}
            }

            fn values<'a>(&'a self) -> Self::BorrowedValues<'a> {
                use $crate::input::Field;

                __BorrowedValues {$(
                    $id: Field::value(&self.$id), 
                )*}
            }
        }

        let __validate = (
            // if $validate is defined, first element is that callback
            $(|$form_id: __BorrowedValues| $validate,)? 
            // otherwise, first element is this default validator that always returns Ok
            |_: __BorrowedValues| std::result::Result::<(), &str>::Ok(()), 
        ).0;
        
        let __form = __Form {
            __selected: 0, 
            __title: std::convert::Into::into($title), 
            // initialize fields with builder pattern using given arguments
            $($id: {
                type __Builder = <$type as $crate::input::Field>::Builder;
                
                let builder = <__Builder as std::default::Default>::default()
                $(
                    .$arg_id($($arg_val)?)
                )*;
                $crate::input::Build::build(builder)
            },)*
        };
        $crate::dialog::form::internal::Form::run_over(__form, $bg, $ctx, __validate)
    }}
}

pub mod internal {
    use std::{io, iter};
    use ratatui::{
        text::{Line, Span}, 
        style::{Style, Stylize}, 
    };
    use crate::{
        dialog::{self, *}, 
        input::Field, 
    };

    pub trait Form {
        type Values;
        type BorrowedValues<'a> where Self: 'a;

        fn into_values(self) -> Self::Values;
        fn values<'a>(&'a self) -> Self::BorrowedValues<'a>;

        fn run_over<G, T, U, V>(mut self, bg: &T, ctx: &mut Context<G>, mut validate: U)
            -> io::Result<Option<Self::Values>>
        where
            Self: Dialog, 
            T: State, 
            U: FnMut(Self::BorrowedValues<'_>) -> std::result::Result<(), V>, 
            V: AsRef<str>, 
        {
            Ok(loop {
                // run form dialog; if the user cancels, exit immediately
                let Some(out) = Dialog::run_over(self, bg, ctx)? else {
                    break None
                };
                self = out;

                match validate(self.values()) {
                    Ok(_) => break Some(self.into_values()), 
                    Err(e) => dialog::error(e, bg, ctx)?, 
                }
            })
        }
    }

    pub fn format_field(field: &impl Field, selected: bool, align_to: usize) -> Line {
        let (delimiter, style) = match selected {
            true => (" : ", Style::new().bold()),
            false => (" │ ", Style::new()),
        };
        let name = field.name();
        let padding = align_to.saturating_sub(name.len());
        let title: String = iter::repeat(' ')
            .take(padding)
            .chain(name.chars())
            .chain(delimiter.chars())
            .collect();
        let title = Span::styled(title, style);
        let mut body = field.format(selected);
        body.spans.insert(0, title);
        body
    }
}

pub use form;

#[cfg(test)]
mod test {
    
}
