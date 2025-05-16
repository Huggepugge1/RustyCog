use rustycog::Machine;

fn main() {
    let mut machine = Machine::<i32>::powered(8);

    let cogs = 10;

    for i in 0..cogs {
        machine.insert_cog(move || {
            println!("Cog: {i}");
            i
        });
    }

    for i in 0..cogs {
        println!("Result: {}", machine.wait_for_result(i).unwrap());
    }

    // std::thread::sleep(std::time::Duration::from_secs(10));
}
