use rustycog::{self, cog::Cog};

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn hash_u64(x: usize) -> usize {
    let mut result: usize = 0;
    for _ in 0..1_000_000 {
        let mut hasher = DefaultHasher::new();
        x.hash(&mut hasher);
        result = result.checked_add(hasher.finish() as usize).unwrap_or(0);
    }
    result
}

fn main() {
    let mut machine = rustycog::Machine::powered(8);

    let cogs = 1_000;

    for i in 0..cogs {
        let _ = machine.insert_cog(Cog::new(move || hash_u64(i)));
    }

    for i in 0..cogs {
        let _result = machine.wait_for_result(i);
        // assert_eq!(result, Ok(i));
    }

    // std::thread::sleep(std::time::Duration::from_secs(10));
}
