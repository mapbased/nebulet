use alloc::BTreeMap;
use alloc::arc::Arc;

use arch::lock::Spinlock;
use thread::{Thread, ThreadId};

/// Per-cpu thread list
pub struct ThreadList {
    /// map of thread id to thread.
    list: BTreeMap<ThreadId, Arc<Spinlock<Thread>>>,
    /// the next thread id
    next: ThreadId,
}

impl ThreadList {
    /// Creates a new, empty list.
    pub fn new() -> ThreadList {
        // initial idle thread
        let idle_thread = Thread::new("[idle]", 128, idle_thread_entry)
            .expect("Could not create the idle thread");

        let mut list = BTreeMap::new();

        list.insert(0, Arc::new(Spinlock::new(idle_thread)));

        ThreadList {
            list,
            next: 1,
        }
    }

    pub fn get(&self, id: ThreadId) -> Option<&Arc<Spinlock<Thread>>> {
        self.list.get(&id)
    }

    pub fn insert(&mut self, thread: Thread) -> Option<ThreadId> {
        if self.next >= super::MAX_TASKS as u64 {
            self.next = 1;
        }

        while self.list.contains_key(&self.next) {
            self.next += 1;
        }

        if self.next >= super::MAX_TASKS as u64 {
            None
        } else {
            let id = self.next;
            self.next += 1;

            debug_assert!(
                self.list
                    .insert(id, Arc::new(Spinlock::new(thread)))
                    .is_none(),
                "thread already exists"
            );

            Some(id)
        }
    }

    pub fn remove(&mut self, id: ThreadId) -> Option<Arc<Spinlock<Thread>>> {
        self.list.remove(&id)
    }
}

extern fn idle_thread_entry()  {
    use arch::interrupt::halt;
    loop {
        unsafe { halt(); }
    }
}
