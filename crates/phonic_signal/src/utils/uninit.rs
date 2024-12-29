use std::mem::MaybeUninit;

pub const fn slice_as_uninit<T>(init: &[T]) -> &[MaybeUninit<T>] {
    unsafe { &*(init as *const [T] as *const [MaybeUninit<T>]) }
}

pub const fn slice_as_uninit_mut<T>(init: &mut [T]) -> &mut [MaybeUninit<T>] {
    unsafe { &mut *(init as *mut [T] as *mut [MaybeUninit<T>]) }
}

/// Assuming all the elements are initialized, get a slice to them.
///
/// # Safety
///
/// It is up to the caller to guarantee that the `MaybeUninit<T>` elements
/// really are in an initialized state.
/// Calling this when the content is not yet fully initialized causes undefined behavior.
pub const unsafe fn slice_as_init<T>(slice: &[MaybeUninit<T>]) -> &[T] {
    unsafe { &*(slice as *const [MaybeUninit<T>] as *const [T]) }
}

/// Assuming all the elements are initialized, get a mutable slice to them.
///
/// # Safety
///
/// It is up to the caller to guarantee that the `MaybeUninit<T>` elements
/// really are in an initialized state.
/// Calling this when the content is not yet fully initialized causes undefined behavior.
pub const unsafe fn slice_as_init_mut<T>(slice: &mut [MaybeUninit<T>]) -> &mut [T] {
    unsafe { &mut *(slice as *mut [MaybeUninit<T>] as *mut [T]) }
}

pub fn copy_to_uninit_slice<'a, T: Copy>(src: &[T], dst: &'a mut [MaybeUninit<T>]) -> &'a mut [T] {
    let uninit_src = slice_as_uninit(src);
    dst.copy_from_slice(uninit_src);

    unsafe { slice_as_init_mut(dst) }
}
