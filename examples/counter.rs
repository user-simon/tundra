use std::io;
use ratatui::widgets::Paragraph;
use tundra::prelude::*;

struct Counter {
    value: u32, 
}

impl State for Counter {
    type Result<T> = T;
    type Out = u32;
    type Global = ();
    
    fn draw(&self, frame: &mut Frame) {
        let widget = Paragraph::new(self.value.to_string());
        frame.render_widget(widget, frame.size());
    }
    
    fn input(mut self, key: KeyEvent, ctx: &mut Context) -> Signal<Self> {
        match key.code {
            KeyCode::Up    => self.value += 1, 
            KeyCode::Tab   => self.value *= counter(ctx), 
            KeyCode::Enter => return Signal::Return(self.value), 
            _ => (), 
        }
        Signal::Continue(self)
    }
}

pub fn counter(ctx: &mut Context) -> u32 {
    Counter{ value: 0 }.run(ctx)
}

fn main() -> io::Result<()> {
    let ctx = &mut Context::new()?;
    let value = counter(ctx);
    dialog::info(format!("You entered {value}! Why?"), &(), ctx);

    Ok(())
}
