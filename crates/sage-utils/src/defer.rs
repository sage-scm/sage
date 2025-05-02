//! Defer functionality inspired by Go and Zig.
//!
//! This module provides macros for deferring execution of code:
//!
//! - `defer!` - Executes when the scope exits normally (not on panic or `Err` return)
//! - `defer_err!` - Executes only when returning an `Err` (not on normal exit or panic)
//!
//! # Examples
//!
//! ```
//! use sage_utils::{defer, defer_err};
//!
//! fn example() -> Result<(), &'static str> {
//!     // This will be executed when the function exits normally with Ok
//!     let _defer_ok = defer! {
//!         println!("Success cleanup - only runs on Ok return");
//!     };
//!
//!     // This will be executed only if the function returns an Err
//!     let _defer_err = defer_err! {
//!         println!("Error cleanup - only runs on Err return");
//!     };
//!
//!     // Simulate success or error
//!     if should_succeed() {
//!         println!("Operation succeeded");
//!         Ok(())
//!     } else {
//!         println!("Operation failed");
//!         Err("something went wrong")
//!     }
//! }
//!
//! fn should_succeed() -> bool {
//!     true
//! }
//! ```

/// Execution mode for deferred code
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeferMode {
    /// Execute on normal exit (Ok result)
    OnSuccess,
    /// Execute only on error (Err result)
    OnError,
}

/// A struct that executes a closure when dropped, based on the specified mode.
pub struct Defer<F: FnOnce()> {
    f: Option<F>,
    // Track if we're in a panic state
    active: bool,
    // Track if we're in an error state
    error_state: bool,
    // When to execute the deferred code
    mode: DeferMode,
}

impl<F: FnOnce()> Defer<F> {
    /// Creates a new `Defer` with the given closure that runs on success.
    #[inline]
    pub fn new(f: F) -> Self {
        Defer {
            f: Some(f),
            active: true,
            error_state: false,
            mode: DeferMode::OnSuccess,
        }
    }

    /// Creates a new `Defer` with the given closure that runs on error.
    #[inline]
    pub fn new_on_error(f: F) -> Self {
        Defer {
            f: Some(f),
            active: true,
            error_state: false,
            mode: DeferMode::OnError,
        }
    }

    /// Deactivates the defer, preventing it from running.
    /// This should be called during a panic.
    #[inline]
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Marks that an error has occurred.
    /// This should be called when returning an Err.
    #[inline]
    pub fn mark_error(&mut self) {
        self.error_state = true;
    }

    /// Checks if the defer is active.
    #[inline]
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Gets the execution mode of this defer.
    #[inline]
    pub fn mode(&self) -> DeferMode {
        self.mode
    }
}

impl<F: FnOnce()> Drop for Defer<F> {
    #[inline]
    fn drop(&mut self) {
        // Only execute if active (not in a panic state)
        if self.active {
            let should_execute = match self.mode {
                // Execute on success only if we're not in an error state
                DeferMode::OnSuccess => !self.error_state,
                // Execute on error only if we are in an error state
                DeferMode::OnError => self.error_state,
            };

            if should_execute {
                // Take the function out to avoid borrowing issues
                if let Some(f) = self.f.take() {
                    f();
                }
            }
        }
    }
}

/// Defers execution of a block of code until the current scope exits normally.
///
/// This macro executes the given code block when the scope exits normally.
/// It will NOT execute if:
/// - A panic occurs
/// - A Result::Err is returned
/// - The scope exits abnormally in any other way
///
/// It will execute on normal scope exit or early return with Ok values.
///
/// # Examples
///
/// ```
/// use sage_utils::defer;
///
/// fn example() -> Result<(), std::io::Error> {
///     let file = std::fs::File::open("example.txt")?;
///
///     // This will be executed when the function exits normally with Ok
///     let mut defer_guard = defer! {
///         println!("Closing file...");
///         // Any cleanup code here
///     };
///
///     // Do work with the file...
///     println!("Working with file...");
///
///     // If an error occurs, mark the error state
///     if let Err(e) = some_operation() {
///         defer_guard.mark_error(); // Prevent defer from running on error
///         return Err(e);
///     }
///
///     // The deferred code will run when the function exits normally
///     Ok(())
/// }
///
/// fn some_operation() -> Result<(), std::io::Error> {
///     // Some operation that might fail
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! defer {
    ($($body:tt)*) => {
        $crate::defer::Defer::new(|| {
            $($body)*
        })
    };
}

