use std::{any::{Any, TypeId}, cell::RefCell, collections::HashMap, sync::Arc, thread::Thread};

use crossbeam_deque::{Injector, Stealer, Worker};
use parking_lot::{Mutex, RwLock};
mod function;
mod params;
pub use function::*;
pub use params::*;

use crate::world::World;
pub type TypeIdMap<T> = HashMap<TypeId, T>;
pub trait Task: 'static + Send + Sync {
    fn execute(&mut self, resources: &World);
    fn initialize(&mut self, resources: &World);
}
pub trait IntoTask<Input> {
    type Task: Task;
    fn into_task(self) -> Self::Task;
}
pub enum ControlFlow {
    RunOnce(Box<dyn Task>),
    Repeat(Box<dyn Task>),
}
pub struct Routines {
    threads: Mutex<Vec<std::thread::JoinHandle<()>>>,
    injector: Injector<ControlFlow>,
    stealers: Arc<RwLock<Vec<Stealer<ControlFlow>>>>,
    world: World,
}

fn find_task<T>(
    local: &Worker<T>,
    global: &Injector<T>,
    stealers: &[Stealer<T>],
) -> Option<T> {
    // Pop a task from the local queue, if not empty.
    local.pop().or_else(|| {
        // Otherwise, we need to look for a task elsewhere.
        std::iter::repeat_with(|| {
            // Try stealing a batch of tasks from the global queue.
            global.steal_batch_and_pop(local)
                // Or try stealing a task from one of the other threads.
                .or_else(|| stealers.iter().map(|s| s.steal()).collect())
        })
        // Loop while no task was stolen and any steal operation needs to be retried.
        .find(|s| !s.is_retry())
        // Extract the stolen task, if there is one.
        .and_then(|s| s.success())
    })
}
impl Routines {
    pub fn new(thread_count: usize) -> Arc<Self> {
        let scheduler = Arc::new(Self { threads: Mutex::new(vec![]), injector: Injector::new(), stealers: Arc::new(RwLock::new(vec![])), world: World::new() });
        for _ in 0..thread_count {
            scheduler.push_thread();
        }
        scheduler
    }
    pub fn insert_resource<T: 'static>(&self, resource: T) {
        self.world.insert_resource(resource);
    }
    pub fn push_thread(self: &Arc<Self>) {
        let thread = std::thread::spawn({
            let this = self.clone();
            move ||{
                let worker = Worker::<ControlFlow>::new_fifo();
                let read = this.stealers.read();
                loop {
                    if this.injector.len() > 1 {
                        this.unpark_threads(this.injector.len()-1);
                    }
                    if let Some(mut task) = find_task(&worker, &this.injector, &read) {
                        match &mut task {
                            ControlFlow::RunOnce(task) => {
                                task.execute( &this.world);
                                println!("EXECUTED {:?}", std::thread::current().id());
                            }
                            ControlFlow::Repeat(task_inner) => {
                                task_inner.execute( &this.world);
                                this.injector.push(task);
                            }
                        }
                    } else {
                        // If no tasks are to be executed, park the thread
                        println!("PARKED {:?}", std::thread::current().id());
                        std::thread::park();
                    }
                }
            }
        });
        self.threads.lock().push(thread);
    }
    fn unpark_threads(&self, count: usize) {
        let guard = self.threads.lock();
        let mut count = guard.len().min(count);
        for (i, thread) in guard.iter().enumerate() {
            if std::thread::current().id() == thread.thread().id() {
                continue;
            }
            if count == 0 {
                return;
            }
            thread.thread().unpark();
            count -= 1;
        }
    }
    pub fn run_once_task(self: &Arc<Self>, mut task: Box<dyn Task>) {
        task.initialize(&self.world);
        let len = self.injector.len();
        self.injector.push(ControlFlow::RunOnce(task));
        if len == 0 {
            self.unpark_threads(1);
        }
    }
    pub fn repeat_task(self: &Arc<Self>, mut task: Box<dyn Task>) {
        task.initialize(&self.world);
        let len = self.injector.len();
        self.injector.push(ControlFlow::Repeat(task));
        if len == 0 {
            self.unpark_threads(1);
        }
    }
}