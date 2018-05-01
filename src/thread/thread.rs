use alloc::String;
use thread::Stack;
use arch::context::Context;
use arch::cpu;

// use super::SCHEDULER;

use nabi::{Result};

pub type ThreadId = u64;

/// The current state of a process.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum State {
    /// The thread is currently executing.
    Running,
    /// This thread is not currently running, but it's ready to execute.
    Ready,
    /// The thread has been suspended.
    Suspended,
    /// It's dead, Jim.
    Dead,
}

/// A single thread of execution.
#[derive(Debug)]
pub struct Thread {
    pub name: String,
    state: State,
    pub ctx: Context,
    pub stack: Stack,
    pub entry: extern fn(),
}

impl Thread {
    pub fn new<S: Into<String>>(name: S, stack_size: usize, entry: extern fn()) -> Result<Thread> {
        let stack = Stack::with_size(stack_size)?;
        let mut ctx = Context::new();
        ctx.rsp = stack.top() as usize;
        unsafe { ctx.push_stack(common_thread_entry as usize); }

        Ok(Thread {
            name: name.into(),
            state: State::Suspended,
            ctx,
            stack,
            entry,
        })
    }

    pub unsafe fn switch_to(&mut self, other: &Thread) {
        self.ctx.switch_to(&other.ctx);
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
    }

    pub fn ctx(&self) -> &Context {
        &self.ctx
    }

    pub fn ctx_mut(&mut self) -> &mut Context {
        &mut self.ctx
    }

    pub fn resume(&mut self) -> Result<usize> {
        self.state = State::Ready;



        Ok(0)
    }
}

pub extern fn common_thread_entry() -> ! {
    let thread = cpu::thread::get();

    // Execute the thread.
    (thread.entry)();

    // let current_id = SCHEDULER.current_id();

    // SCHEDULER.kill(current_id);

    unreachable!();
}
