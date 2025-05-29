use rustycog::{self, cog::Cog};

fn main() {
    let mut machine = rustycog::Machine::powered(1);

    let cogs = 1_000_000;

    for i in 0..cogs {
        let _ = machine.insert_cog(Cog::new(move || i));
    }

    for i in 0..cogs {
        let result = machine.get_result(i);
        println!("{result:?}");
    }

    for i in 0..cogs {
        let result = machine.wait_for_result(i);
        assert_eq!(result, Ok(i));
    }

    // std::thread::sleep(std::time::Duration::from_secs(10));
}
