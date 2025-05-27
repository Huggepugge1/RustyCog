use rustycog::Machine;

fn main() {
    let mut machine = Machine::powered(8);

    let cogs = 1_000;

    for i in 0..cogs {
        let _ = machine.insert_cog(move || i);
    }

    for i in 0..cogs {
        let _result = machine.wait_for_result(i);
    }

    // std::thread::sleep(std::time::Duration::from_secs(10));
}
