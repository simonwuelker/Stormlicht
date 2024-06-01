//! A gc cell allows for mutating content on the gc heap
//!
//! The structure and code is very similar to a [RefCell](std::cell::RefCell)
use std::{
    cell::{Cell, UnsafeCell},
    cmp::Ordering,
    fmt,
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut},
    panic,
    ptr::NonNull,
};

use crate::Trace;

/// A mutable memory location with dynamically checked borrow rules.
///
/// Whenever the contained value is accessed (mutably), the cell and all its contained gc elements are rooted.
/// This is necessary since they might be moved from the value onto the stack (becoming a root) without us noticing
/// otherwise.
pub struct GcCell<T: ?Sized> {
    borrow: Cell<BorrowFlag>,

    #[cfg(feature = "debug_gccell")]
    /// Stores the location of the earliest currently active borrow.
    /// This gets updated whenever we go from having zero borrows
    /// to having a single borrow. When a borrow occurs, this gets included
    /// in the generated `BorrowError`/`BorrowMutError`
    borrowed_at: Cell<Option<&'static panic::Location<'static>>>,

    value: UnsafeCell<T>,
}

#[non_exhaustive]
pub struct BorrowError {
    #[cfg(feature = "debug_gccell")]
    location: &'static panic::Location<'static>,
}

#[non_exhaustive]
pub struct BorrowMutError {
    #[cfg(feature = "debug_gccell")]
    location: &'static panic::Location<'static>,
}

/// Tracks the number of outstanding borrows and whether or not the type is borrowed
///
/// If all the borrow count bits are set that indicates (a single) writing borrow, otherwise
/// the borrows are reading.
#[derive(Clone, Copy, Debug)]
struct BorrowFlag(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BorrowState {
    Unused,
    Reading,
    Writing,
}

impl BorrowFlag {
    const ROOT_BIT: usize = 1 << (usize::BITS - 1);
    const BORROW_COUNT_MASK: usize = !Self::ROOT_BIT;
    const MAX_BORROW_COUNT: usize = Self::BORROW_COUNT_MASK;

    #[inline]
    #[must_use]
    fn root(&self) -> Self {
        Self(self.0 | Self::ROOT_BIT)
    }

    #[inline]
    #[must_use]
    fn unroot(&self) -> Self {
        Self(self.0 & !Self::ROOT_BIT)
    }

    #[inline]
    #[must_use]
    fn is_rooted(&self) -> bool {
        self.0 & Self::ROOT_BIT != 0
    }

    #[inline]
    #[must_use]
    fn borrow_count(&self) -> usize {
        self.0 & Self::BORROW_COUNT_MASK
    }

    #[inline]
    #[must_use]
    fn borrow_state(&self) -> BorrowState {
        match self.borrow_count() {
            usize::MIN => BorrowState::Unused,
            Self::BORROW_COUNT_MASK => BorrowState::Writing,
            _ => BorrowState::Reading,
        }
    }

    #[inline]
    #[must_use]
    fn decrement_borrow_count(&self) -> Self {
        Self(self.0 - 1)
    }

    /// Increment the borrow count if possible, else do nothing
    ///
    /// Return value indicates whether or not the borrow count was updated
    #[inline]
    #[must_use]
    fn borrow_if_possible(&mut self) -> bool {
        if self.borrow_count() < Self::MAX_BORROW_COUNT - 1 {
            self.0 += 1;
            true
        } else {
            false
        }
    }

    #[inline]
    #[must_use]
    fn increment_borrow_count(&self) -> Self {
        debug_assert!(self.borrow_count() != Self::MAX_BORROW_COUNT);
        Self(self.0 + 1)
    }

    #[inline]
    #[must_use]
    fn mark_mutably_borrowed(&self) -> Self {
        debug_assert!(self.borrow_count() == 0);

        Self(self.0 | Self::BORROW_COUNT_MASK)
    }

    #[inline]
    #[must_use]
    fn zero_borrow_count(&self) -> Self {
        Self(self.0 & Self::ROOT_BIT)
    }
}

impl Default for BorrowFlag {
    fn default() -> Self {
        // Default flag is rooted without borrows
        Self(Self::ROOT_BIT)
    }
}

impl<T: Trace> GcCell<T> {
    /// Creates a new `GcCell` containing `value`.
    #[must_use]
    pub fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            borrow: Cell::new(BorrowFlag::default()),
            #[cfg(feature = "debug_gccell")]
            borrowed_at: Cell::new(None),
        }
    }

    /// Consumes the `GcCell`, returning the wrapped value.
    pub fn into_inner(self) -> T {
        // Since this function takes `self` (the `GcCell`) by value, the
        // compiler statically verifies that it is not currently borrowed.
        self.value.into_inner()
    }

    /// Replaces the wrapped value with a new one, returning the old value,
    /// without deinitializing either one.
    ///
    /// This function corresponds to [`std::mem::replace`].
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed.
    #[track_caller]
    pub fn replace(&self, t: T) -> T {
        mem::replace(&mut *self.borrow_mut(), t)
    }
}

