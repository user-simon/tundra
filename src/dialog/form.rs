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
/// };
/// if let Some(values) = values {
///     // type annotation is not required
///     let location: String = values.location;
///     let rent: u32 = values.rent;
///     let pets_allowed: bool = values.pets_allowed;
/// }
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
/// };
/// match values {
///     Some(_) => { /* form submitted -> login success */ }
///     None => { /* form cancelled -> login failure */ }
/// }
/// ```
#[macro_export]
macro_rules! form {
    [
        // A comma-separated list of fields
        $(
            $id:ident: $type:ty {
                // Parameters for each field using builder pattern methods
                $(
                    $arg_id:ident $(: $arg_val:expr)?
                ),+
                $(,)?
            }
            $(
                if $control:expr => $control_err:literal
            )*
        ),+, 
        $([$meta_id:ident]: $meta_expr:expr),*
        $(,)?
    ] => {{
        use std::{
            convert::Into as __Into, 
            borrow::Cow as __Cow, 
            result::Result as __Result, 
            option::Option as __Option, 
        };
        use $crate::{
            dialog::form::internal as __internal, 
            field::Field as __Field, 
        };

        #[allow(non_camel_case_types)]
        enum __Indices {$(
            $id, 
        )*}

        #[allow(dead_code)]
        struct __Values {$(
            $id: <$type as __Field>::Value,
        )*}
        
        #[allow(dead_code)]
        struct __BorrowedValues<'a> {$(
            $id: &'a <$type as __Field>::Value,
        )*}

        struct __Control<'a> {$(
            $id: __internal::Control<'a, $type>, 
        )*}

        struct __Form<'a> {
            __focus: usize, 
            __control: __Control<'a>, 
            __title: __Cow<'a, str>, 
            __message: __Cow<'a, str>, 
            $(
                $id: $type, 
            )*
        }

        const __FIELDS: usize = [$(__Indices::$id),*].len();

        impl __Form<'_> {
            fn values(&self) -> __BorrowedValues {
                __BorrowedValues {$(
                    $id: __Field::value(&self.$id), 
                )*}
            }

            fn into_values(self) -> __Values {
                __Values {$(
                    $id: __Field::into_value(self.$id), 
                )*}
            }
        }

        impl<'a> $crate::dialog::Dialog for __Form<'a> {
            type Out = __Option<Self>;

            fn format(&self) -> $crate::dialog::DrawInfo {
                use std::{
                    default::Default as _, 
                    convert::From as _, 
                };
                use ratatui::text::Text;
                use $crate::dialog::DrawInfo;

                let name_lengths = [$(
                    __Field::name(&self.$id).len(), 
                )*];
                let max_name = name_lengths
                    .into_iter()
                    .max()
                    .unwrap_or(0);
                let texts = [
                    $({
                        let focus = __Indices::$id as usize == self.__focus;
                        let name = __Field::name(&self.$id);
                        let body = __Field::format(&self.$id, focus);
                        let error = self.__control.$id.is_err();
                        __internal::format_field(name, body, focus, max_name, error)
                    },)*
                ];
                let header = (self.__message.len() != 0)
                    .then(|| [Text::from(self.__message.as_ref()), Text::from("")])
                    .into_iter()
                    .flatten();
                let body = header
                    .chain(texts)
                    .fold(Text::default(), |mut acc, body| {
                        acc.extend(body);
                        acc
                    });

                DrawInfo {
                    title: __Cow::from(self.__title.as_ref()), 
                    body, 
                    hint: __Cow::from("Press (enter) to submit, (esc) to cancel..."), 
                    wrap: __Option::None, 
                    ..DrawInfo::default()
                }
            }
            
            fn input(mut self, key: KeyEvent) -> $crate::Signal<Self> {
                use $crate::{Signal, field::InputResult};

                type Dispatch<'a> = fn(&mut __Form, KeyEvent) -> InputResult;

                const JUMP_TABLE: [Dispatch; __FIELDS] = [$(
                    |form, key| __internal::input_dispatch(&mut form.$id, &mut form.__control.$id, key)
                ),*];

                match key.code {
                    KeyCode::Esc => Signal::Return(None), 
                    KeyCode::Enter => Signal::Return(Some(self)), 
                    _ => {
                        let dispatch_result = JUMP_TABLE[self.__focus](&mut self, key);
        
                        match (dispatch_result, key.code) {
                            (InputResult::Ignored, KeyCode::Up) => {
                                self.__focus = self.__focus.saturating_sub(1);
                            }
                            (InputResult::Ignored, KeyCode::Down) => {
                                self.__focus = usize::min(self.__focus + 1, __FIELDS - 1);
                            }
                            _ => (), 
                        };
                        Signal::Continue(self)
                    }
                }
            }
        }

        fn __run<'a, T>(
            mut form: __Form<'a>, 
            bg: &impl $crate::State, 
            ctx: &mut $crate::Context<T>, 
            mut validate: impl std::ops::FnMut(__BorrowedValues) -> __Result<(), __Cow<'a, str>>, 
        ) -> __Option<__Values> {
            use $crate::dialog::Dialog as _;

            loop {
                // run form dialog; if the user cancels, exit immediately
                let __Option::Some(out) = form.run_over(bg, ctx) else {
                    break None
                };
                form = out;

                let control_result = __internal::format_control_error(&[$(
                    (__Field::name(&form.$id), form.__control.$id.updated_result(&form.$id)), 
                )*]);
                let validation_result = match control_result {
                    __Result::Ok(()) => validate(form.values()), 
                    __Result::Err(e) => __Result::Err(__Cow::from(e)), 
                };
                match validation_result {
                    __Result::Ok(()) => break __Option::Some(form.into_values()), 
                    __Result::Err(e) => $crate::dialog::error(e, bg, ctx), 
                }
            }
        }

        struct __Meta<'a, A, B, C, D, E, X>
        where
            A: __Into<__Cow<'a, str>>, 
            D: __Into<__Cow<'a, str>>, 
            E: std::ops::FnMut(__BorrowedValues) -> __Result<(), X>, 
            X: __Into<__Cow<'a, str>>, 
        {
            title: A, 
            context: &'a mut $crate::Context<B>, 
            background: &'a C, 
            message: D, 
            validate: E, 
        }

        let mut meta = $crate::parse_form_meta!{
            __Meta {
                $($meta_id: $meta_expr,)*
            } else {
                message: "", 
                validate: |_| __Result::<(), __Cow<'_, str>>::Ok(()), 
            }
        };

        let control = __Control {
            $($id: __internal::Control {
                callback: &|value: &<$type as __Field>::Value| {
                    $(
                        if $control(value) {
                            return __Result::Err(__Cow::from($control_err))
                        }
                    )*
                    __Result::Ok(())
                }, 
                state: __internal::ControlState::Unknown, 
            },)*
        };
        let validate = |values: __BorrowedValues| (meta.validate)(values).map_err(__Cow::from);

        let form = __Form {
            __focus: 0, 
            __control: control, 
            __title: __Cow::from(meta.title), 
            __message: __Cow::from(meta.message), 
            // initialise fields with builder pattern using given arguments
            $($id: {
                let builder = <$type as __Field>::builder()
                $(
                    .$arg_id($($arg_val)?)
                )*;
                $crate::field::Build::build(builder)
            },)*
        };
        __run(form, meta.background, meta.context, validate)
    }}
}

