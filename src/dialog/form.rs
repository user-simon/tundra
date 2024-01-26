#![doc(hidden)]

/// Displays a user input form with specified [fields](crate::field) in a [dialog](crate::dialog) to the
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
/// - A field type; any type that implements [`Field`](crate::field::Field). 
/// - A set of parameters used when instantiating the field; these are translated into methods on the
/// [field builder](crate::field::builder). There are two kinds of parameters allowed: those with one
/// argument and those with none. Those with one argument are specified as `IDENTIFIER: VALUE`. Those with no
/// argument are specified simply as `IDENTIFIER`. 
/// 
/// The syntax for declaring a field follows the form: `IDENTIFIER: TYPE{ PARAMS }`. For example, to declare
/// a textbox with identifier `password`, and parameters `name = "Password"`, `value = "admin"`, and `hidden`
/// (no argument). 
/// ```text
/// password: Textbox{ name: "Password", value: "admin", hidden }, 
/// ```
/// 
/// The DSL `Textbox{ name: "Password", value: "admin", hidden }` gets (loosely) translated as: 
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
/// See the [`field::builder`](crate::field::builder) module for more information about builders. 
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
/// ```text
/// [validate]: |form| if form.foo == &0 {
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
/// To show a form with a [textbox](crate::field::Textbox), [slider](crate::field::Slider), and
/// [checkbox](crate::field::Checkbox), extracting the entered values from each: 
/// ```no_run
/// use tundra::{prelude::*, field::*};
/// 
/// # let current_state = &();
/// # let ctx = &mut Context::new().unwrap();
/// // let current_state: &impl State
/// // let ctx: &mut Context<_>
/// 
/// let values = dialog::form!{
///     location: Textbox{ name: "Location" }, 
///     rent: Slider<u32>{ name: "Monthly rent", range: 1..=5000, step: 50 }, 
///     pets_allowed: Checkbox{ name: "Pets allowed" }, 
///     [title]: "Register Rent Unit", 
///     [context]: ctx, 
///     [background]: current_state, 
/// }?;
/// if let Some(values) = values {
///     // type annotation is not required
///     let location: String = values.location;
///     let rent: u32 = values.rent;
///     let pets_allowed: bool = values.pets_allowed;
/// }
/// # Ok::<(), std::io::Error>(())
/// ```
/// 
/// To show a login prompt, checking the credentials before proceeding: 
/// ```no_run
/// use tundra::{prelude::*, field::*};
/// 
/// # let current_state = &();
/// # let ctx = &mut Context::new().unwrap();
/// // let current_state: &impl State
/// // let ctx: &mut Context<_>
/// 
/// let values = dialog::form!{
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
/// match values {
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
            pub __focus: usize, 
            pub __title: std::borrow::Cow<'a, str>, 
            $(
                pub $id: $type, 
            )*
        }

        #[allow(dead_code)]
        struct __Values {$(
            pub $id: <$type as $crate::field::Field>::Value,
        )*}
        
        #[allow(dead_code)]
        struct __BorrowedValues<'a> {$(
            pub $id: &'a <$type as $crate::field::Field>::Value,
        )*}

        #[allow(unused_variables)]
        const __FIELDS: usize = 0 $(
            + {
                let $id = ();
                1
            }
        )*;

        impl $crate::dialog::form::internal::Form for __Form<'_> {
            type Values = __Values;
            type BorrowedValues<'a> = __BorrowedValues<'a> where Self: 'a;

            fn title(&self) -> &str {
                std::convert::AsRef::as_ref(&self.__title)
            }

            fn max_focus(&self) -> usize {
                __FIELDS - 1
            }

            fn focus(&self) -> usize {
                self.__focus
            }

            fn set_focus(&mut self, focus: usize) {
                self.__focus = focus;
            }

            fn format_dispatch(&self) -> std::vec::Vec<ratatui::text::Text> {
                use std::vec::Vec;
                use ratatui::text::{Line, Text};
                use $crate::{field::Field, dialog::form::internal};

                type Dispatch = for<'a> fn(&'a __Form<'a>, bool, usize) -> Text;

                let name_lengths = [$(
                    Field::name(&self.$id).len(), 
                )*];
                let max_name = name_lengths
                    .into_iter()
                    .max()
                    .unwrap_or(0);

                const DISPATCHES: [Dispatch; __FIELDS] = [$(
                    |form, focus, align_to| {
                        let name = Field::name(&form.$id);
                        let body = Field::format(&form.$id, focus);
                        internal::format_field(name, body, focus, align_to)
                    }, 
                )*];

                DISPATCHES.iter()
                    .enumerate()
                    .map(|(i, f)| f(self, i == self.__focus, max_name))
                    .collect()
            }

            fn input_dispatch(&mut self, key: KeyEvent) -> $crate::field::InputResult {
                use $crate::field::{Field, InputResult};

                type Dispatch = fn(&mut __Form, KeyEvent) -> InputResult;

                const JUMP_TABLE: [Dispatch; __FIELDS] = [$(
                    |form, key| Field::input(&mut form.$id, key)
                ),*];
                JUMP_TABLE[self.__focus](self, key)
            }
    
            fn into_values(self) -> Self::Values {
                use $crate::field::Field;

                __Values {$(
                    $id: Field::into_value(self.$id), 
                )*}
            }

            fn values<'a>(&'a self) -> Self::BorrowedValues<'a> {
                use $crate::field::Field;

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
            __focus: 0, 
            __title: std::convert::Into::into($title), 
            // initialize fields with builder pattern using given arguments
            $($id: {
                let builder = <$type as $crate::field::Field>::builder()
                $(
                    .$arg_id($($arg_val)?)
                )*;
                builder.build()
            },)*
        };
        $crate::dialog::form::internal::Form::run_over(__form, $bg, $ctx, __validate)
    }}
}