impl<T: ?Sized + Trace> GcCell<T> {
    /// Immutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned `Ref` exits scope. Multiple
    /// immutable borrows can be taken out at the same time.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently mutably borrowed. For a non-panicking variant, use
    /// [`try_borrow`](#method.try_borrow).
    #[track_caller]
    pub fn borrow(&self) -> Ref<'_, T> {
        match self.try_borrow() {
            Ok(b) => b,
            Err(err) => panic!("Already mutably borrowed: {err:?}"),
        }
    }

    /// Immutably borrows the wrapped value, returning an error if the value is currently mutably
    /// borrowed.
    ///
    /// The borrow lasts until the returned `Ref` exits scope. Multiple immutable borrows can be
    /// taken out at the same time.
    ///
    /// This is the non-panicking variant of [`borrow`](#method.borrow).
    ///
    /// # Examples
    ///
    /// ```
    /// use gc::GcCell;
    ///
    /// let c = GcCell::new(5);
    ///
    /// {
    ///     let m = c.borrow_mut();
    ///     assert!(c.try_borrow().is_err());
    /// }
    ///
    /// ```
    /// {
    ///     let m = c.borrow();
    ///     assert!(c.try_borrow().is_ok());
    /// }
    #[cfg_attr(feature = "debug_gccell", track_caller)]
    pub fn try_borrow(&self) -> Result<Ref<'_, T>, BorrowError> {
        match BorrowRef::new(&self.borrow) {
            Some(b) => {
                #[cfg(feature = "debug_gccell")]
                {
                    // `borrowed_at` is always the *first* active borrow
                    if b.borrow.get().borrow_count() == 1 {
                        self.borrowed_at.set(Some(panic::Location::caller()));
                    }
                }

                // SAFETY: `BorrowRef` ensures that there is only immutable access
                // to the value while borrowed.
                let value = unsafe { NonNull::new_unchecked(self.value.get()) };

                Ok(Ref { value, borrow: b })
            },
            None => Err(BorrowError {
                #[cfg(feature = "debug_gccell")]
                location: self
                    .borrowed_at
                    .get()
                    .expect("Previous borrow location is None but value is borrowed"),
            }),
        }
    }

    /// Mutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned `RefMut` or all `RefMut`s derived
    /// from it exit scope. The value cannot be borrowed while this borrow is
    /// active.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed. For a non-panicking variant, use
    /// [`try_borrow_mut`](#method.try_borrow_mut).
    #[track_caller]
    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        match self.try_borrow_mut() {
            Ok(b) => b,
            Err(err) => panic!("Already borrowed: {err:?}"),
        }
    }

    /// Mutably borrows the wrapped value, returning an error if the value is currently borrowed.
    ///
    /// The borrow lasts until the returned `RefMut` or all `RefMut`s derived
    /// from it exit scope. The value cannot be borrowed while this borrow is
    /// active.
    ///
    /// This is the non-panicking variant of [`borrow_mut`](#method.borrow_mut).
    #[inline]
    #[cfg_attr(feature = "debug_gccell", track_caller)]
    pub fn try_borrow_mut(&self) -> Result<RefMut<'_, T>, BorrowMutError> {
        match BorrowRefMut::new(&self.borrow) {
            Some(b) => {
                #[cfg(feature = "debug_gccell")]
                {
                    self.borrowed_at.set(Some(panic::Location::caller()));
                }

                // SAFETY: `BorrowRefMut` guarantees unique access.
                let value = unsafe { NonNull::new_unchecked(self.value.get()) };

                // The value is now accessible on the stack (and must therefore be rooted)
                if !self.borrow.get().is_rooted() {
                    unsafe { value.as_ref() }.root();
                }

                Ok(RefMut {
                    value,
                    borrow: b,
                    marker: PhantomData,
                })
            },
            None => Err(BorrowMutError {
                // If a borrow occurred, then we must already have an outstanding borrow,
                // so `borrowed_at` will be `Some`
                #[cfg(feature = "debug_gccell")]
                location: self.borrowed_at.get().unwrap(),
            }),
        }
    }

    /// Returns a raw pointer to the underlying data in this cell.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gc::GcCell;
    ///
    /// let c = GcCell::new(5);
    ///
    /// let ptr = c.as_ptr();
    /// ```
    #[inline]
    #[must_use]
    pub fn as_ptr(&self) -> NonNull<T> {
        NonNull::new(self.value.get()).expect("Cell value ptr should never be null")
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this method borrows `GcCell` mutably, it is statically guaranteed
    /// that no borrows to the underlying data exist. The dynamic checks inherent
    /// in [`borrow_mut`] and most other methods of `GcCell` are therefore
    /// unnecessary.
    ///
    /// This method can only be called if `GcCell` can be mutably borrowed,
    /// which in general is only the case directly after the `GcCell` has
    /// been created. In these situations, skipping the aforementioned dynamic
    /// borrowing checks may yield better ergonomics and runtime-performance.
    ///
    /// In most situations where `GcCell` is used, it can't be borrowed mutably.
    /// Use [`borrow_mut`] to get mutable access to the underlying data then.
    ///
    /// [`borrow_mut`]: GcCell::borrow_mut()
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gc::GcCell;
    ///
    /// let mut c = GcCell::new(5);
    /// *c.get_mut() += 1;
    ///
    /// assert_eq!(c, GcCell::new(6));
    /// ```
    #[inline]
    #[must_use]
    pub fn get_mut(&mut self) -> &mut T {
        self.value.get_mut()
    }
}

