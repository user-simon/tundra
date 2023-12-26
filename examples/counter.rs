use std::io;
use ratatui::widgets::Paragraph;
use tundra::prelude::*;

struct Counter {
    value: u32, 
}

impl State for Counter {
    type Error = io::Error;
    type Global = ();
    
    fn draw(&self, frame: &mut Frame) {
        let widget = Paragraph::new(format!("{}", self.value));
        frame.render_widget(widget, frame.size());
    }
    
    fn input(&mut self, key: KeyEvent, ctx: &mut Context) -> io::Result<Signal> {
        match key.code {
            KeyCode::Up    => self.value += 1, 
            KeyCode::Tab   => self.value += counter(ctx)?, 
            KeyCode::Enter => return Ok(Signal::Done), 
            KeyCode::Esc   => return Ok(Signal::Cancelled), 
            _ => (), 
        }
        Ok(Signal::Running)
    }
}

pub fn counter(ctx: &mut Context) -> io::Result<u32> {
    let counter = Counter{ value: 0 }.run(ctx)?;
    let value = counter
        .map(|c| c.value)
        .unwrap_or(0);
    Ok(value)
}

fn main() -> io::Result<()> {
    let ctx = &mut Context::new()?;
    let value = counter(ctx)?;
    dialog::info(format!("You entered {value}! Why?"), &(), ctx)
}
