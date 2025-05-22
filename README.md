# RustyCog
Effortless Task Management with Dynamic Execution

## Overview
RustyCog is a lightweight task manager for Rust. It allows
the scheduling the cogs (Tasks), engaging (executing) the cogs and finally
retrieve the results. The results can be retrieved at any time, either
blocking or simply check back in a while to see if the cog has finished or not.

## Installation
```bash
cargo add rustycog
```

## Example
```rs
use rustycog::Machine;

let mut machine = Machine::powered(8);
let id0 = machine.insert_cog(|| 42);
let id1 = machine.insert_cog(|| {
    println!("Running task...");
    99
});

println!("Result of cog 0: {:?}", machine.wait_for_result(id0)); // Ok(42)
println!("Result of cog 1: {:?}", machine.wait_for_result(id1)); // Ok(99)

```

## Error Handling
See the documentation for more information about the different errors.
### Example
```rs
use rustycog::{Machine, error::CogError};

fn main() {
    let mut machine = Machine::<i32>::powered(8);
    let id = machine.insert_cog(|| {
        panic!("Oh no!");
    });

    match machine.wait_for_result(id) {
        Ok(result) => println!("Result: {}", result),
        Err(CogError::Panicked) => println!("The cog panicked!"),
        _ => println!("Unexpected error"),
    }
}
```

## Future Plans
- Dynamic Engine Management: Automatically adjust the amount of background threads
  depending on the current workload.
- Prioritization: Allowing certain cogs to be prioritized. For example a cog
  currently being waited for will get priority for decreased response times
- Cancel cogs before they are ran

## Why Choose RustyCog?
RustyCog provides a unique approach to task management in Rust,
allowing you to manage tasks like futures without actually being asynchronous.
Synchronous + asyncrhonous in one!

## Contrbuting
Contributions are welcome! Please open an issue to discuss your ideas, or fork the repo and open a pull request.

## Licence
This project is licensed under the MIT License. See [LICENSE] for details.