impl<T: Default + Trace> GcCell<T> {
    /// Takes the wrapped value, leaving `Default::default()` in its place.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed.
    ///
    /// # Examples
    ///
    /// ```
    /// use gc::GcCell;
    ///
    /// let c = GcCell::new(5);
    /// let five = c.take();
    ///
    /// assert_eq!(five, 5);
    /// assert_eq!(c.into_inner(), 0);
    /// ```
    pub fn take(&self) -> T {
        self.replace(Default::default())
    }
}

impl<T: ?Sized + Trace + PartialOrd> PartialOrd for GcCell<T> {
    /// # Panics
    ///
    /// Panics if the value in either `GcCell` is currently mutably borrowed.
    #[inline]
    fn partial_cmp(&self, other: &GcCell<T>) -> Option<Ordering> {
        self.borrow().partial_cmp(&*other.borrow())
    }

    /// # Panics
    ///
    /// Panics if the value in either `GcCell` is currently mutably borrowed.
    #[inline]
    fn lt(&self, other: &GcCell<T>) -> bool {
        *self.borrow() < *other.borrow()
    }

    /// # Panics
    ///
    /// Panics if the value in either `GcCell` is currently mutably borrowed.
    #[inline]
    fn le(&self, other: &GcCell<T>) -> bool {
        *self.borrow() <= *other.borrow()
    }

    /// # Panics
    ///
    /// Panics if the value in either `GcCell` is currently mutably borrowed.
    #[inline]
    fn gt(&self, other: &GcCell<T>) -> bool {
        *self.borrow() > *other.borrow()
    }

