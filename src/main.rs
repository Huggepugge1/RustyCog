use rustycog::Machine;

fn main() {
    let mut machine = Machine::<usize>::powered(8);

    let cogs = 1_000_000;

    for i in 0..100 {
        let mut cog_vec = Vec::new();
        for j in 0..(cogs / 100) {
            cog_vec.push(move || i * j)
        }
        machine.insert_cog_batch(cog_vec);
    }

    for i in 0..cogs {
        let result = machine.wait_for_result(i);
        println!("Result: {:?}", result);
    }

    // std::thread::sleep(std::time::Duration::from_secs(10));
}
