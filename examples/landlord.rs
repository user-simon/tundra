use std::borrow::Borrow;
use std::cell::RefCell;
use std::io;
use ratatui::{layout::*, style::*, widgets::*};
use tundra::prelude::*;
use tundra::field::*;

/// Holds rent unit data. 
struct Unit {
    location: String, 
    rent: usize, 
    pets_allowed: bool, 
}

/// Interactive manager for adding and removing rent units to some database, here represented as a vector. 
#[derive(Default)]
struct Manager {
    /// Database of rent units being managed. 
    database: Vec<Unit>, 
    /// Table state holding what rent unit is currently selected. [`RefCell`] is used for interior mutability
    /// since a mutable reference is required by [`Frame::render_stateful_widget`] in [`State::draw`]. 
    table_state: RefCell<TableState>, 
}

impl Manager {
    /// Show a dialog with available commands using [`dialog::help`]. 
    fn show_help(&self, ctx: &mut Context) {
        const MSG: &str = "\
            (ctrl + a) Add new rent unit\n\
            (ctrl + r) Remove selected rent unit\n\
            (ctrl + e) Evict tenant at selected rent unit\n\
            (ctrl + h) Show this help message\n\
            (escape)   Quit the application\
        ";
        dialog::help(MSG, self, ctx)
    }

    /// Add a new rent unit to the database from values entered in a [`dialog::form!`]. 
    fn enter_new_unit(&mut self, ctx: &mut Context) {
        let values = dialog::form!{
            location: Textbox{ name: "Location" } if str::is_empty => "Must be non-empty", 
            rent: Slider<usize>{ name: "Monthly rent", range: 1..=5000, step: 50, value: 50, prefix: "$" }, 
            pets_allowed: Checkbox{ name: "Pets allowed" }, 
            [title]: "Register Rent Unit", 
            [context]: ctx, 
            [background]: self, 
        };
        // add the rent unit if the form wasn't cancelled
        if let Some(values) = values {
            let unit = Unit {
                location: values.location, 
                rent: values.rent, 
                pets_allowed: values.pets_allowed, 
            };
            self.database.push(unit);
            self.table_state.borrow_mut().select_last();
        }
    }

    /// Remove the currently selected rent unit if the user confirms with [`dialog::confirm`]. 
    fn remove_unit(&mut self, ctx: &mut Context) {
        let Some(selected) = self.table_state.borrow().selected() else {
            return
        };
        let location = &self.database[selected].location;
        let warning = format!("Are you sure you want to remove unit at {location}?");

        if dialog::confirm(warning, self, ctx) {
            self.database.remove(selected);
            self.table_state.borrow_mut().select_first();
        }
    }

    /// Tries evicting tentant at selected location. Unfortunately, this tends to fail, in which case an
    /// error message is shown with [`dialog::error`]. 
    fn evict_tentant(&self, ctx: &mut Context) {
        let Some(_) = self.table_state.borrow().selected() else {
            return
        };
        // landlords are evil
        dialog::error("Failed evicting tenant", self, ctx);
    }
}

impl State for Manager {
    type Result<T> = T;
    type Out = ();
    type Global = ();

    /// Delegate incoming key input events. 
    fn input(mut self, key: KeyEvent, ctx: &mut Context) -> Signal<Self> {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match (key.code, ctrl) {
            // move selected row up/down
            (KeyCode::Up, false) => self.table_state.borrow_mut().select_previous(), 
            (KeyCode::Down, false) => self.table_state.borrow_mut().select_next(), 
            // delegate commands
            (KeyCode::Char('a'), true) => self.enter_new_unit(ctx), 
            (KeyCode::Char('r'), true) => self.remove_unit(ctx), 
            (KeyCode::Char('e'), true) => self.evict_tentant(ctx), 
            (KeyCode::Char('h'), true) => self.show_help(ctx), 
            // exit the application
            (KeyCode::Esc, false) => return Signal::Return(()), 
            _ => (), 
        };
        Signal::Continue(self)
    }

    /// Draw the table using [`ratatui`]. 
    fn draw(&self, frame: &mut Frame) {
        // if the table is empty, allocate space for the header row and a help message. otherwise, allocate
        // space for just the table
        let [table_rect, help_rect] = {
            let constraints = match self.database.is_empty() {
                true => [Constraint::Length(2), Constraint::Min(1)], 
                false => [Constraint::Min(1), Constraint::Length(0)], 
            };
            Layout::default()
                .horizontal_margin(3)
                .vertical_margin(1)
                .constraints(constraints)
                .split(frame.area())
                .as_ref()
                .try_into()
                .expect("Two constraints are given")
        };

        // draw rent unit table
        {
            let header = Row::new(["LOCATION", "MONTHLY RENT", "PETS ALLOWED"])
                .bold()
                .bottom_margin(1);
            let rows = self.database
                .iter()
                .map(|unit| [
                    unit.location.clone(), 
                    format!("${}", unit.rent), 
                    match unit.pets_allowed {
                        true => "Yes".into(), 
                        false => "No".into(), 
                    }, 
                ])
                .map(Row::new);
            let widths = [Constraint::Ratio(1, 3); 3];
            let highlight_style = Style::new()
                .bold()
                .reversed();
            let widget = Table::new(rows, widths)
                .header(header)
                .highlight_style(highlight_style);
            let table_state = &mut self.table_state.borrow_mut();
            frame.render_stateful_widget(widget, table_rect, table_state);
        }

        // draw help message if the table is empty
        {
            const HELP: &str = "Nothing to show here. Press (ctrl + h) to see available commands...";
            let widget = Paragraph::new(HELP)
                .italic()
                .dim()
                .wrap(Wrap{ trim: true });
            frame.render_widget(widget, help_rect);
        }
        
    }
}

/// Constructs and runs the [`Manager`] state. 
fn manager(ctx: &mut Context) {
    Manager::default().run(ctx)
}

fn main() -> io::Result<()> {
    // initialise context
    let mut ctx = Context::new()?;
    // run the state
    manager(&mut ctx);
    // return once the state has finished running (when the user presses escape per State::input)
    Ok(())
}