#[macro_export]
#[doc(hidden)]
macro_rules! parse_form_meta {
    // Entry point. 
    [
        $struct:ident {
            $($meta_id:ident: $meta_val:expr,)*
        } else {
            $($default_id:ident: $default_val:expr,)*
        }
    ] => {
        $crate::parse_form_meta!{@impl $struct ($)
            <$(($default_id, $default_val))*>
            <>
            $(($meta_id, $meta_val))*
        }
    };
    // Base case: takes all meta-field name-value pairs along with the required defaults and constructs the
    // struct using them. 
    [@impl $struct:ident $_:tt
        // Required defaults
        <$(($default_id:ident, $default_val:expr))*>
        // Name-value pairs
        <$(($id:ident, $val:expr))*>
    ] => {
        $struct {
            $(
                $id: $val, 
            )*
            $(
                $default_id: $default_val, 
            )*
        }
    };
    // Recursive case: for each provided name-value pair, filters out the corresponding default (if one
    // exists). 
    [@impl $struct:ident ($s:tt)
        // Remaining defaults that haven't yet gotten filtered out
        <$(($default_id:ident, $default_val:expr))*>
        // Accumulated name-value pairs. "Stored" here so we can access them in the base case
        <$(($acc_id:ident, $acc_val:expr))*>
        // Name-value pairs yet to be processed
        ($id:ident, $val:expr) $($tail:tt)*
    ] => {{
        // macro to go through all the remaining defaults, accumulate the ones that don't have a $default_id
        // equal to $id, and then recursively call parse_form_meta! to process the rest of the name-value
        // pairs. this has to be a nested macro to hard-code $id in its pattern (and the $s argument is
        // needed to insert $ without having the outer macro try to expand it). note that this amount of
        // TT-munching probably isn't ideal from a compile-time performance standpoint, but I can't think of
        // a better way of doing it without compromising usability and error handling
        macro_rules! __filter {
            // base case: $id has been filtered from the accumulated defaults; proceed to the next $id
            [<$s(($s ID:ident, $s VAL:expr))*>] => {
                $crate::parse_form_meta!{@impl $struct ($s)
                    <$s(($s ID, $s VAL))*>
                    <$(($acc_id, $acc_val))* ($id, $val)>
                    $($tail)*
                }
            };
            // recursive case where the $default_id is equal to $id: ignore the default and process the rest
            [<$s(($s ID:ident, $s VAL:expr))*> ($id, $s _:tt) $s($s TAIL:tt)*] => {
                __filter!(<$s(($s ID, $s VAL))*> $s($s TAIL)*)
            };
            // recursive case otherwise: add the default to the accumulator and process the rest
            [<$s(($s ID:ident, $s VAL:expr))*> $s HEAD:tt $s($s TAIL:tt)*] => {
                __filter!(<$s(($s ID, $s VAL))* $s HEAD> $s($s TAIL)*)
            };
        }
        __filter!(<> $(($default_id, $default_val))*)
    }};
}

