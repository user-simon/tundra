use std::io;
use ratatui::{style::Color, text::Line};
use tundra::{prelude::*, dialog::*};

/// Asks the user to select a label from its index. 
struct NumberSelect<'a> {
    /// Message shown at the top of the dialog. 
    msg: &'a str, 
    /// Labels corrsponding to each index. 
    labels: &'a [&'a str], 
}

impl Dialog for NumberSelect<'_> {
    /// Value returned from [`Dialog::run_over`]; the selected index. 
    type Out = u8;

    /// Instantiate a struct describing how the [`Dialog`] machinery should render the dialog. Analogous to
    /// [`State::draw`]. 
    fn format(&self) -> DrawInfo {
        // renders each label to a Line
        let labels = self.labels
            .iter()
            .enumerate()
            .map(|(i, label)| (i + 1, label))
            .map(|(i, label)| format!("({i}) {label}"))
            .map(Line::from);
        // append the message before the labels
        let body = std::iter::once(Line::from(self.msg))
            .chain(std::iter::once(Line::default()))
            .chain(labels)
            .collect();
        // hint shown at the bottom of the dialog, used for keybindings
        let hint = format!("Press a number 1-{}...", self.labels.len())
            .into();
        // return info necessary to render the dialog
        DrawInfo {
            title: "Select an option".into(), 
            color: Color::Green, 
            body, 
            hint, 
            ..Default::default()
        }
    }

    /// Conceptually the same as [`State::input`]. 
    fn input(self, key: KeyEvent) -> Signal<Self> {
        // if a number is entered...
        if let KeyCode::Char(char@'1'..='9') = key.code {
            let number = (char as u8) - b'0';

            // return if that number is a valid index to the labels
            if number <= self.labels.len() as u8 {
                return Signal::Return(number)
            }
        }
        // otherwise, continue running
        Signal::Continue(self)
    }
}

fn main() -> io::Result<()> {
    let mut ctx = Context::new()?;

    // instantiate and run the dialog
    let dialog = NumberSelect {
        msg: "What is the worst thing you can do?", 
        labels: &[
            "Commit a war crime", 
            "Murder a stranger", 
            "Make a mistake", 
            "Lose a friend", 
            "Oversleep for the 3rd time this week", 
        ], 
    };
    let number = dialog.run_over(&(), &mut ctx);

    // output a response based on the entered number
    let result = match number {
        1 => "Depends on the war crime.", 
        2 => "They probably deserved it.", 
        3 => "As long as it doesn't cost the company.", 
        4 => "As long as you grieve on your own time.", 
        5 => "Correct. There is no excuse.", 
        _ => unreachable!(), 
    };
    dialog::info(result, &(), &mut ctx);

    Ok(())
}
