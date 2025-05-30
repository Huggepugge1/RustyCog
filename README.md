# RustyCog
Effortless Task Management with Dynamic Execution

## Overview
RustyCog is a lightweight task manager for Rust. It allows
you to schedule cogs (Tasks), engage (executing) them and finally
retrieve their results. Results can be retrieved at any time, either
blocking or simply check back in a while to see if the cog has finished.

## Installation
```bash
cargo add rustycog
```

## Example
```rs
use rustycog::{Machine, Cog};

let mut machine = Machine::powered(1);
let id0 = machine.insert_cog(Cog::new(|| 42));
let id1 = machine.insert_cog(Cog::new(|| {
    println!("Running task...");
    99
}));

println!("Result of cog 0: {:?}", machine.wait_for_result(id0).unwrap()); // Ok(42)
println!("Result of cog 1: {:?}", machine.wait_for_result(id1).unwrap()); // Ok(99)

```

### Priority cogs
```rs
use rustycog::{Machine, PrioCog};

// Cold = don't run immediatly
let mut machine = Machine::cold(1);

// Will run first!
let id0 = machine.insert_cog(PrioCog::new(|| 42), 0);
// Will run second!
let id1 = machine.insert_cog(PrioCog::new(|| {
    println!("Running task...");
    99
}, 1));

// Start running cogs
machine.power();

println!("Result of cog 0: {:?}", machine.wait_for_result(id0).unwrap()); // Ok(42)
println!("Result of cog 1: {:?}", machine.wait_for_result(id1).unwrap()); // Ok(99)

```

## Future Plans
- Dynamic Engine Management: Automatically adjust the amount of background threads
  depending on the current workload.
- Calcellation: Allow cogs to be cancelled before engagement.

## Why Choose RustyCog?
RustyCog provides a unique approach to task management in Rust,
allowing you to manage tasks like futures without actually being asynchronous.
Synchronous + asyncrhonous in one!

## Contributing
Contributions are welcome! Please open an issue to discuss your ideas, or fork the repo and open a pull request.

## License
This project is licensed under the MIT License. See [LICENSE] for details.
