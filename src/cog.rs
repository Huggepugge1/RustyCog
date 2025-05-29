use std::cmp::Ordering;

use crate::error::DefaultCogError;

/// A unique identifier for a submitted cog (task).
///
/// Typically returned by `Machine::insert_cog`, and used to retrieve results.
pub type CogId = usize;

/// Trait representing a task ("Cog") that can be scheduled and executed by the `Machine`.
///
/// # Associated Types
/// - `T`: The output type produced when the cog is executed.
///
/// # Required Methods
/// - `run`: The main work function. Executes the task and returns its result.
/// - `priority`: Defines how this cog compares to another for scheduling order.
///
/// # Scheduling and Priority
/// Implement `priority` to control task ordering in the scheduler.  
/// It should return an `Ordering` that determines which cog runs first (e.g., by deadline, importance).
///
/// # Example
/// ```
/// use std::cmp::Ordering;
///
/// use rustycog::{cog::CogTrait};
///
/// #[derive(PartialEq, Eq, PartialOrd, Ord)]
/// enum Importance {
///     Low,
///     High,
///     Critical,
/// }
///
/// struct MyCog {
///     importance: Importance,
///     func: Option<Box<dyn FnOnce() -> usize + Send>>
/// }
///
/// impl CogTrait for MyCog {
///     type T = usize;
///
///     fn run(&mut self) -> Self::T {
///         let task = std::mem::take(&mut self.func).unwrap();
///         task()
///     }
///
///     fn priority(&self, other: &Self) -> Ordering {
///         self.importance.cmp(&other.importance)
///     }
/// }
/// ```
pub trait CogTrait: Send + 'static {
    /// The result type returned by the cog's execution
    type T: Send;

    /// Runs the task, returns the result
    fn run(&mut self) -> Self::T;

    /// Compares this cog's priority to another's.
    ///
    /// Return Ordering::Greater if `self` should run before `other`
    fn priority(&self, _other: &Self) -> Ordering {
        Ordering::Equal
    }
}

impl<T, F> From<F> for Cog<T>
where
    T: Send,
    F: FnOnce() -> T + Send + 'static,
{
    fn from(func: F) -> Cog<T> {
        Cog::new(func)
    }
}

/// A basic task ("Cog") that wraps a single `FnOnce()` closure to produce a result.
///
/// # Type Parameters
/// - `T`: The output type of the closure.
///
/// # Example
/// ```
/// use rustycog::{Machine, cog::Cog};
///
/// let mut machine = Machine::powered(8);
/// let mut cog = Cog::new(|| 42);
/// let cog_id = machine.insert_cog(cog);
/// assert_eq!(machine.wait_for_result(cog_id).unwrap().unwrap(), 42);
/// ```
pub struct Cog<T>
where
    T: Send,
{
    func: Option<Box<dyn FnOnce() -> T + Send + 'static>>,
}

impl<T> CogTrait for Cog<T>
where
    T: Send + 'static,
{
    type T = Result<T, DefaultCogError>;

    /// Executes the task if it hasn’t already run.
    ///
    /// # Errors
    /// Returns `DefaultCogError::AlreadyRan` if called more than once.
    fn run(&mut self) -> Self::T {
        let func = std::mem::take(&mut self.func).ok_or(DefaultCogError::AlreadyRan)?;
        Ok(func())
    }
}

impl<T> Cog<T>
where
    T: Send,
{
    /// Creates a new cog with the given closure.
    ///
    /// # Arguments
    /// - `func`: A closure representing the work to be done.
    ///
    /// # Returns
    /// A `Cog` instance ready for insertion into a [`Machine`](crate::Machine).
    ///
    /// # Example
    /// ```
    /// use rustycog::cog::PrioCog;
    ///
    /// let cog = PrioCog::new(|| 42, 10);
    /// ```
    pub fn new<F>(func: F) -> Self
    where
        F: FnOnce() -> T + Send + 'static,
    {
        Self {
            func: Some(Box::new(func)),
        }
    }
}

/// A prioritized task ("Cog") that wraps a closure with an associated priority.
///
/// Higher-priority cogs are scheduled to run before lower-priority ones.
///
/// # Type Parameters
/// - `T`: The output type of the closure.
///
/// # Example
/// ```
/// use rustycog::{Machine, cog::PrioCog};
///
/// let mut machine = Machine::cold(1);
///
/// let cog1 = PrioCog::new(|| print!("hello"), 1);
/// let cog2 = PrioCog::new(|| print!(" world"), 0);
/// let _ = machine.insert_cog(cog1);
/// let _ = machine.insert_cog(cog2);
///
/// machine.power();
///
/// machine.wait_until_done();
/// // Prints: hello world
/// ```
pub struct PrioCog<T>
where
    T: Send,
{
    prio: usize,
    func: Option<Box<dyn FnOnce() -> T + Send + 'static>>,
}

impl<T> CogTrait for PrioCog<T>
where
    T: Send + 'static,
{
    type T = Result<T, DefaultCogError>;

    /// Executes the task if it hasn’t already run.
    ///
    /// # Errors
    /// Returns `DefaultCogError::AlreadyRan` if called more than once.
    fn run(&mut self) -> Self::T {
        let func = std::mem::take(&mut self.func).ok_or(DefaultCogError::AlreadyRan)?;
        Ok(func())
    }

    /// Compares this cog's priority with another's to determine scheduling order.
    ///
    /// By default, higher values indicate higher priority (run earlier).
    /// Override this logic by implementing a custom `CogTrait` if needed.
    fn priority(&self, other: &Self) -> Ordering {
        self.prio.cmp(&other.prio)
    }
}

impl<T> PrioCog<T>
where
    T: Send,
{
    /// Creates a new prioritized cog with the given closure and priority.
    ///
    /// # Arguments
    /// - `func`: A closure representing the work to be done.
    /// - `prio`: A numeric priority; higher values indicate higher priority.
    ///
    /// # Returns
    /// A `PrioCog` instance ready for insertion into a [`Machine`](crate::Machine).
    ///
    /// # Example
    /// ```
    /// use rustycog::cog::PrioCog;
    ///
    /// let cog = PrioCog::new(|| 42, 10);
    /// ```
    pub fn new<F>(func: F, prio: usize) -> Self
    where
        F: FnOnce() -> T + Send + 'static,
    {
        Self {
            prio,
            func: Some(Box::new(func)),
        }
    }
}
