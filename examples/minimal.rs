use std::io;
use tundra::prelude::*;

struct MyState;

impl State for MyState {
    type Error = io::Error;
    type Global = ();

    fn draw(&self, _frame: &mut Frame) {
        todo!("Draw the state using Ratatui")
    }

    fn input(&mut self, _key: KeyEvent, _ctx: &mut Context) -> io::Result<Signal> {
        todo!("Handle key press events")
    }
}

fn main() -> io::Result<()> {
    let ctx = &mut Context::new()?;
    let state = MyState;
    state.run(ctx)?;

    Ok(())
}