#[test]
fn playground() {
    use crate::prelude::*;
    use crate::field::*;

    let mut ctx = Context::new().unwrap();

    form!{
        location: Textbox{ name: "Location" }
            if str::is_empty => "Value must be non-empty"
            if |value| value == "asdf" => "Must not be equal to asdf", 
        rent: Slider<u32>{ name: "Monthly rent", range: 1..=5000, step: 1 }, 
        pets_allowed: Checkbox{ name: "Pets allowed" }, 

        [background]: &(), 
        [context]: &mut ctx, 
        [title]: "Register Rent Unit", 
        [validate]: |values| {
            if values.location.len() == *values.rent as usize {
                Err("Error")
            } else {
                Ok(())
            }
        }, 
    };
}

pub mod internal {
    use std::iter;
    use ratatui::{
        style::{Style, Stylize}, 
        text::{Line, Span}, 
    };
    use crate::{dialog::*, field::{Field, InputResult}};

    pub enum ControlState<'a> {
        Unknown, 
        Ok, 
        Err(Cow<'a, str>), 
    }

    pub struct Control<'a, T: Field> {
        pub callback: &'a dyn Fn(&T::Value) -> Result<(), Cow<'a, str>>, 
        pub state: ControlState<'a>, 
    }

    impl<'a, T: Field> Control<'a, T> {
        pub fn updated_result<'b>(&'b mut self, field: &T) -> Result<(), &'b str> {
            if let ControlState::Unknown = self.state {
                self.update(field);
            }
            match &self.state {
                ControlState::Unknown => unreachable!(),
                ControlState::Ok => Ok(()),
                ControlState::Err(e) => Err(e),
            }
        }

        pub fn update(&mut self, field: &T) {
            self.state = match (self.callback)(field.value()) {
                Ok(()) => ControlState::Ok, 
                Err(err) => ControlState::Err(err), 
            };
        }

        pub const fn is_err(&self) -> bool {
            match self.state {
                ControlState::Unknown => false,
                ControlState::Ok => false,
                ControlState::Err(_) => true,
            }
        }
    }

    #[inline(never)]
    pub fn input_dispatch<T: Field>(field: &mut T, control: &mut Control<T>, key: KeyEvent) -> InputResult {
        let result = field.input(key);
        
        if let InputResult::Updated = result {
            control.update(&field);
        }
        result
    }

    #[inline(never)]
    pub fn format_field<'a>(name: &'a str, mut body: Text<'a>, focused: bool, align_to: usize, error: bool)
        -> Text<'a>
    {
        // make sure we have at least one line to put the title in
        if body.lines.is_empty() {
            body.lines.push(Line::default())
        }

        // add title to first line
        {
            let delimiter = match focused {
                true => " : ", 
                false => " │ ", 
            };
            let style = {
                let style = Style::default();
                let style = match focused {
                    true => style.bold(), 
                    false => style, 
                };
                let style = match error {
                    true => style.red(), 
                    false => style, 
                };
                style
            };
            let padding: Span = iter::repeat(' ')
                .take(align_to.saturating_sub(name.len()))
                .collect::<String>()
                .into();
            let name = Span::styled(name, style);
            let delimiter = Span::raw(delimiter);
            let title = [padding, name, delimiter];
            body.lines[0].spans.splice(0..0, title);
        };

        // indent remaining lines
        for line in &mut body.lines[1..] {
            let indent: String = iter::repeat(' ')
                .take(align_to)
                .chain(" │ ".chars())
                .collect();
            line.spans.insert(0, indent.into());
        }
        body
    }

    #[inline(never)]
    pub fn format_control_error(results: &[(&str, Result<(), &str>)]) -> Result<(), String> {
        let messages: Vec<String> = results
            .iter()
            .filter_map(|(name, state)| state
                .as_ref()
                .err()
                .map(|e| (name, e))
            )
            .map(|(name, error)| format!("{name}: {error}"))
            .collect();
        match messages.is_empty() {
            true => Ok(()), 
            false => Err(messages.join("\n")), 
        }
    }
}

pub use form;
