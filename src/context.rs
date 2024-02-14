use std::{
    cell::RefCell, 
    io, 
    ops::{Deref, DerefMut}, 
    rc::Rc, 
};
use crate::State;
use self::managed::Wrapper;

pub type Backend = ratatui::backend::CrosstermBackend<io::Stdout>;
pub type Terminal = ratatui::Terminal<Backend>;

/// Stores the [`Terminal`] and represents the terminal environment as a whole. 
#[derive(Debug)]
enum Environment {
    /// RAII wrapper over [`Terminal`] to initialize/reset the terminal environment. 
    Managed(Wrapper), 
    /// Just stores the [`Terminal`]. 
    Unmanaged(Terminal), 
}

/// Manages the terminal environment. 
/// 
/// Serves as a wrapper around [Ratatui's terminal](ratatui::Terminal) with added RAII to automatically
/// initialise and reset the terminal environment. The initialisation of the terminal environment consists
/// of: 
/// - Installing a panic handler to make sure the terminal environment is reset before the program exits and
/// a panic message is printed. 
/// - Enabling raw mode. 
/// - Hiding the cursor. 
/// - Entering an alternate terminal buffer. 
/// 
/// 
/// # Basic usage
/// 
/// Construct the context using [`Context::new`] and give a mutable reference to it when running states with
/// [`State::run`]. 
/// 
/// 
/// # Application-defined global
/// 
/// In addition to managing the terminal environment, the context also provides the utility of a global 
/// value, which can be whatever makes sense in the application. Suitable examples include configuration 
/// data or user information. The global will then be available via the [`global`](Context::global) field of
/// the context for all states ran with it. 
/// 
/// Note that this is purely opt-in; for applications where no global data is necessary, `()` may be used, 
/// which is the default. 
/// 
/// To use a context global, construct the context using [`Context::with_global`] and set the
/// [`Global`](crate::State::Global) type of all states ran with the context equal to the type of the global. 
/// 
/// 
/// # Chaining with new globals
/// 
/// Though globals should generally persist across an entire application, there is support for creating a
/// "new" context with a new global value, while reusing the same internal [`Terminal`] handle. This is
/// achieved through _chaining_ using [`Context::chain_with_global`] or [`Context::chain_without_global`]. 
/// 
/// Chaining may be useful where there are distinct clusters of states in an application, with each cluster
/// having its own associated global. 
/// 
/// ⚠️ Creating several context instances using [`Context::new`] or [`Context::with_global`] should generally
/// be avoided. 
/// 
/// 
/// # Custom panic handler
/// 
/// The installed panic handler will delegate to the previous one after resetting the terminal. If a custom
/// panic handler is used in the application, it should be installed *before* creating the context to ensure
/// compatability. 
/// 
/// 
/// # Unmanaged terminal environment
/// 
/// The automatic initialisation and resetting of the terminal environment can be opted out from by using
/// [`Context::new_unmanaged`] or [`Context::with_global_unmanaged`] to construct the context. Note that in
/// these cases, the [`Terminal`] instance must be constructed manually by application code. See
/// [Ratatui's documentation](ratatui) on how to do this. 
/// 
/// 
/// # Examples
/// 
/// Creating a context without global data and using it to run a [`State`]: 
/// ```no_run
/// # use tundra::prelude::*;
/// let mut ctx = Context::new()?;
/// # let some_state = ();
/// // let some_state: impl State<Global = ()>
/// some_state.run(&mut ctx)
/// # ; Ok::<(), std::io::Error>(())
/// ```
/// 
/// Creating a context with global user data: 
/// ```no_run
/// # use tundra::prelude::*;
/// struct User {
///     name: String, 
///     id: u32, 
/// }
/// 
/// let user = User {
///     name: "Don Hertzfeldt".into(), 
///     id: 2012, 
/// };
/// let mut ctx = Context::with_global(user)?;
/// 
/// // the global can then be retrieved as: 
/// let user: &User = &ctx.global;
/// 
/// // and the context can be used to run states that have `State::Global = User`. 
/// # struct SomeState;
/// # impl SomeState{ fn run<T>(self, _: T) {} }
/// # let some_state = SomeState;
/// // let some_state: impl State<Global = User>
/// some_state.run(&mut ctx)
/// # ; Ok::<(), std::io::Error>(())
/// ```
/// 
/// "Removing" the global from an existing context: 
/// ```no_run
/// # use std::path::PathBuf;
/// # use tundra::prelude::*;
/// let cache_dir = "~/.cache/svalbard/".into();
/// 
/// let mut old: Context<PathBuf> = Context::with_global(cache_dir)?;
/// let new: Context = old.chain_without_global();
/// 
/// // old context is still available!
/// # struct SomeState;
/// # impl SomeState{ fn run<T>(self, _: T) {} }
/// # let some_state = SomeState;
/// // let some_state: impl State<Global = PathBuf>
/// some_state.run(&mut old)
/// # ; Ok::<(), std::io::Error>(())
/// ```
/// 
/// Constructing a context without automatic management of the terminal environment (this requires adding
/// [`crossterm`] to the application's dependencies): 
/// ```no_run
/// use std::io;
/// use crossterm::{
///     terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}, 
///     cursor::{Hide, Show}, 
/// };
/// use ratatui::prelude::*;
/// use tundra::{Terminal, Backend};
/// # use tundra::prelude::*;
/// 
/// // construct and initialize terminal
/// let backend = Backend::new(io::stdout());
/// let terminal = Terminal::new(backend)?;
/// terminal::enable_raw_mode()?;
/// crossterm::execute!(io::stdout(), Hide, EnterAlternateScreen)?;
/// 
/// // construct context and run some state with it
/// let mut ctx = Context::new_unmanaged(terminal);
/// # let some_state = ();
/// // let some_state: impl State<Global = ()>
/// some_state.run(&mut ctx);
/// 
/// // reset terminal
/// terminal::disable_raw_mode();
/// crossterm::execute!(io::stdout(), Show, LeaveAlternateScreen);
/// # Ok::<(), std::io::Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct Context<G = ()> {
    /// Application-defined global value. See the [context documentation](Context#application-defined-global)
    /// for more information. 
    pub global: G, 
    /// A reference to the RAII wrapper over the terminal environment. This is reference-counted to allow for
    /// [chaining](Context#chaining-with-new-globals). 
    environment: Rc<RefCell<Environment>>, 
}

