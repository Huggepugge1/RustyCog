use rustycog::cog_pool::CogPool;

fn main() {
    let mut pool = CogPool::new();
    let _id1 = pool.add_task(|| println!("Hello World from task 0"));
    let _id2 = pool.add_task(|| println!("Hello World from task 1"));
    let id3 = pool.add_task_with_result(|| {
        println!("Hello World from task 2");
        Box::new(1)
    });

    pool.run();
    let result = pool.get_result::<i32>(id3);
    println!("{:?}", result);
    let result = pool.wait_for_result::<i32>(id3);
    println!("{:?}", result);
    let result = pool.wait_for_result::<()>(_id1);
    println!("{:?}", result);
}
