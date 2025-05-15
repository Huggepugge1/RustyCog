use rustycog::Machine;

fn main() {
    let mut machine = Machine::<i32>::new();

    let cogs = 1_000_000;

    for i in 0..cogs {
        machine.insert_cog(move || i);
    }

    machine.run();

    // for i in 0..cogs {
    //     println!("{:?}", machine.wait_for_result(i));
    // }
}
