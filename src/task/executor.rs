//! This module maintains a queue of TaskIds and processes the corresponding Tasks.
//! It makes use of Waker notifications and the halt instruction to sleep while there
//! are no ready Tasks, which is more efficient than polling the queue of TaskIds.

use super::{Task, TaskId};
use alloc::{collections::BTreeMap, sync::Arc, task::Wake};
use core::task::{Context, Poll, Waker};
use crossbeam_queue::ArrayQueue;

/// Executor maintains a queue of the TaskIds of ready Tasks, and maps of all
/// spawned Tasks' Waker and Task structs.
pub struct Executor {
    /// BTreeMap of Tasks indexed by their TaskIds
    tasks: BTreeMap<TaskId, Task>,

    /// Queue of TaskIds which Wakers will push TaskIds onto, and Executors will receive
    /// TaskIds from, before executing the corresponding Task
    ///
    /// The queue is wrapped in an atomic reference counter to enable shared ownership between
    /// Executors and Wakers
    task_queue: Arc<ArrayQueue<TaskId>>,

    /// BTreeMap of the Wakers of Tasks, indexed by the TaskId of the corresponding Task
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),

            // Task queue has capacity bounded at 100 to avoid any allocations, which could lead to a deadlock
            task_queue: Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
        }
    }

    /// Spawns a Task by adding it to the tasks map and pushing the TaskId to the task_queue
    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with same ID already in tasks");
        }
        self.task_queue.push(task_id).expect("queue full");
    }

    /// Process the TaskIds on the task_queue
    fn run_ready_tasks(&mut self) {
        // destructure `self` to avoid borrow checker errors
        let Self {
            tasks,
            task_queue,
            waker_cache,
        } = self;

        // Get the next TaskId from the task_queue
        while let Some(task_id) = task_queue.pop() {
            // Get the corresponding Task from the tasks map
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue, // task no longer exists
            };

            // Get the corresponding Waker (create one if it does not exist)
            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new_waker(task_id, task_queue.clone()));

            let mut context = Context::from_waker(waker);

            // Poll the task
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    // task done -> remove it and its cached Waker
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }

                // If the Task is not complete, do not readd its TaskId to the task_queue as it is not ready,
                // however do not remove the Task and its Waker from the tasks and waker_cache maps as they
                // are required for when it is ready
                Poll::Pending => {}
            }
        }
    }

    /// Loop which processes Tasks, and sleeps once there are no ready Tasks, until the next interrupt
    ///
    /// Interrupt handlers are the source of ready Tasks so sleeping until the next interrupt is more
    /// efficient than continuously polling the Task queue
    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }

    /// If there are no ready tasks, use the hlt instruction to sleep until the next interrupt
    ///
    /// If there are ready tasks, return
    fn sleep_if_idle(&self) {
        use x86_64::instructions::interrupts::{self, enable_and_hlt};

        // Disable interrupts while checking the task_queue to prevent racing with
        // interrupt handlers which add TaskIds to the task_queue
        interrupts::disable();
        if self.task_queue.is_empty() {
            // If the task queue is empty, re-enable interrupts and sleep until the next interrupt
            enable_and_hlt();
        } else {
            // If the task queue is not empty, re-enable interrupts and return, as the Task in the task queue must be processed
            interrupts::enable();
        }
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

/// The TaskWaker's job is to push its TaskId to the Executor's task_queue
struct TaskWaker {
    /// TaskId of the Task this TaskWaker is associated with
    task_id: TaskId,

    /// Reference to the Executor's task_queue
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    /// Wake the TaskWaker's Task by pushing its TaskId to the Executor's task_queue
    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task_queue full");
    }

    /// Creates a new Waker from the TaskWaker created with the task_id and task_queue arguments
    fn new_waker(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }
}

impl Wake for TaskWaker {
    /// Wake the TaskWaker's Task by pushing its TaskId to the Executor's task_queue
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    /// Wake the TaskWaker's Task by pushing its TaskId to the Executor's task_queue
    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}
