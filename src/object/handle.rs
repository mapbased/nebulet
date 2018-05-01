use alloc::arc::Arc;
use alloc::boxed::Box;
use core::any::{TypeId, Any};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

// use arch::lock::{Spinlock, SpinGuard};
use spin::{Mutex, MutexGuard};
use common::table::{Table, TableIndex};
use nabi::{Result, Error};

bitflags! {
    pub struct HandleRights: u32 {
        const DUPLICATE = 1 << 0;
        const TRANSFER  = 1 << 1;
    }
}

pub struct HandleCast<'guard, T> {
    guard: MutexGuard<'guard, Box<Any>>,
    phantom: PhantomData<T>,
}

impl<'guard, T> Deref for HandleCast<'guard, T> {
    type Target = T;

    fn deref(&self) -> &T {
        use core::ops::Deref;
        unsafe {
            let boxed: &Box<Any> = self.guard.deref();
            let raw: *const Any = boxed.deref() as *const Any;
            &*(raw as *const T)
        }
    }
}

impl<'guard, T> DerefMut for HandleCast<'guard, T> {
    fn deref_mut(&mut self) -> &mut T {
        use core::ops::DerefMut;
        unsafe {
            let boxed: &mut Box<Any> = self.guard.deref_mut();
            let raw: *mut Any = boxed.deref_mut() as *mut Any;
            &mut*(raw as *mut T)
        }
    }
}

/// A Handle represents an atomically reference-counted object with specfic rights.
/// Handles can be duplicated if they have the `HandleRights::DUPLICATE` right.
#[derive(Debug)]
pub struct Handle {
    obj_ref: Arc<Mutex<Box<Any>>>,
    rights: HandleRights,
    type_id: TypeId,
}

impl Handle {
    fn new<T: Any>(object: T, rights: HandleRights) -> Handle {
        Handle {
            obj_ref: Arc::new(Mutex::new(Box::new(object))),
            rights,
            type_id: TypeId::of::<T>(),
        }
    }

    /// Duplicate the handle if it has the `DUPLICATE` right.
    fn duplicate(&self, new_rights: HandleRights) -> Option<Handle> {
        if self.rights.contains(new_rights | HandleRights::DUPLICATE) {
            // `new_rights` contains the same or fewer rights and `HandleRights::DUPLICATE`
            // so it's okay to duplicate it.
            Some(Handle {
                obj_ref: Arc::clone(&self.obj_ref),
                rights: new_rights,
                type_id: self.type_id,
            })
        } else {
            None
        }
    }

    /// Retrive the inner type if it is the specified type
    pub fn lock_cast<T: Any>(&self) -> Result<HandleCast<T>> {
        if self.type_id == TypeId::of::<T>() {
            Ok(HandleCast {
                guard: self.obj_ref.lock(),
                phantom: PhantomData,
            })
        } else {
            Err(Error::WRONG_TYPE)
        }
    }
}

pub struct HandleTable {
    table: Table<Handle>,
}

impl HandleTable {
    pub fn new() -> Self {
        HandleTable {
            table: Table::new(),
        }
    }

    pub fn get_handle(&self, index: TableIndex) -> Result<&Handle> {
        self.table.get(index)
            .ok_or(Error::NOT_FOUND)
    }

    pub fn allocate<'table, T: Any>(&mut self, object: T, rights: HandleRights) -> Result<TableIndex> {
        let handle = Handle::new(object, rights);
        self.table.allocate(handle)
    }

    pub fn free(&mut self, index: TableIndex) -> Result<()> {
        self.table
            .free(index)
    }

    pub fn duplicate(&mut self, index: TableIndex, new_rights: HandleRights) -> Result<TableIndex> {
        let handle = self.get_handle(index)?;
        let new_handle = handle.duplicate(new_rights)
            .ok_or(Error::ACCESS_DENIED)?; // can't duplicate a handle without the right rights
        
        self.table.allocate(new_handle)
    }
}