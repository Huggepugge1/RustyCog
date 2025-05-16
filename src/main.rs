use rustycog::Machine;

fn main() {
    let mut machine = Machine::<usize>::powered(8);

    let cogs = 1000;

    for i in 0..cogs {
        machine.insert_cog(move || i);
        // std::thread::sleep(std::time::Duration::from_secs(1));
    }

    for i in 0..cogs {
        let _result = machine.wait_for_result(i);
        // println!("Result: {:?}", result);
    }

    // std::thread::sleep(std::time::Duration::from_secs(10));
}
