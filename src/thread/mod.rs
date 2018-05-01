mod stack;
mod thread;
mod scheduler;
mod list;

pub use self::stack::Stack;
pub use self::thread::{Thread, State, ThreadId};

use arch::lock::PreemptLock;
use alloc::LinkedList;

pub const MAX_TASKS: usize = 65_536;

// lazy_static! {
    // static ref THREAD_RUN_QUEUE: PreemptLock<LinkedList<Thread>>
// }