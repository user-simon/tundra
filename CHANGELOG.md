## v0.4.0: Form quality-of-life

#### Overview:

- Exposed API for showing custom message dialogs via `dialog::message`. 
- Added the ability to return values from form validation functions via `Result::Ok`. If the `Ok`-value is non-`()`, a tuple of `(entered_values, ok_value)` is returned. Otherwise, just `entered_values` is returned. 
- Added re-exports of Ratatui and Crossterm to ensure compatiblity. 
- Changed `field::Radio` rendering to be on a single line for readability.
- Changed `field::Toggle` handling of min/max values to be through form control statements, and added prefab
error conditions like `field::toggle::in_range(a, b)`. 
- Added tab/backtab form keybinds to move field focus while wrapping around the fields. 
- Added a Nix flake to provide a development environment. 
- Added a `|` delimiter between control statements to be more intuitive, which also allows e.g. treesitter to properly highlight `if` as a keyword. 


## v0.3.0: Forms 2.0

#### Overview: 

- Major rework of forms with per-field validation, messages, and more fluid metadata parsing. 
- Added a message to the `Select` dialog.
- Configurable affixes to sliders. This allows you to have e.g. a `$` prefix to the value of a slider. 
- Auto implementation of `State` for dialogs. This allows us to reuse `state::Signal` for dialogs, and 
`dialog::Signal` is therefore removed. 
- Bumped Ratatui version to 0.27. 

#### Forms 2.0!

You can now specify per-field validation of forms using control statements. Each control statement declares 
an error condition as a boolean function over the value of the field, and an error message. If the error 
condition triggers, the name of the offending field turns red, and the erorr message is shown if the user 
attempts to submit the form. 

Form metadata can now also be given in any order. This allows us to add more optional metadata without 
requiring that all previous optional metadata is given (e.g. how default parameters to functions work in many 
languages), and should be more intuitive to use. Using this, we've added a message that can be shown above 
the fields when the form is displayed, which may be useful for explaining the context and effects of a form 
to the user. 

See the [form macro documentation](https://docs.rs/tundra/latest/tundra/macro.form.html) for more
information. 


## v0.2.0: Ratatui bump

#### Overview: 

- Bumped Ratatui version to 0.26. 
- [doc] Added notes on Ratatui version compatibility. 
