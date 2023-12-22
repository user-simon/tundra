use std::{io, rc::Rc, cell::RefCell};
use crate::State;

pub type Backend = ratatui::backend::CrosstermBackend<io::Stdout>;
pub type Terminal = ratatui::Terminal<Backend>;

#[cfg(feature = "managed_env")]
type Environment = managed::Environment;

#[cfg(not(feature = "managed_env"))]
type Environment = Terminal;

/// Manages the terminal environment. 
/// 
/// Serves as a wrapper around [Ratatui's terminal](ratatui::Terminal) with added RAII to automatically
/// initialize and reset the terminal environment. This automatic management can be disabled by disabling the
/// `managed_env` feature, in which case the application code has to create the terminal instance manually.
/// See the [Ratatui documentation](ratatui) for how to do this. 
/// 
/// The initialization of the terminal environment consists of: 
/// - Installing a panic handler to make sure the terminal environment is reset before the program exits.
/// - Enabling raw mode. 
/// - Hiding the cursor. 
/// - Entering an alternate terminal buffer. 
/// 
/// The installed panic handler will delegate to the previous one after resetting the terminal. If a custom
/// panic handler is used in the application, it should be installed *before* creating the context to ensure
/// compatability. 
/// 
/// In addition to managing the terminal environment, the context also provides the utility of a global 
/// value, which can be whatever makes sense in the application. Suitable examples include configuration 
/// data or user information. This will then available via the [`global`](Context::global) field of the 
/// context for all states ran with it. Note that this is purely opt-in; for applications where no global 
/// data is necessary, `()` may be used, which is the default. 
/// 
/// # Examples
/// To create a context without global data: 
/// ```
/// # use tundra::Context;
/// let context = Context::new().unwrap();
/// ```
/// 
/// To create a context with global data: 
/// ```
/// # use tundra::Context;
/// struct Config {
///     foo: bool, 
///     bar: String, 
/// }
/// 
/// let config = Config {
///     foo: true, 
///     bar: "Hello world".into(), 
/// };
/// let context = Context::with_global(config).unwrap();
/// ```
#[derive(Clone)]
pub struct Context<G = ()> {
    pub global: G, 
    environment: Rc<RefCell<Environment>>, 
}

impl<G> Context<G> {
    /// Creates a new context with given global value. 
    /// 
    /// If no global is needed, prefer [`Context::new`]. 
    #[cfg(feature = "managed_env")]
    pub fn with_global(global: G) -> io::Result<Self> {
        Environment::new().map(|env| Self::with_global_impl(global, env))
    }

    /// Creates a new context with given global value. 
    /// 
    /// If no global is needed, prefer [`Context::new`]. 
    #[cfg(not(feature = "managed_env"))]
    pub fn with_global(global: G, terminal: Terminal) -> Self {
        Self::with_global_impl(global, terminal)
    }

    fn with_global_impl(global: G, environment: Environment) -> Self {
        Context {
            global, 
            environment: Rc::new(RefCell::new(environment)), 
        }
    }

    /// Applies an arbitrary function to the internal [`Terminal`] handle. 
    /// 
    /// # Examples
    /// ```
    /// # use tundra::{Context, Terminal};
    /// let context = Context::new().unwrap();
    /// let size = context.apply(Terminal::size).unwrap();
    /// ```
    pub fn apply<T>(&self, f: impl FnOnce(&Terminal) -> T) -> T {
        let env = self.environment.borrow();

        #[cfg(feature = "managed_env")]
        let term = &env.0;

        #[cfg(not(feature = "managed_env"))]
        let term = &env;

        f(term)
    }

    /// Applies an arbitrary function to the internal [`Terminal`] handle. 
    /// 
    /// # Examples
    /// ```
    /// # use tundra::{Context, Terminal};
    /// let mut context = Context::new().unwrap();
    /// context.apply_mut(Terminal::clear).unwrap();
    /// ```
    pub fn apply_mut<T>(&mut self, f: impl FnOnce(&mut Terminal) -> T) -> T {
        let mut env = self.environment.borrow_mut();

        #[cfg(feature = "managed_env")]
        let term = &mut env.0;

        #[cfg(not(feature = "managed_env"))]
        let term = &mut env;

        f(term)
    }

    /// Draws a [state](crate::State) using the internal [`Terminal`] handle. 
    pub fn draw_state(&mut self, state: &impl State) -> io::Result<()> {
        self.apply_mut(|terminal| terminal
            .draw(|frame| state.draw(frame))
            .map(|_| ())
        )
    }

    pub fn chain_with_global<F>(&self, global: F) -> Context<F> {
        Context {
            global, 
            environment: Rc::clone(&self.environment), 
        }
    }

    pub fn chain_without_global(&self) -> Context {
        self.chain_with_global(())
    }
}

impl Context<()> {
    /// Creates a new context without a global value. 
    /// 
    /// If a global is needed, prefer [`Context::with_global`]. 
    #[cfg(feature = "managed_env")]
    pub fn new() -> io::Result<Context> {
        Context::with_global(())
    }

    /// Creates a new context without a global value. 
    /// 
    /// If a global is needed, prefer [`Context::with_global`]. 
    #[cfg(not(feature = "managed_env"))]
    pub fn new(terminal: Terminal) -> Context {
        Context::with_global((), terminal)
    }
}

#[cfg(feature = "managed_env")]
mod managed {
    use std::{
        io, 
        panic, 
        sync::atomic::{AtomicBool, Ordering}, 
    };
    use crossterm::{
        terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}, 
        cursor::{Hide, Show}
    };
    use super::{Terminal, Backend};

    /// RAII wrapper over [`Terminal`] to initialize/reset the terminal environment. 
    pub struct Environment(pub Terminal);

    impl Environment {
        pub fn new() -> io::Result<Environment> {
            init().map(Environment)
        }
    }

    impl Drop for Environment {
        fn drop(&mut self) {
            reset()
        }
    }

    /// Initializes the terminal environment. 
    /// 
    /// - Installs a panic handler to make sure the terminal environment is reset before the program exits. 
    /// - Enables raw mode. 
    /// - Hides the cursor. 
    /// - Enters an alternate terminal buffer. 
    fn init() -> io::Result<Terminal> {
        static PANIC_HOOKED: AtomicBool = AtomicBool::new(false);

        let backend = Backend::new(io::stdout());
        let mut term = Terminal::new(backend)?;
    
        if !PANIC_HOOKED.swap(true, Ordering::Relaxed) {
            let prev_hook = panic::take_hook();
            panic::set_hook(Box::new(move |info| {
                reset();
                prev_hook(info);
            }));
        }
        terminal::enable_raw_mode()?;
        crossterm::execute!(term.backend_mut(), Hide, EnterAlternateScreen)?;
        Ok(term)
    }
    
    /// Resets the terminal environment. 
    /// 
    /// - Disables raw mode. 
    /// - Shows the cursor. 
    /// - Leaves the alternate terminal buffer. 
    fn reset() {
        // if anything goes wrong, try to continue resetting the terminal; the program is probably closing
        // anyways
        let _ = terminal::disable_raw_mode();
        let _ = crossterm::execute!(io::stdout(), Show, LeaveAlternateScreen);
    }
}