#![doc(hidden)]

/// Displays a user input form with specified [fields](crate::field) in a [dialog](crate::dialog) to the
/// user. 
/// 
/// The easiest way to get a hang of how to use the macro is to just examine the [examples](#examples), but a
/// (somewhat) more formal description of the syntax is provided as well. 
/// 
/// The syntax expects first a set of fields, followed by a set of metadata. The items should be
/// comma-separated (trailing commas are optional). 
/// 
/// 
/// # Fields
/// 
/// A field consists of:
/// - An identifier; used to reference the entered value. 
/// - A field type; any type that implements [`Field`](crate::field::Field). 
/// - A set of parameters used when instantiating the field; these are translated into methods on the
/// [field builder](crate::field::Build). There are two kinds of parameters allowed: those with one argument
/// and those with none. Those with one argument are specified as `IDENTIFIER: VALUE`. Those with no argument
/// are specified simply as `IDENTIFIER`. 
/// - (Optional) a set of control statements. A more detailed description of these are given
/// [below](#field-validation). 
/// 
/// The syntax for declaring a field follows the form: `IDENTIFIER: TYPE{ PARAMS } CONTROL_STMTS`. 
/// 
/// For example, to declare a textbox without validation with identifier `password`, and parameters
/// `name = "Password"`, `value = "admin"`, and `hidden` (no argument): 
/// ```no_run
/// # use tundra::{prelude::*, field::Textbox};
/// # dialog::form!{
/// password: Textbox{ name: "Password", value: "admin", hidden }, 
/// # [title]: "", 
/// # [context]: &mut Context::new().unwrap(), 
/// # [background]: &(), 
/// # };
/// ```
/// 
/// The DSL `Textbox{ name: "Password", value: "admin", hidden }` gets (loosely) translated as: 
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
/// See the [`field::Build`](crate::field::Build) module for more information on builders. 
/// 
/// 
/// # Metadata
/// 
/// In addition to the fields of the form, some other pieces of data must be supplied in order to show the 
/// form. These include a reference to the current [context](crate::Context) and the title of the dialog box. 
/// These pieces of metadata are supplied with syntax of the form `[IDENTIFIER]: VALUE`. For example, to
/// provide the title of the form as `"My form"`: 
/// ```no_run
/// # use tundra::{prelude::*, field::Checkbox};
/// # dialog::form!{
/// # foo: Checkbox{ name: "" }, 
/// # [context]: &mut Context::new().unwrap(), 
/// # [background]: &(), 
/// [title]: "", 
/// # };
/// ```
/// 
/// The following metadata can be defined in any order: 
/// - `title` (required); the user-visible title of the dialog box. Should be `impl Into<Cow<str>>`. 
/// - `context` (required); the current [context](crate::Context). Should be `&mut Context<_>`. 
/// - `background` (required); the state shown underneath the dialog box. Should be `&impl State`. 
/// - `message`; user-visible string of text displayed above the fields. Should be `impl Into<Cow<str>>`. 
/// - `validate`; validation function over the values entered by the user. See [below](#form-validation). 
/// 
/// 
/// # Validation
/// 
/// Two kinds of validations are supported: field validation and form validation. Both are optional and place
/// requirements on the values entered by the user, but operate on different scopes. Field validation is
/// performed whenever the value of a field changes and is local to the field. Form validation is performed
/// whenever the user attempts to submit the form and has global access to all fields. 
/// 
/// Since field validation is more localised, it can be used to provide more intuitive feedback by turning
/// the name of the offending field red. 
/// 
/// Prefer field validation for simple checks that require only local knowledge of the fields, and form
/// validation for checks that are either more complicated or require global knowledge of the fields (such
/// as comparing the values of two fields against each other). 
/// 
/// A more in-depth description of the two kinds of validation is provided below. 
/// 
/// 
/// ### Field validation
/// 
/// Field validation is provided on a per-field basis using control statements. Each control statement
/// defines a boolean function over the entered value (the error condition) and an error message to be shown
/// if the function returns `true`. Any number of control statements can be given per field. 
/// 
/// Whenever the value of a field is changed or the form is submitted (whichever happens first), it is
/// checked against the error condition. If the error condition triggers, the name of the field turns red,
/// and the error message is displayed if the user attempts to submit the form. For some fields (textboxes in
/// particular), the error condition could be checked quite frequently and should therefore be fairly fast.
/// For more complicated validation, prefer [form validation](#form-validation), which is only checked once
/// the form is submitted. 
/// 
/// The syntax of a control statement follows the form `if ERR_CONDITION => MESSAGE`, where `ERR_CONDITION`
/// is either a path to a function (e.g. `str::is_empty`) or a closure (e.g. `|&value| value == 123`), and
/// `MESSAGE` is a value that implements `Into<Cow<str>>`. Several control statements are given by repeating
/// the syntax, delimited by a space or newline. Note that the comma that separates different fields in the
/// macro is given after all control statements. 
/// 
/// For example, to require that the password in the example from before is non-empty and not equal to
/// "password1": 
/// ```no_run
/// # use tundra::{prelude::*, field::Textbox};
/// # dialog::form!{
/// password: Textbox{ name: "Password", value: "admin", hidden }
///     if str::is_empty => "Password must not be empty"
///     if |value| value == "password1" => "You can choose a better password than that!", 
/// # [title]: "", 
/// # [context]: &mut Context::new().unwrap(), 
/// # [background]: &(), 
/// # };
/// ```
/// 
/// 
/// ### Form validation
/// 
/// Form validation is provided through a function over the values of all fields. It can be used to place
/// requirements on the relationships between fields or in cases where field validation is too complex to be
/// performed each time a field is updated. 
/// 
/// The validation function accepts as argument a struct containing a reference to the values of all fields. 
/// Since this struct is unspellable by application code, the function must be a closure. It should return a
/// value of `Result<(), impl AsRef<str>>`; `Ok` on validation success, and `Err` with a given error message
/// otherwise. 
/// 
/// To enable form validation, supply a closure as the `validate` metadatum. For example, to validate that
/// the value of slider `foo` is less than the value of slider `bar`: 
/// ```no_run
/// # use tundra::{prelude::*, field::Slider};
/// # dialog::form!{
/// # foo: Slider<u8>{ name: "" }, 
/// # bar: Slider<u8>{ name: "" }, 
/// # [title]: "", 
/// # [context]: &mut Context::new().unwrap(), 
/// # [background]: &(), 
/// [validate]: |values| if values.foo >= values.bar {
///     Err("Foo must be less than bar!")
/// } else {
///     Ok(())
/// }
/// # };
/// ```
/// Note that the validation function closure may implement [`FnMut`], and can therefore cache values
/// computed during validation. 
/// 
/// 
/// # Returns
/// 
/// The return value of the macro is an [`Option`]: 
/// - `Some` if the form was submitted. Contains the values of all fields as members of an unspellable
/// struct. The identifiers of the values are the same as the corresponding fields. 
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
///     location: Textbox{ name: "Location" } if str::is_empty => "Value required", 
///     rent: Slider<u32>{ name: "Monthly rent", range: 1..=5000, step: 50 }, 
///     pets_allowed: Checkbox{ name: "Pets allowed" }, 
///     [title]: "Register Rent Unit", 
///     [context]: ctx, 
///     [background]: current_state, 
/// };
/// if let Some(values) = values {
///     // type annotations are not required
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

        // used to look up the index of a field by its name via `__Indices::$id as usize`. 
        #[allow(non_camel_case_types)]
        enum __Indices {$(
            $id, 
        )*}

        // holds the owned values of all fields once the form is submitted. 
        #[allow(dead_code)]
        struct __Values {$(
            $id: <$type as __Field>::Value,
        )*}

        // holds the borrowed values of all fields for form validation. 
        #[allow(dead_code)]
        struct __BorrowedValues<'a> {$(
            $id: &'a <$type as __Field>::Value,
        )*}

        // holds control callbacks and state for all fields, for implementing field validation. 
        struct __Control<'a> {$(
            $id: __internal::Control<'a, $type>, 
        )*}

        // the form dialog itself. contains the input-fields as regular struct-fields, and some meta-data
        // required for the [`Dialog`] implementation.  
        struct __Form<'a> {
            __focus: usize, 
            __control: __Control<'a>, 
            __title: __Cow<'a, str>, 
            __message: __Cow<'a, str>, 
            $(
                $id: $type, 
            )*
        }

        // the number of fields in the form. 
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

        impl $crate::dialog::Dialog for __Form<'_> {
            type Out = __Option<Self>;

            fn format(&self) -> $crate::dialog::DrawInfo {
                let name_lengths = [$(
                    __Field::name(&self.$id).len(), 
                )*];
                let max_name = name_lengths
                    .into_iter()
                    .max()
                    .unwrap_or(0);
                let mut fields = [
                    $({
                        let focus = __Indices::$id as usize == self.__focus;
                        let name = __Field::name(&self.$id);
                        let body = __Field::format(&self.$id, focus);
                        let error = self.__control.$id.is_err();
                        __internal::format_field(name, body, focus, max_name, error)
                    },)*
                ];
                __internal::format_dialog(&mut fields, self.__message.as_ref(), self.__title.as_ref())
            }
            
            fn input(mut self, key: KeyEvent) -> $crate::Signal<Self> {
                use $crate::{Signal, field::InputResult};

                type Dispatch<'a> = fn(&mut __Form, KeyEvent) -> InputResult;

                // holds a function pointer that dispatches to the `Field::input` implementation
                // corresponding to each field. this can then be indexed by `self.__focus` to dispatch the
                // input event to the correct field
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

                // perform field validation
                let control_result = __internal::format_control_error(&[$(
                    (__Field::name(&form.$id), form.__control.$id.updated_result(&form.$id)), 
                )*]);
                // if field validation passes, perform form validation
                let validation_result = match control_result {
                    __Result::Ok(()) => validate(form.values()), 
                    __Result::Err(e) => __Result::Err(__Cow::from(e)), 
                };
                // if either validation fails, show error message and continue. otherwise, return values
                match validation_result {
                    __Result::Ok(()) => break __Option::Some(form.into_values()), 
                    __Result::Err(e) => $crate::dialog::error(e, bg, ctx), 
                }
            }
        }

        // temporary container for all metadata, used for parsing. see [`parse_form_meta!`]
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

        // instantiates the struct above with the given metadata, using the defaults defined under `else` for
        // optional metadata that were not given
        let mut meta = $crate::parse_form_meta!{
            __Meta {
                $($meta_id: $meta_expr,)*
            } else {
                message: "", 
                validate: |_| __Result::<(), __Cow<'_, str>>::Ok(()), 
            }
        };

        // field validation. for each field, creates a callback `Control::callback` bundling all
        // control-statements for the field. this callback is invoked each time the field is updated. if the
        // callback results in error, it is saved in `Control::state`
        let control = __Control {
            $($id: __internal::Control {
                callback: &|value: &<$type as __Field>::Value| {
                    $(
                        if $control(value) {
                            return __Result::Err(__Cow::from($control_err))
                        }
                    )*
                    let _ = value;
                    __Result::Ok(())
                }, 
                state: __internal::ControlState::Unknown, 
            },)*
        };

        // form validation. simply invokes `__Meta::validate`
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

