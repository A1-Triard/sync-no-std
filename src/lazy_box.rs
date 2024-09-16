use alloc::boxed::Box;
use core::alloc::Allocator;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::ptr::null_mut;
use core::sync::atomic::AtomicPtr;
use core::sync::atomic::Ordering::{AcqRel, Acquire};

pub struct LazyBox<T: LazyInit> {
    ptr: AtomicPtr<T>,
    allocator: T::Allocator,
    _phantom: PhantomData<T>,
}

pub trait LazyInit {
    type Allocator: Allocator + Clone;

    /// This is called before the box is allocated, to provide the value to
    /// move into the new box.
    ///
    /// It might be called more than once per LazyBox, as multiple threads
    /// might race to initialize it concurrently, each constructing and initializing
    /// their own box. All but one of them will be passed to `cancel_init` right after.
    fn init(allocator: Self::Allocator) -> Box<Self, Self::Allocator>;

    /// Any surplus boxes from `init()` that lost the initialization race
    /// are passed to this function for disposal.
    ///
    /// The default implementation calls destroy().
    fn cancel_init(x: Box<Self, Self::Allocator>) {
        Self::destroy(x);
    }

    /// This is called to destroy a used box.
    ///
    /// The default implementation just drops it.
    fn destroy(_: Box<Self, Self::Allocator>) { }
}

impl<T: LazyInit> LazyBox<T> {
    #[allow(dead_code)]
    pub const fn new_in(allocator: T::Allocator) -> Self {
        Self { ptr: AtomicPtr::new(null_mut()), allocator, _phantom: PhantomData }
    }

    pub const fn allocator(lb: &LazyBox<T>) -> &T::Allocator { &lb.allocator }

    fn get_pointer(&self) -> *mut T {
        let ptr = self.ptr.load(Acquire);
        if ptr.is_null() { self.initialize() } else { ptr }
    }

    fn initialize(&self) -> *mut T {
        let new_ptr = Box::into_raw(T::init(self.allocator.clone()));
        match self.ptr.compare_exchange(null_mut(), new_ptr, AcqRel, Acquire) {
            Ok(_) => new_ptr,
            Err(ptr) => {
                // Lost the race to another thread.
                // Drop the box we created, and use the one from the other thread instead.
                T::cancel_init(unsafe { Box::from_raw_in(new_ptr, self.allocator.clone()) });
                ptr
            }
        }
    }
}

impl<T: LazyInit> Deref for LazyBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.get_pointer() }
    }
}

impl<T: LazyInit> DerefMut for LazyBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.get_pointer() }
    }
}

impl<T: LazyInit> Drop for LazyBox<T> {
    fn drop(&mut self) {
        let ptr = *self.ptr.get_mut();
        if !ptr.is_null() {
            T::destroy(unsafe { Box::from_raw_in(ptr, self.allocator.clone()) });
        }
    }
}
