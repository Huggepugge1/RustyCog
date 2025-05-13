use rustycog::cog_pool::CogPool;

fn main() {
    let mut pool = CogPool::new();
    pool.add_task(move || println!("Hello World from task 0"));
    pool.add_task(|| println!("Hello World from task 1"));
    pool.add_task(|| println!("Hello World from task 2"));

    pool.run();
}
