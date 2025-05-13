use rustycog::cog_pool::CogPool;

fn main() {
    let mut pool = CogPool::new();
    pool.add_task(|| println!("Hello World from task 0"));
    pool.add_task(|| println!("Hello World from task 1"));
    let int_id = pool.add_task_with_result(|| {
        println!("Hello World from task 2");
        Box::new(1)
    });

    pool.run();
    let result = pool.get_result::<i32>(int_id);
    println!("{:?}", result);
    let result = pool.wait_for_result::<i32>(int_id);
    println!("{:?}", result);
}
