use rustycog::{self, cog::Cog, types::CogId};

fn main() {
    let mut machine = rustycog::machine!(Cog, CogId, 8);
    // let mut machine = rustycog::cold_machine!(Cog, CogId, 8);

    let cogs = 1_000_000;

    for i in 0..cogs {
        let _ = machine.insert_cog(move || i);
    }

    println!("{:?}", machine.power());

    for i in 0..cogs {
        let result = machine.wait_for_result(i);
        assert_eq!(result, Ok(i));
    }

    // std::thread::sleep(std::time::Duration::from_secs(10));
}