/// Utility macro for parsing form metadata as a struct instantiation. 
/// 
/// The problem being solved is (a) having a set of required fields and a set of optional fields --- the
/// latter having defined default values --- and (b) allowing them to be given in any order. Hard-coding the
/// metadata in the [`form`] macro arguments provides (a), but not (b). Making the metadata translate
/// directly to a struct instantiation provides (b), but not (a). 
/// 
/// This macro attempts to solve this by:
/// 1. Taking the metadata given by the application along with the defaults for all optional metadata. 
/// 2. Recursively removing the defaults for the optional metadata that were given by the application. This
/// provides (a). 
/// 3. Taking the defined metadata and the remaining defaults (those that were left undefined by the
/// application) and using them to instantiate a struct. This provides (b). 
/// 
/// This macro is agnostic to the struct being instantiated (taking the name of it as parameter) and its
/// contents. 
/// 
/// The filtering is implemented using a nested macro definition, involves a lot of TT-munching, and has
/// complexity `O(m · n)` --- where `m` is the number of metadata given by the application, and `n` is the
/// number of defaults --- and is therefore likely very inefficient. Further work is needed to find a better
/// way of accomplishing the same thing without sacrificing usability and error-handling. 
/// 
/// 
/// # Examples
/// 
/// Assume that we have `Meta` defined as: 
/// 
/// ```
/// struct Meta {
///     required: u32, 
///     optional: &'static str, 
/// }
/// ```
/// 
/// Without `optional` defined: 
/// ```
/// # use tundra::parse_form_meta;
/// # struct Meta {
/// #     required: u32, 
/// #     optional: &'static str, 
/// # }
/// parse_form_meta!{
///     Meta {
///         required: 123, 
///     } else {
///         optional: "default", 
///     }
/// }
/// # ;
/// // yields:
/// Meta {
///     required: 123, 
///     optional: "default", 
/// }
/// # ;
/// ```
/// 
/// With `optional` defined: 
/// ```
/// # use tundra::parse_form_meta;
/// # struct Meta {
/// #     required: u32, 
/// #     optional: &'static str, 
/// # }
/// parse_form_meta!{
///     Meta {
///         required: 123, 
///         optional: "custom", 
///     } else {
///         optional: "default", 
///     }
/// }
/// # ;
/// // yields:
/// Meta {
///     required: 123, 
///     optional: "custom", 
/// }
/// # ;
/// ```
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