pub mod internal {
    use std::{io, iter};
    use ratatui::{
        text::{Span, Line}, 
        style::{Style, Stylize}, 
    };
    use crate::{
        dialog::{self, *}, 
        field::InputResult, 
    };

    pub trait Form: Sized {
        type Values;
        type BorrowedValues<'a> where Self: 'a;

        fn title(&self) -> &str;

        fn max_focus(&self) -> usize;

        fn focus(&self) -> usize;
        fn set_focus(&mut self, focus: usize);

        fn into_values(self) -> Self::Values;
        fn values<'a>(&'a self) -> Self::BorrowedValues<'a>;

        fn input_dispatch(&mut self, key: KeyEvent) -> InputResult;
        fn format_dispatch(&self) -> Vec<Text>;

        fn run_over<G, T, U, V>(mut self, bg: &T, ctx: &mut Context<G>, mut validate: U)
            -> io::Result<Option<Self::Values>>
        where
            T: State, 
            U: FnMut(Self::BorrowedValues<'_>) -> std::result::Result<(), V>, 
            V: AsRef<str>, 
        {
            Ok(loop {
                let dialog = FormDialog(self);

                // run form dialog; if the user cancels, exit immediately
                let Some(out) = dialog.run_over(bg, ctx) else {
                    break None
                };
                self = out.0;

                match validate(self.values()) {
                    Ok(_) => break Some(self.into_values()), 
                    Err(e) => dialog::error(e, bg, ctx), 
                }
            })
        }
    }

    struct FormDialog<T>(T);

    impl<T: Form> Dialog for FormDialog<T> {
        fn format(&self) -> DrawInfo {
            let body: Vec<Line> = self.0.format_dispatch()
                .into_iter()
                .flat_map(|body| body.lines)
                .collect();
            DrawInfo {
                title: self.0.title().into(), 
                body: body.into(), 
                hint: "Press (enter) to submit, (esc) to cancel...".into(), 
                wrap: None, 
                ..Default::default()
            }
        }

        fn input(&mut self, key: KeyEvent) -> Signal {
            match key.code {
                KeyCode::Esc => Signal::Cancelled, 
                KeyCode::Enter => Signal::Done, 
                _ => {
                    let dispatch_result = self.0.input_dispatch(key);
                    let focus = self.0.focus();
    
                    match (dispatch_result, key.code) {
                        (InputResult::Ignored, KeyCode::Up) => {
                            self.0.set_focus(focus.saturating_sub(1));
                        }
                        (InputResult::Ignored, KeyCode::Down) => {
                            let focus = usize::min(focus + 1, self.0.max_focus());
                            self.0.set_focus(focus);
                        }
                        _ => (), 
                    };
                    Signal::Running
                }
            }
        }
    }

    pub fn format_field<'a>(name: &str, mut body: Text<'a>, focused: bool, align_to: usize) -> Text<'a> {
        let (delimiter, style) = match focused {
            true => (" : ", Style::new().bold()),
            false => (" │ ", Style::new()),
        };

        let padding = align_to.saturating_sub(name.len());
        let title: String = iter::repeat(' ')
            .take(padding)
            .chain(name.chars())
            .chain(delimiter.chars())
            .collect();
        let title = Span::styled(title, style);

        let mut lines = body.lines.iter_mut();

        if let Some(first) = lines.next() {
            first.spans.insert(0, title);
        }
        
        for line in lines {
            let indent: String = iter::repeat(' ')
                .take(align_to)
                .chain(delimiter.chars())
                .collect();
            line.spans.insert(0, indent.into())
        }
        body
    }
}

pub use form;
