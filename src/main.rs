use std::any::Any;

use rustycog::Machine;

fn main() {
    let mut machine = Machine::<Box<dyn Any + Send>>::new();

    let cogs = 1;

    for _i in 0..cogs {
        machine.insert_cog(move || Box::new(3));
    }

    machine.run();

    for i in 0..cogs {
        println!("{:?}", machine.get_result(i));
        println!(
            "{:?}",
            machine
                .wait_for_result(i)
                .unwrap()
                .downcast_ref::<i32>()
                .unwrap()
                + 5
        );
        println!("{:?}", machine.wait_for_result(i));
    }

    // std::thread::sleep(std::time::Duration::from_secs(10));
}
