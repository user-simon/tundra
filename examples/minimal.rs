// ↓ REMOVE THIS ↓
#![allow(unused_variables)]

use std::io;
use tundra::prelude::*;

struct MyState;

impl State for MyState {
    type Result<T> = T;
    type Out = ();
    type Global = ();

    fn draw(&self, frame: &mut Frame) {
        todo!("Draw the state using Ratatui")
    }

    fn input(self, key: KeyEvent, ctx: &mut Context) -> Signal<Self> {
        todo!("Handle key press events")
    }
}

fn main() -> io::Result<()> {
    let ctx = &mut Context::new()?;
    let state = MyState;
    state.run(ctx);

    Ok(())
}
