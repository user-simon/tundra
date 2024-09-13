use std::io;
use ratatui::widgets::Paragraph;
use tundra::prelude::*;

struct Tally {
    value: u32, 
}

impl State for Tally {
    type Result<T> = T;
    type Out = u32;
    type Global = ();
    
    fn draw(&self, frame: &mut Frame) {
        let widget = Paragraph::new(self.value.to_string());
        frame.render_widget(widget, frame.area());
    }
    
    fn input(mut self, key: KeyEvent, ctx: &mut Context) -> Signal<Self> {
        match key.code {
            KeyCode::Up    => self.value += 1, 
            KeyCode::Tab   => self.value *= tally(ctx), 
            KeyCode::Enter => return Signal::Return(self.value), 
            _ => (), 
        }
        Signal::Continue(self)
    }
}

pub fn tally(ctx: &mut Context) -> u32 {
    Tally{ value: 0 }.run(ctx)
}

fn main() -> io::Result<()> {
    let ctx = &mut Context::new()?;
    let value = tally(ctx);
    dialog::info(format!("You entered {value}! Why?"), &(), ctx);

    Ok(())
}
