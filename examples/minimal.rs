use std::io;
use ratatui::widgets::Paragraph;
use tundra::prelude::*;

struct MyState {
    counter: isize, 
}

impl State for MyState {
    type Error = io::Error;
    type Global = ();

    fn draw(&self, frame: &mut Frame) {
        let counter_string = format!("{}", self.counter);
        let widget = Paragraph::new(counter_string);
        frame.render_widget(widget, frame.size());
    }

    fn input(&mut self, key: KeyEvent, _ctx: &mut Context) -> io::Result<Signal> {
        match key.code {
            KeyCode::Up    => self.counter += 1, 
            KeyCode::Down  => self.counter -= 1, 
            KeyCode::Enter => return Ok(Signal::Done), 
            _ => (), 
        }
        Ok(Signal::Running)
    }
}

fn main() -> io::Result<()> {
    let context = &mut Context::new()?;
    let state = MyState{ counter: 0 };
    
    state.run(context)?;

    Ok(())
}
