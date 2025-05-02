use sage_utils::{defer, defer_err};

fn main() {
    println!("Starting defer example");

    // This will be executed when the function exits normally
    defer! {
        println!("Deferred cleanup at the end of main");
    };

    // Demonstrate nested defers
    {
        println!("Entering nested scope");

        defer! {
            println!("Deferred cleanup for nested scope");
        };

        println!("Exiting nested scope");
    }

    // Demonstrate early return with defer
    let result = example_with_early_return(true);
    println!("Result from early return: {}", result);

    let result = example_with_early_return(false);
    println!("Result from normal execution: {}", result);

    // Demonstrate error handling with defer and defer_err
    println!("\nTesting defer and defer_err with error case:");
    match example_with_both_defers(true) {
        Ok(val) => println!("Success: {}", val),
        Err(e) => println!("Error occurred: {}", e),
    }

    println!("\nTesting defer and defer_err with success case:");
    match example_with_both_defers(false) {
        Ok(val) => println!("Success: {}", val),
        Err(e) => println!("Error occurred: {}", e),
    }

    println!("\nExiting main function");
}

fn example_with_early_return(should_return_early: bool) -> i32 {
    let _defer = defer! {
        println!("Cleaning up in example_with_early_return");
    };

    if should_return_early {
        println!("Returning early");
        return -1;
    }

    println!("Normal execution path");
    42
}

fn example_with_both_defers(should_error: bool) -> Result<i32, &'static str> {
    // This will run only on successful completion
    let mut defer_success = defer! {
        println!("SUCCESS CLEANUP: This runs only on normal exit");
    };

    // This will run only on error
    let mut defer_error = defer_err! {
        println!("ERROR CLEANUP: This runs only when returning Err");
    };

    println!("Working in example_with_both_defers...");

    if should_error {
        println!("Error detected, marking error state");
        // Mark both defers with the error state
        defer_success.mark_error(); // Prevents success defer from running
        defer_error.mark_error();   // Enables error defer to run
        return Err("Something went wrong");
    }

    println!("Successful execution");
    Ok(100)
}
