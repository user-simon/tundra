use std::io;
use ratatui::widgets::Paragraph;
use tundra::prelude::*;

struct Counter {
    value: u32, 
}

impl State for Counter {
    type Result<T> = T;
    type Global = ();
    
    fn draw(&self, frame: &mut Frame) {
        let widget = Paragraph::new(format!("{}", self.value));
        frame.render_widget(widget, frame.size());
    }
    
    fn input(&mut self, key: KeyEvent, ctx: &mut Context) -> Signal {
        match key.code {
            KeyCode::Up    => self.value += 1, 
            KeyCode::Tab   => self.value += counter(ctx), 
            KeyCode::Enter => return Signal::Done, 
            KeyCode::Esc   => return Signal::Cancelled, 
            _ => (), 
        }
        Signal::Running
    }
}

pub fn counter(ctx: &mut Context) -> u32 {
     Counter{ value: 0 }
         .run(ctx)
         .map(|counter| counter.value)
         .unwrap_or(0)
}

fn main() -> io::Result<()> {
    let ctx = &mut Context::new()?;
    let value = counter(ctx);
    dialog::info(format!("You entered {value}! Why?"), &(), ctx);

    Ok(())
}
