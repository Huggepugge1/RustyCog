# RustyCog
Effortless Task Management with Dynamic Execution

## Overview
RustyCog is a lightweight task manager for Rust. It allows
the scheduling of cogs (tasks), engageng (executing) the cogs and finally
retrieve the results. The results can be retrieved at any time, either
blocking or simply check back in a while to see if the cog has finishedc or not.

## Installation
TBD

## Example
```rs
use rustycog::Machine;

fn main() {
    // NOTE: In RustyCog 0.2.0 Machine::new() will deprecated.
    // Use Machine::powered() or Machine::cold() instead
    // But as of RustyCog 0.1.0, Machine::new() is the way to go
    let mut machine = Machine::<i32>::new();
    let id0 = machine.insert_cog(|| 42);
    let id1 = machine.insert_cog(|| {
        println!("Running a task...");
        99
    });

    // NOTE: In RustyCog 0.2.0 Machine::run() will be deprecated.
    // Use Machine::start() instead
    // But as of RustyCog 0.1.0, Machine::run() is the way to go
    machine.run();

    println!("Result of cog 0: {:?}", machine.wait_for_result(id0)); // Ok(42)
    println!("Result of cog 1: {:?}", machine.wait_for_result(id1)); // Ok(99)
}

```

## Error Handling
See the documentation for more information about the different errors.
### Example
```rs
use rustycog::{Machine, error::CogError};

fn main() {
    let mut machine = Machine::<i32>::new();
    let id = machine.insert_cog(|| {
        panic!("Oh no!");
    });

    machine.run();
    match machine.wait_for_result(id) {
        Ok(result) => println!("Result: {}", result),
        Err(CogError::Panicked) => println!("The cog panicked!"),
        _ => println!("Unexpected error"),
    }
}
```

## Future Plans
- Dynamic Boiler Management: Automatically adjust the amount of background threads
  depending on the current workload.
- Prioritization: Allowing certain cogs to be prioritized. For example a cog
  currently being waited for will get priority for decreased response times
- Automatic cleanup: When a cogs result has been retrieved, it automatically cleans itself.
- Immediate Cog Engagement: Remove the hassle of starting the boilers yourself.
  Cogs engage as soon as they can after being inserted into the machine.
- Cancel cogs at any time

## Why Choose RustyCog?
RustyCog provides a unique approach to task management in Rust,
allowing you to manage tasks like futures without actually being asynchronous.
Synchronous + asyncrhonous in one!

## Contrbuting
Contributions are welcome! Please open an issue to discuss your ideas, or fork the repo and open a pull request.

## Licence
This project is licensed under the MIT License. See [LICENSE] for details.