/// Defers execution of a block of code to run only when returning an Err.
///
/// This macro is inspired by Zig's `errdefer` statement. It executes the given
/// code block only when the scope exits with an Err result. It will NOT execute if:
/// - A panic occurs
/// - The scope exits normally with an Ok result
///
/// # Examples
///
/// ```
/// use sage_utils::{defer, defer_err};
///
/// fn example() -> Result<(), std::io::Error> {
///     // This will execute only if we return an Err
///     let mut error_handler = defer_err! {
///         println!("An error occurred, cleaning up...");
///         // Error-specific cleanup code here
///     };
///
///     // Do some work that might fail
///     println!("Working...");
///
///     // If an error occurs
///     if let Err(e) = some_operation() {
///         // Mark the error state so the defer_err will run
///         error_handler.mark_error();
///         return Err(e);
///     }
///
///     // The error handler won't run on normal exit
///     Ok(())
/// }
///
/// fn some_operation() -> Result<(), std::io::Error> {
///     // Some operation that might fail
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! defer_err {
    ($($body:tt)*) => {
        $crate::defer::Defer::new_on_error(|| {
            $($body)*
        })
    };
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;
    use super::DeferMode;

    #[test]
    fn test_defer_executes_on_scope_exit() {
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();

        {
            let _defer = defer! {
                *counter_clone.borrow_mut() += 1;
            };

            assert_eq!(*counter.borrow(), 0);
        }

        assert_eq!(*counter.borrow(), 1);
    }

    #[test]
    fn test_defer_executes_on_early_return() {
        let counter = Rc::new(RefCell::new(0));

        fn inner(counter: Rc<RefCell<u32>>, should_return_early: bool) -> u32 {
            let counter_clone = counter.clone();

            let _defer = defer! {
                *counter_clone.borrow_mut() += 1;
            };

            if should_return_early {
                return 0;
            }

            42
        }

        assert_eq!(inner(counter.clone(), true), 0);
        assert_eq!(*counter.borrow(), 1);

        assert_eq!(inner(counter.clone(), false), 42);
        assert_eq!(*counter.borrow(), 2);
    }

    #[test]
    fn test_multiple_defers() {
        let execution_order = Rc::new(RefCell::new(Vec::new()));

        {
            let exec_clone = execution_order.clone();
            let _defer1 = defer! {
                exec_clone.borrow_mut().push("first");
            };

            let exec_clone = execution_order.clone();
            let _defer2 = defer! {
                exec_clone.borrow_mut().push("second");
            };
        }

        // Defers execute in LIFO order (last in, first out)
        assert_eq!(*execution_order.borrow(), vec!["second", "first"]);
    }

    #[test]
    fn test_defer_does_not_execute_on_error() {
        let counter = Rc::new(RefCell::new(0));

        fn inner(counter: Rc<RefCell<u32>>, should_error: bool) -> Result<u32, &'static str> {
            let counter_clone = counter.clone();

            let mut defer_guard = defer! {
                *counter_clone.borrow_mut() += 1;
            };

            if should_error {
                defer_guard.mark_error(); // Mark error state
                return Err("An error occurred");
            }

            Ok(42)
        }

        // Should not increment counter on error
        let result = inner(counter.clone(), true);
        assert!(result.is_err());
        assert_eq!(*counter.borrow(), 0);

        // Should increment counter on success
        let result = inner(counter.clone(), false);
        assert!(result.is_ok());
        assert_eq!(*counter.borrow(), 1);
    }

    #[test]
    fn test_defer_can_be_manually_deactivated() {
        let counter = Rc::new(RefCell::new(0));
        let counter_clone = counter.clone();

        {
            let mut defer_guard = defer! {
                *counter_clone.borrow_mut() += 1;
            };

            // Manually deactivate
            defer_guard.deactivate();
            assert!(!defer_guard.is_active());
        }

        // Counter should not have been incremented
        assert_eq!(*counter.borrow(), 0);
    }

    #[test]
    fn test_defer_err_executes_only_on_error() {
        let counter = Rc::new(RefCell::new(0));

        fn inner(counter: Rc<RefCell<u32>>, should_error: bool) -> Result<u32, &'static str> {
            let counter_clone = counter.clone();

            let mut defer_err_guard = defer_err! {
                *counter_clone.borrow_mut() += 1;
            };

            assert_eq!(defer_err_guard.mode(), DeferMode::OnError);

            if should_error {
                defer_err_guard.mark_error(); // Mark error state
                return Err("An error occurred");
            }

            Ok(42)
        }

        // Should increment counter on error
        let result = inner(counter.clone(), true);
        assert!(result.is_err());
        assert_eq!(*counter.borrow(), 1);

        // Should NOT increment counter on success
        let result = inner(counter.clone(), false);
        assert!(result.is_ok());
        assert_eq!(*counter.borrow(), 1); // Counter unchanged
    }

    #[test]
    fn test_defer_and_defer_err_together() {
        let success_counter = Rc::new(RefCell::new(0));
        let error_counter = Rc::new(RefCell::new(0));

        fn inner(
            success_counter: Rc<RefCell<u32>>,
            error_counter: Rc<RefCell<u32>>,
            should_error: bool
        ) -> Result<u32, &'static str> {
            let success_clone = success_counter.clone();
            let error_clone = error_counter.clone();

            // This should run only on success
            let mut defer_guard = defer! {
                *success_clone.borrow_mut() += 1;
            };

            // This should run only on error
            let mut defer_err_guard = defer_err! {
                *error_clone.borrow_mut() += 1;
            };

            if should_error {
                // Mark both guards with the error state
                defer_guard.mark_error();
                defer_err_guard.mark_error();
                return Err("An error occurred");
            }

            Ok(42)
        }

        // Error case
        let result = inner(success_counter.clone(), error_counter.clone(), true);
        assert!(result.is_err());
        assert_eq!(*success_counter.borrow(), 0); // Success counter unchanged
        assert_eq!(*error_counter.borrow(), 1);   // Error counter incremented

        // Success case
        let result = inner(success_counter.clone(), error_counter.clone(), false);
        assert!(result.is_ok());
        assert_eq!(*success_counter.borrow(), 1); // Success counter incremented
        assert_eq!(*error_counter.borrow(), 1);   // Error counter unchanged
    }
}