/// Private utilities used for implementing the form macro. 
/// 
/// Most of this consists of stuff that could be factored out from the form macro body to reduce codegen. 
pub mod internal {
    use ratatui::{
        style::{Style, Stylize}, 
        text::{Line, Span}, 
    };
    use crate::{dialog::*, field::{Field, InputResult}};

    /// Holds the last known control state; [`ControlState::Unknown`] if it has never been tested. 
    pub enum ControlState<'a> {
        Unknown, 
        Ok, 
        Err(Cow<'a, str>), 
    }

    /// Stores the callback to validate a field and the last known result of that callback. 
    pub struct Control<'a, T: Field> {
        pub callback: &'a dyn Fn(&T::Value) -> Result<(), Cow<'a, str>>, 
        pub state: ControlState<'a>, 
    }

    impl<'a, T: Field> Control<'a, T> {
        /// Makes sure that the field has been validated and returns the last known error. 
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

        /// Validates a field by updating [`Control::state`]. 
        pub fn update(&mut self, field: &T) {
            self.state = match (self.callback)(field.value()) {
                Ok(()) => ControlState::Ok, 
                Err(err) => ControlState::Err(err), 
            };
        }

        /// Whether the field is *known* to be invalid. 
        pub const fn is_err(&self) -> bool {
            match self.state {
                ControlState::Unknown => false,
                ControlState::Ok => false,
                ControlState::Err(_) => true,
            }
        }
    }

    /// Delegates to [`Field::input`] and updates the [`Control::state`]. 
    #[inline(never)]
    pub fn input_dispatch<T: Field>(field: &mut T, control: &mut Control<T>, key: KeyEvent) -> InputResult {
        let result = field.input(key);
        
        if let InputResult::Updated = result {
            control.update(&field);
        }
        result
    }

    /// Formats a field for use in a form. 
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
            let padding: Span = std::iter::repeat(' ')
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
            let indent: String = std::iter::repeat(' ')
                .take(align_to)
                .chain(" │ ".chars())
                .collect();
            line.spans.insert(0, indent.into());
        }
        body
    }

    /// Formats the form dialog from the formatted fields. 
    #[inline(never)]
    pub fn format_dialog<'a>(fields: &mut [Text<'a>], message: &'a str, title: &'a str) -> DrawInfo<'a> {
        let message = (message.len() != 0)
            .then(|| [Text::from(message), Text::from("")])
            .into_iter()
            .flatten();
        let fields = fields
            .into_iter()
            .map(std::mem::take);
        let body = message
            .chain(fields)
            .fold(Text::default(), |mut acc, body| {
                acc.extend(body);
                acc
            });
        DrawInfo {
            title: Cow::from(title), 
            body, 
            hint: Cow::from("Press (enter) to submit, (esc) to cancel..."), 
            wrap: None, 
            ..DrawInfo::default()
        }
    }

    /// Takes a set of control states and constructs an error message from them. 
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