    /// # Panics
    ///
    /// Panics if the value in either `GcCell` is currently mutably borrowed.
    #[inline]
    fn ge(&self, other: &GcCell<T>) -> bool {
        *self.borrow() >= *other.borrow()
    }
}

impl<T: ?Sized + Trace + Ord> Ord for GcCell<T> {
    /// # Panics
    ///
    /// Panics if the value in either `GcCell` is currently mutably borrowed.
    #[inline]
    fn cmp(&self, other: &GcCell<T>) -> Ordering {
        self.borrow().cmp(&*other.borrow())
    }
}

impl<T: ?Sized + Trace + PartialEq> PartialEq for GcCell<T> {
    /// # Panics
    ///
    /// Panics if the value in either `GcCell` is currently mutably borrowed.
    #[inline]
    fn eq(&self, other: &GcCell<T>) -> bool {
        *self.borrow() == *other.borrow()
    }
}

impl<T: ?Sized + Trace + Eq> Eq for GcCell<T> {}

impl<T: Trace> From<T> for GcCell<T> {
    /// Creates a new `GcCell<T>` containing the given value.
    fn from(t: T) -> GcCell<T> {
        GcCell::new(t)
    }
}

struct BorrowRef<'b> {
    borrow: &'b Cell<BorrowFlag>,
}

impl<'b> BorrowRef<'b> {
    fn new(borrow: &'b Cell<BorrowFlag>) -> Option<BorrowRef<'b>> {
        let mut flag = borrow.get();

        if flag.borrow_if_possible() {
            borrow.set(flag);
            Some(BorrowRef { borrow })
        } else {
            None
        }
    }
}

impl Drop for BorrowRef<'_> {
    #[inline]
    fn drop(&mut self) {
        let borrow = self.borrow.get();
        debug_assert!(borrow.borrow_state() == BorrowState::Reading);
        self.borrow.set(borrow.decrement_borrow_count());
    }
}

impl Clone for BorrowRef<'_> {
    #[inline]
    fn clone(&self) -> Self {
        // Since this Ref exists, we know the borrow flag
        // is a reading borrow.
        let borrow = self.borrow.get();
        debug_assert_eq!(borrow.borrow_state(), BorrowState::Reading);

        // Prevent the borrow counter from overflowing into
        // a writing borrow.
        assert!(borrow.borrow_count() != BorrowFlag::BORROW_COUNT_MASK);
        self.borrow.set(borrow.increment_borrow_count());

        BorrowRef {
            borrow: self.borrow,
        }
    }
}

/// Wraps a borrowed reference to a value in a `GcCell` box.
#[must_not_suspend = "holding a Ref across suspend points can cause BorrowErrors"]
pub struct Ref<'b, T: ?Sized + 'b> {
    // NB: we use a pointer instead of `&'b T` to avoid `noalias` violations, because a
    // `Ref` argument doesn't hold immutability for its whole scope, only until it drops.
    // `NonNull` is also covariant over `T`, just like we would have with `&T`.
    value: NonNull<T>,
    borrow: BorrowRef<'b>,
}

impl<T: ?Sized> Deref for Ref<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        // SAFETY: the value is accessible as long as we hold our borrow.
        unsafe { self.value.as_ref() }
    }
}

