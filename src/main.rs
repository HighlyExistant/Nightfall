use std::{any::{Any, TypeId}, cell::RefCell, collections::HashMap, time::Duration};

use nightfall_allocators::arena::StandardArena;
use nightfall_collections::routines::{IntoTask, Routines, Task};
fn func(res: nightfall_collections::routines::Res<u32>) {
    println!("EXEC from thread {:?}", std::thread::current().id());
}
fn func2(res: nightfall_collections::routines::Res<u32>) {
    println!("EXEC 2 from thread {:?}", std::thread::current().id());
}
fn func3(res: nightfall_collections::routines::Res<u32>) {
    println!("EXEC 3 from thread {:?}", std::thread::current().id());
}
fn func4(res: nightfall_collections::routines::Res<u32>) {
    println!("EXEC 4 from thread {:?}", std::thread::current().id());
}
fn main() {
    let scheduler = Routines::new(5);
    let mut task = Box::new(func.into_task());
    let mut task2 = Box::new(func2.into_task());
    let mut task3 = Box::new(func3.into_task());
    let mut task4 = Box::new(func4.into_task());
    let mut task5 = Box::new(func3.into_task());
    let mut task6 = Box::new(func2.into_task());
    scheduler.insert_resource(0u32);
    scheduler.run_once_task(task);
    scheduler.run_once_task(task2);
    scheduler.run_once_task(task3);
    std::thread::sleep(Duration::from_secs(2));
    println!("\n\n\n\n\n");
    scheduler.run_once_task(task4);
    scheduler.run_once_task(task6);
    scheduler.repeat_task(task5);
    std::thread::sleep(Duration::from_secs(2));
}