use std::io;
use tundra::{prelude::*, input::{Field, self, field::FieldWidget, Textbox}};

struct Config {
    enable_confirm: bool, 
}

struct MyState {
    checkbox: Foo, 
}

impl State for MyState {
    type Error = io::Error;
    type Global = Config;

    fn draw(&self, frame: &mut Frame) {
        FieldWidget(&self.checkbox);
    }

    fn input(&mut self, key: KeyEvent, ctx: &mut Context<Config>) -> Result<Signal, Self::Error> {
        todo!()
    }
}

fn main() -> io::Result<()> {
    let config = Config {
        enable_confirm: false, 
    };
    let context = &mut Context::with_global(config)?;

    let form = input::run_form!{
        asdf: Foo{ name: "Checkbox" }, 
        wasd: Textbox{ name: "Textbox", value: "123    abc 1 😀😀😀1abc" }, 
        [title]: "Add seed", 
        [ctx]: context, 
        [background]: &(), 
    };

    dialog::info("Hello world!", &(), context)?;

    Ok(())
}

#[derive(Default)]
struct Foo {
    name: &'static str, 
    value: bool, 
}

impl Foo {
    fn name(self, name: &'static str) -> Self {
        Foo{ name, ..self }
    } 
}

impl tundra::input::Field for Foo {
    type Value = bool;
    type Builder = Foo;

    fn name(&self) -> &str {
        self.name
    }

    fn input(&mut self, key: KeyEvent) {
        self.value = !self.value;
    }

    fn format(&self, selected: bool) -> ratatui::text::Line {
        use ratatui::{
            text::Line, 
            style::{Style, Stylize}
        };

        let val = match self.value {
            true => "✔", 
            false => "✖", 
        };
        let style = match selected {
            true => Style::new().bold(), 
            false => Style::new(), 
        };
        Line::styled(val, style)
    }

    fn value(&self) -> &Self::Value {
        &self.value
    }

    fn into_value(self) -> Self::Value {
        self.value
    }
}

impl tundra::input::Build<Foo> for Foo {
    fn build(self) -> Foo {
        self
    }
}
