/// Shows a form with specified fields in a [`Dialog`](crate::dialog::Dialog) to the user. 
#[macro_export]
macro_rules! run_form {
    {
        // fields
        $(
            $id:ident: $type:ty {
                $(
                    $arg_id:ident $(: $arg_val:expr)?
                ),+
                $(,)?
            }
        ),+
        $(,)?, 
        [title]: $title:expr, 
        [ctx]: $ctx:expr, 
        [background]: $bg:expr
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
                use $crate::input::{Field, form::internal};

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
                    color: Color::Cyan, 
                    body: self.format_fields(), 
                    hint: "Press (enter) to submit, (esc) to cancel...".into(), 
                }
            }
        }

        impl $crate::input::form::internal::Form for __Form<'_> {
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
        $crate::input::form::internal::Form::run(__form, $bg, $ctx, __validate)
    }}
}

pub use run_form;

#[doc(hidden)]
pub mod internal {
    use std::{io, iter};
    use ratatui::{
        text::{Line, Span}, 
        style::{Style, Stylize}, 
    };
    use crate::{
        prelude::*, 
        dialog::{self, *}, 
        input::Field, 
    };

    pub trait Form {
        type Values;
        type BorrowedValues<'a> where Self: 'a;

        fn into_values(self) -> Self::Values;
        fn values<'a>(&'a self) -> Self::BorrowedValues<'a>;

        fn run<G, T, U, V>(mut self, bg: &T, ctx: &mut Context<G>, mut validate: U)
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

#[cfg(test)]
mod test {
    #[test]
    fn foo() {
        let mut ctx = crate::context::Context::new().unwrap();
        let ctx = &mut ctx;
        let bg = &();

        let asdf = "Identifier";
        let bar = 1;
        let name = "123";

        let form = crate::input::run_form!{
            identifier: crate::input::Textbox{ name: asdf, value: "123    abc 1 😀😀😀1abc" }, 
            len: crate::input::Slider<u8>{ name: "Length" }, 
            salt: crate::input::Slider<u64>{ name: "Salt" }, 
            description: crate::input::Textbox{ name: "Description", hidden }, 
            asdf: crate::input::Checkbox{ name: "Checkbox" }, 
            [title]: format!("-- {name} --"), 
            [ctx]: ctx, 
            [background]: bg, 
            [validate]: |f| {
                if f.len == &bar {
                    Err(format!("Length must not be equal to {bar}."))
                } else {
                    Ok(())
                }
            }
        };
    }
}