impl<'b, T: ?Sized> Ref<'b, T> {
    /// Copies a `Ref`.
    ///
    /// The `GcCell` is already immutably borrowed, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `Ref::clone(...)`. A `Clone` implementation or a method would interfere
    /// with the widespread use of `r.borrow().clone()` to clone the contents of
    /// a `GcCell`.
    #[inline]
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn clone(orig: &Ref<'b, T>) -> Ref<'b, T> {
        Ref {
            value: orig.value,
            borrow: orig.borrow.clone(),
        }
    }

    /// Makes a new `Ref` for a component of the borrowed data.
    ///
    /// The `GcCell` is already immutably borrowed, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as `Ref::map(...)`.
    /// A method would interfere with methods of the same name on the contents
    /// of a `GcCell` used through `Deref`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gc::{GcCell, Ref};
    ///
    /// let c = GcCell::new((5, 'b'));
    /// let b1: Ref<'_, (u32, char)> = c.borrow();
    /// let b2: Ref<'_, u32> = Ref::map(b1, |t| &t.0);
    /// assert_eq!(*b2, 5)
    /// ```
    #[inline]
    pub fn map<U: ?Sized, F>(orig: Ref<'b, T>, f: F) -> Ref<'b, U>
    where
        F: FnOnce(&T) -> &U,
    {
        Ref {
            value: NonNull::from(f(&*orig)),
            borrow: orig.borrow,
        }
    }

    /// Makes a new `Ref` for an optional component of the borrowed data. The
    /// original guard is returned as an `Err(..)` if the closure returns
    /// `None`.
    ///
    /// The `GcCell` is already immutably borrowed, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `Ref::filter_map(...)`. A method would interfere with methods of the same
    /// name on the contents of a `GcCell` used through `Deref`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gc::{GcCell, Ref};
    ///
    /// let c = GcCell::new(vec![1, 2, 3]);
    /// let b1: Ref<'_, Vec<u32>> = c.borrow();
    /// let b2: Result<Ref<'_, u32>, _> = Ref::filter_map(b1, |v| v.get(1));
    /// assert_eq!(*b2.unwrap(), 2);
    /// ```
    #[inline]
    pub fn filter_map<U: ?Sized, F>(orig: Ref<'b, T>, f: F) -> Result<Ref<'b, U>, Self>
    where
        F: FnOnce(&T) -> Option<&U>,
    {
        match f(&*orig) {
            Some(value) => Ok(Ref {
                value: NonNull::from(value),
                borrow: orig.borrow,
            }),
            None => Err(orig),
        }
    }

    /// Splits a `Ref` into multiple `Ref`s for different components of the
    /// borrowed data.
    ///
    /// The `GcCell` is already immutably borrowed, so this cannot fail.
    ///
    /// This is an associated function that needs to be used as
    /// `Ref::map_split(...)`. A method would interfere with methods of the same
    /// name on the contents of a `GcCell` used through `Deref`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gc::{Ref, GcCell};
    ///
    /// let cell = GcCell::new([1, 2, 3, 4]);
    /// let borrow = cell.borrow();
    /// let (begin, end) = Ref::map_split(borrow, |slice| slice.split_at(2));
    /// assert_eq!(*begin, [1, 2]);
    /// assert_eq!(*end, [3, 4]);
    /// ```
    #[inline]
    pub fn map_split<U: ?Sized, V: ?Sized, F>(orig: Ref<'b, T>, f: F) -> (Ref<'b, U>, Ref<'b, V>)
    where
        F: FnOnce(&T) -> (&U, &V),
    {
        let (a, b) = f(&*orig);
        let borrow = orig.borrow.clone();
        (
            Ref {
                value: NonNull::from(a),
                borrow,
            },
            Ref {
                value: NonNull::from(b),
                borrow: orig.borrow,
            },
        )
    }

    /// Convert into a reference to the underlying data.
    ///
    /// The underlying `GcCell` can never be mutably borrowed from again and will always appear
    /// already immutably borrowed. It is not a good idea to leak more than a constant number of
    /// references. The `GcCell` can be immutably borrowed again if only a smaller number of leaks
    /// have occurred in total.
    ///
    /// This is an associated function that needs to be used as
    /// `Ref::leak(...)`. A method would interfere with methods of the
    /// same name on the contents of a `GcCell` used through `Deref`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gc::{GcCell, Ref};
    /// let cell = GcCell::new(0);
    ///
    /// let value = Ref::leak(cell.borrow());
    /// assert_eq!(*value, 0);
    ///
    /// assert!(cell.try_borrow().is_ok());
    /// assert!(cell.try_borrow_mut().is_err());
    /// ```
    pub fn leak(orig: Ref<'b, T>) -> &'b T {
        // By forgetting this Ref we ensure that the borrow counter in the GcCell can't go back to
        // UNUSED within the lifetime `'b`. Resetting the reference tracking state would require a
        // unique reference to the borrowed GcCell. No further mutable references can be created
        // from the original cell.
        mem::forget(orig.borrow);

        // SAFETY: after forgetting, we can form a reference for the rest of lifetime `'b`.
        unsafe { orig.value.as_ref() }
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for Ref<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T: ?Sized + fmt::Display> fmt::Display for Ref<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

struct BorrowRefMut<'b> {
    borrow: &'b Cell<BorrowFlag>,
}

impl Drop for BorrowRefMut<'_> {
    #[inline]
    fn drop(&mut self) {
        let borrow = self.borrow.get();
        debug_assert!(borrow.borrow_state() == BorrowState::Writing);
        self.borrow.set(borrow.zero_borrow_count());
    }
}

#[must_not_suspend = "holding a RefMut across suspend points can cause BorrowErrors"]
pub struct RefMut<'b, T: ?Sized> {
    // NB: we use a pointer instead of `&'b mut T` to avoid `noalias` violations, because a
    // `RefMut` argument doesn't hold exclusivity for its whole scope, only until it drops.
    value: NonNull<T>,

    #[allow(dead_code)]
    borrow: BorrowRefMut<'b>,
    // `NonNull` is covariant over `T`, so we need to reintroduce invariance.
    marker: PhantomData<&'b mut T>,
}

impl<'b> BorrowRefMut<'b> {
    #[inline]
    fn new(borrow: &'b Cell<BorrowFlag>) -> Option<BorrowRefMut<'b>> {
        let flag = borrow.get();
        match flag.borrow_state() {
            BorrowState::Unused => {
                borrow.set(flag.mark_mutably_borrowed());
                Some(BorrowRefMut { borrow })
            },
            _ => None,
        }
    }
}

impl<T: ?Sized> Deref for RefMut<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        // SAFETY: the value is accessible as long as we hold our borrow.
        unsafe { self.value.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for RefMut<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: the value is accessible as long as we hold our borrow.
        unsafe { self.value.as_mut() }
    }
}

impl<T: ?Sized + fmt::Debug> fmt::Debug for RefMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T: ?Sized + fmt::Display> fmt::Display for RefMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

unsafe impl<T: ?Sized + Trace> Trace for GcCell<T> {
    fn trace(&self) {
        // Don't need to trace if the value is borrowed mutably, since it is already rooted
        // in that case
        if self.borrow.get().borrow_state() != BorrowState::Writing {
            unsafe { self.as_ptr().as_ref() }.trace()
        }
    }

    fn root(&self) {
        let current_state = self.borrow.get();
        debug_assert!(!current_state.is_rooted(), "GcCell is already rooted");

        self.borrow.set(current_state.root());

        if self.borrow.get().borrow_state() != BorrowState::Writing {
            unsafe { self.as_ptr().as_ref() }.root()
        }
    }

    fn unroot(&self) {
        let current_state = self.borrow.get();
        debug_assert!(current_state.is_rooted(), "GcCell is not rooted");
        self.borrow.set(current_state.unroot());

        if current_state.borrow_state() != BorrowState::Writing {
            unsafe { self.as_ptr().as_ref() }.unroot()
        }
    }
}

impl fmt::Debug for BorrowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("BorrowError");

        #[cfg(feature = "debug_gccell")]
        builder.field("location", self.location);

        builder.finish()
    }
}

impl fmt::Display for BorrowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt("already mutably borrowed", f)
    }
}

impl fmt::Debug for BorrowMutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut builder = f.debug_struct("BorrowMutError");

        #[cfg(feature = "debug_gccell")]
        builder.field("location", self.location);

        builder.finish()
    }
}

impl fmt::Display for BorrowMutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt("already borrowed", f)
    }
}

impl<T: ?Sized + Trace + fmt::Debug> fmt::Debug for GcCell<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("GcCell");
        match self.try_borrow() {
            Ok(borrow) => d.field("value", &borrow),
            Err(_) => d.field("value", &format_args!("<borrowed>")),
        };
        d.finish()
    }
}

impl<T: ?Sized + Trace + Default> Default for GcCell<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}
