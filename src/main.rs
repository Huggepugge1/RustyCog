use rustycog::cog_pool::CogPool;

fn main() {
    let mut pool = CogPool::<i32>::new();
    let id0 = pool.add_task(|| {
        println!("Hello World from task 0");
        33
    });
    let id1 = pool.add_task(|| {
        println!("Hello World from task 1");
        54327
    });

    pool.run();

    std::thread::sleep(std::time::Duration::from_micros(100));

    let result0 = pool.get_result(id0);
    let result1 = pool.get_result(id1);

    println!("{:?}, {:?}", result0, result1);

    let result0 = pool.wait_for_result(id0);
    let result1 = pool.wait_for_result(id1);

    println!("{:?}, {:?}", result0, result1);
}