impl<G> Context<G> {
    /// Creates a new context with given global value. If no global is needed, prefer [`Context::new`]. 
    pub fn with_global(global: G) -> io::Result<Self> {
        Wrapper::new()
            .map(Environment::Managed)
            .map(|env| Self::with_global_impl(global, env))
    }

    /// Creates a new context with given global value without a managed terminal environment. See the
    /// [type-level](Context#unmanaged-terminal-environment) documentation for more information. If no global
    /// is needed, prefer [`Context::new`]. 
    pub fn with_global_unmanaged(global: G, terminal: Terminal) -> Self {
        Self::with_global_impl(global, Environment::Unmanaged(terminal))
    }

    fn with_global_impl(global: G, environment: Environment) -> Self {
        Context {
            global, 
            environment: Rc::new(RefCell::new(environment)), 
        }
    }

    /// Applies an arbitrary function to the internal [`Terminal`] handle. 
    /// 
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// use ratatui::{Terminal, layout::Rect};
    /// 
    /// # use tundra::Context;
    /// # let ctx = Context::new().unwrap();
    /// // let ctx: &Context<_>
    /// let size: Rect = ctx.apply(Terminal::size)?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn apply<T>(&self, f: impl FnOnce(&Terminal) -> T) -> T {
        let env = self.environment.borrow();
        let term = match env.deref() {
            Environment::Unmanaged(term) => term, 
            Environment::Managed(wrapper) => &wrapper.0, 
        };
        f(term)
    }

    /// Applies an arbitrary function to the internal [`Terminal`] handle. 
    /// 
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// use ratatui::Terminal;
    /// # use tundra::Context;
    /// 
    /// # let mut ctx = Context::new().unwrap();
    /// // let ctx: &mut Context<_>
    /// ctx.apply_mut(Terminal::clear)?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn apply_mut<T>(&mut self, f: impl FnOnce(&mut Terminal) -> T) -> T {
        let mut env = self.environment.borrow_mut();
        let term = match env.deref_mut() {
            Environment::Unmanaged(term) => term, 
            Environment::Managed(wrapper) => &mut wrapper.0, 
        };
        f(term)
    }

    /// Draws a [`State`] using the internal [`Terminal`] handle. 
    pub fn draw_state(&mut self, state: &impl State) -> io::Result<()> {
        self.apply_mut(|terminal| terminal
            .draw(|frame| state.draw(frame))
            .map(|_| ())
        )
    }

    /// Creates a new context with a new global from an existing context, reusing the internal [`Terminal`]
    /// handle. This can be used "replace" the global value. See the
    /// [context documentation](Context#chaining-with-new-globals) for more information. 
    pub fn chain_with_global<F>(&self, global: F) -> Context<F> {
        Context {
            global, 
            environment: Rc::clone(&self.environment), 
        }
    }

    /// Creates a new context without a global from an existing context, reusing the internal [`Terminal`]
    /// handle. This can be used "remove" the global value. See the
    /// [context documentation](Context#chaining-with-new-globals) for more information. 
    pub fn chain_without_global(&self) -> Context {
        self.chain_with_global(())
    }
}

impl Context<()> {
    /// Creates a new context without a global value. If a global is needed, prefer [`Context::with_global`]. 
    pub fn new() -> io::Result<Context> {
        Context::with_global(())
    }

    /// Creates a new context without a global value and without a managed terminal environment. See the
    /// [type-level](Context#unmanaged-terminal-environment) documentation for more information. If a global
    /// is needed, prefer [`Context::with_global`]. 
    pub fn new_unmanaged(terminal: Terminal) -> Context {
        Context::with_global_unmanaged((), terminal)
    }
}

mod managed {
    use std::{
        io, 
        panic, 
        sync::atomic::{AtomicBool, Ordering}, 
    };
    use crossterm::{
        terminal::{self, EnterAlternateScreen, LeaveAlternateScreen}, 
        cursor::{Hide, Show}, 
    };
    use super::{Terminal, Backend};

    /// RAII wrapper over [`Terminal`] to initialize/reset the terminal environment. 
    #[derive(Debug)]
    pub struct Wrapper(pub Terminal);

    impl Wrapper {
        pub fn new() -> io::Result<Wrapper> {
            init().map(Wrapper)
        }
    }

    impl Drop for Wrapper {
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
        // this guard ensures that the panic handler is not installed multiple times, even if the user (for
        // whatever reason) creates multiple context instances with `Context::new` or `Context::with_global`
        static PANIC_HOOKED: AtomicBool = AtomicBool::new(false);

        let backend = Backend::new(io::stdout());
        let term = Terminal::new(backend)?;
    
        if !PANIC_HOOKED.swap(true, Ordering::Relaxed) {
            let prev_hook = panic::take_hook();
            panic::set_hook(Box::new(move |info| {
                reset();
                prev_hook(info);
            }));
        }
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stdout(), Hide, EnterAlternateScreen)?;
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
