use rustycog::Machine;

fn main() {
    let mut machine = Machine::<usize>::powered(8);

    let cogs = 1_000_000;

    const BATCHES: usize = 10;
    for i in 0..BATCHES {
        let mut cog_vec = Vec::new();
        for j in 0..(cogs / BATCHES) {
            cog_vec.push(move || i * BATCHES + j)
        }
        machine.insert_cog_batch(cog_vec);
    }

    for i in 0..cogs {
        let _result = machine.wait_for_result(i);
        // println!("Result: {:?}", result);
    }

    // std::thread::sleep(std::time::Duration::from_secs(10));
}
