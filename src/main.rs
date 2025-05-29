use rustycog::{
    Machine,
    cog::{Cog, PrioCog},
};

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
    use std::cmp::Ordering;

    use rustycog::cog::CogTrait;

    #[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
    enum Importance {
        Low,
        High,
        Critical,
    }

    impl From<usize> for Importance {
        fn from(x: usize) -> Self {
            match x % 3 {
                0 => Self::Low,
                1 => Self::High,
                2 => Self::Critical,
                _ => unreachable!(),
            }
        }
    }

    struct MyCog {
        importance: Importance,
        func: Option<Box<dyn FnOnce() -> usize + Send>>,
    }

    impl CogTrait for MyCog {
        type T = usize;

        fn run(&mut self) -> Self::T {
            let task = std::mem::take(&mut self.func).unwrap();
            task()
        }

        fn priority(&self, other: &Self) -> Ordering {
            self.importance.cmp(&other.importance)
        }
    }

    let mut machine = Machine::cold(1);

    let cogs = 1_000;

    for i in 0..cogs {
        let _ = machine.insert_cog(MyCog {
            func: Some(Box::new(move || {
                println!("{i:#03}: {:?}", Importance::from(i));
                i
            })),
            importance: Importance::from(i),
        });
    }

    let _ = machine.power();

    for i in 0..cogs {
        let result = machine.wait_for_result(i).unwrap();
        assert_eq!(result, i);
    }

    // std::thread::sleep(std::time::Duration::from_secs(10));
}
