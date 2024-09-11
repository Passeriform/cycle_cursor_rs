//! # Cycle Cursor
//!
//! A cyclic bidirectional cursor implementation over generic iterators.
//!
//! To begin, create an instance of the [`CycleCursor`]
//! ```
//! # use cycle_cursor::CycleCursor;
//! # use std::collections::BTreeSet;
//!
//! // Cursor from [`Vec`]
//! let mut vec_cursor = CycleCursor::from(vec![1, 2, 3, 4]);
//! // Cursor from [`BTreeSet`]
//! let mut btree_cursor = CycleCursor::from(BTreeSet::from([2, 2, 4, 6, 8, 8]));
//!
//! // Cyclically go to next element
//! vec_cursor.cycle_next();
//!
//! // Cyclically go to previous element
//! vec_cursor.cycle_prev();
//! vec_cursor.cycle_prev();
//!
//! // Get the current pointed element
//! assert_eq!(vec_cursor.get().unwrap(), &3);
//!
//! // Cyclically peek an element without moving the cursor
//! assert_eq!(vec_cursor.peek(5).unwrap(), &4);
//! assert_eq!(vec_cursor.peek(-2).unwrap(), &1);
//! assert_eq!(vec_cursor.get().unwrap(), &3);
//!
//! // Cyclically seek the cursor by signed offset
//! vec_cursor.seek(5);
//! assert_eq!(vec_cursor.get().unwrap(), &4);
//! vec_cursor.seek(-2);
//! assert_eq!(vec_cursor.get().unwrap(), &2);
//! ```
use std::ops::{Deref, DerefMut};

// TODO: Convert to no_std

/// Implements a cycling, seekable and peekable cursor over an iterable.
///
/// By default the cursor points to [`None`] value. A first call to
/// [`Self::cycle_next()`] or [`Self::cycle_prev()`] is required to start pointing to an
/// element. This has implementation benefits and thus cursor must be tested for
/// [`None`] value before usage.
///
/// # Examples
/// ```
/// # use cycle_cursor::CycleCursor;
/// #
/// let source = vec![1, 2, 3, 4];
/// let mut cycle_cursor = CycleCursor::from(source);
///
/// assert_eq!(cycle_cursor.pos, None);
///
/// cycle_cursor.cycle_next();
/// assert_eq!(cycle_cursor.pos.unwrap(), 0);
/// assert_eq!(cycle_cursor.get().unwrap(), &1);
///
/// cycle_cursor.cycle_next();
/// cycle_cursor.cycle_next();
/// cycle_cursor.cycle_next();
/// assert_eq!(cycle_cursor.pos.unwrap(), 3);
/// assert_eq!(cycle_cursor.get().unwrap(), &4);
///
/// // Cycles to first element
/// cycle_cursor.cycle_next();
/// assert_eq!(cycle_cursor.pos.unwrap(), 0);
/// assert_eq!(cycle_cursor.get().unwrap(), &1);
/// ```
///
/// ```
/// # use cycle_cursor::CycleCursor;
/// #
/// let source = vec![1, 2, 3, 4];
/// let mut cycle_cursor = CycleCursor::from(source);
///
/// assert_eq!(cycle_cursor.pos, None);
///
/// // Cycles back to last element
/// cycle_cursor.cycle_prev();
/// assert_eq!(cycle_cursor.pos.unwrap(), 3);
/// assert_eq!(cycle_cursor.get().unwrap(), &4);
///
/// cycle_cursor.cycle_prev();
/// cycle_cursor.cycle_prev();
/// cycle_cursor.cycle_prev();
/// assert_eq!(cycle_cursor.pos.unwrap(), 0);
/// assert_eq!(cycle_cursor.get().unwrap(), &1);
/// ```
///
/// CycleCursor implements [`Deref`] and [`DerefMut`] to dereference to the
/// underlying [`Vec`] to expose common collection methods.
/// Instead of using indexing directly, consider using [`Self::get()`] method to
/// safely access the element.
///
/// # Possible Undefined Behavior
/// [Any modification to
/// [`DerefMut`] will keep the cursor position unchanged. Accessing the cursor
/// at this point is undefined behavior. To fix, call [`Self::cycle_prev()`] or
/// [`Self::cycle_next()`] on it. This is another reason to always to [`Self::get()`]
/// instead of indexing directly.]
///
/// ```
/// # use std::panic;
/// # use cycle_cursor::CycleCursor;
/// #
/// let source = vec![1, 2, 3, 4];
///
/// let mut cycle_cursor = CycleCursor::from(source);
///
/// cycle_cursor.cycle_next();
/// cycle_cursor.cycle_next();
/// cycle_cursor.cycle_next();
/// cycle_cursor.cycle_next();
///
/// cycle_cursor.remove(0);
///
/// assert_eq!(cycle_cursor.pos.unwrap(), 3);
///
/// let result = panic::catch_unwind(|| cycle_cursor[cycle_cursor.pos.unwrap()]);
/// assert!(result.is_err());
/// ```
#[derive(Clone, Debug, Default)]
pub struct CycleCursor<T> {
    /// Inner vector that holds the source iterator.
    /// <div class="warning">This is directly accessible but mutable access may lead to
    /// undefined behavior. See [`Self#possible-undefined-behavior`]</div>
    pub inner: Vec<T>,
    /// Cursor pointing to pos
    pub pos: Option<usize>,
}

/// Convert from an iterator to a `CycleCursor`
impl<I> From<I> for CycleCursor<I::Item>
where
    I: IntoIterator,
    I::Item: Clone,
{
    fn from(inner: I) -> Self {
        Self {
            inner: inner.into_iter().collect(),
            pos: None,
        }
    }
}

/// Implementations for `CycleCursor`
impl<T> CycleCursor<T> {
    /// Moves the cursor to the next element. If no element exists, wrap back to
    /// the first element.
    pub fn cycle_next(&mut self) {
        let max_items = self.inner.len();
        if max_items == 0 {
            return;
        }

        #[allow(clippy::integer_division_remainder_used)]
        let pos = (self.pos.unwrap_or(max_items - 1) + max_items + 1) % max_items;
        self.pos = Some(pos);
    }

    /// Moves the cursor to the previous element. If no element exists, wrap to
    /// the last element.
    pub fn cycle_prev(&mut self) {
        let max_items = self.inner.len();
        if max_items == 0 {
            return;
        }

        #[allow(clippy::integer_division_remainder_used)]
        let pos = (self.pos.unwrap_or(max_items) + max_items - 1) % max_items;
        self.pos = Some(pos);
    }

    /// Peek element at an offset from the current cursor position
    /// (positive/negative)
    ///
    /// This method does not modify the cursor position.
    ///
    /// # Examples
    /// ```
    /// # use cycle_cursor::CycleCursor;
    /// #
    /// let source = vec![1, 2, 3, 4];
    ///
    /// let mut cycle_cursor = CycleCursor::from(source);
    ///
    /// cycle_cursor.cycle_next();
    /// cycle_cursor.cycle_next();
    /// assert_eq!(cycle_cursor.peek(3).unwrap(), &1);
    /// assert_eq!(cycle_cursor.peek(-3).unwrap(), &3);
    /// ```
    ///
    /// ```
    /// # use cycle_cursor::CycleCursor;
    /// #
    /// let source: Vec<usize> = vec![];
    ///
    /// let cycle_cursor = CycleCursor::from(source);
    ///
    /// assert_eq!(cycle_cursor.peek(2), None);
    /// ```
    pub fn peek(&self, peek_distance: isize) -> Option<&T> {
        let max_items = self.inner.len();
        if max_items == 0 {
            return None;
        }

        let norm_peek_distance = if peek_distance < 0 {
            (max_items as isize) + peek_distance
        } else {
            peek_distance
        } as usize;

        #[allow(clippy::integer_division_remainder_used)]
        let pos = (self.pos.unwrap_or(max_items - 1) + max_items + norm_peek_distance) % max_items;
        self.inner.get(pos)
    }

    /// Move the cursor seek by an offset from the current cursor position
    /// (positive/negative)
    ///
    /// # Examples
    /// ```
    /// # use cycle_cursor::CycleCursor;
    /// #
    /// let source = vec![1, 2, 3, 4];
    ///
    /// let mut cycle_cursor = CycleCursor::from(source);
    ///
    /// cycle_cursor.cycle_next();
    /// cycle_cursor.cycle_next();
    ///
    /// cycle_cursor.seek(3);
    /// assert_eq!(cycle_cursor.get().unwrap(), &1);
    ///
    /// cycle_cursor.seek(-3);
    /// assert_eq!(cycle_cursor.get().unwrap(), &2);
    /// ```
    ///
    /// ```
    /// # use cycle_cursor::CycleCursor;
    /// #
    /// let source: Vec<usize> = vec![];
    ///
    /// let mut cycle_cursor = CycleCursor::from(source);
    ///
    /// cycle_cursor.seek(2);
    /// assert_eq!(cycle_cursor.get(), None);
    /// ```
    pub fn seek(&mut self, seek_distance: isize) {
        let max_items = self.inner.len();
        if max_items == 0 {
            return;
        }

        let norm_seek_distance = if seek_distance < 0 {
            (max_items as isize) + seek_distance
        } else {
            seek_distance
        } as usize;

        #[allow(clippy::integer_division_remainder_used)]
        let pos = (self.pos.unwrap_or(max_items - 1) + max_items + norm_seek_distance) % max_items;
        self.pos = Some(pos);
    }

    /// Safely access currently pointed element from [`Self`]. Consider using
    /// this instead of directly dereferencing into inner [`Vec`]
    ///
    /// # Panics
    /// Calling this method will panic if the underlying vector has been altered
    /// and the position marker drops below the maximum length of the vector.
    pub fn get(&self) -> Option<&T> {
        if self.pos.is_none() {
            return None;
        }

        if self.pos.unwrap() >= self.inner.len() {
            // TODO: Consider changing to Result/bail!()
            panic!(
                "Undefined behavior: Underlying vec was modified. \
                Run cycle_next or cycle_prev to return to standard."
            );
        }

        // TODO: Move to use proper error system. Remove all unwrap calls
        self.inner.get(self.pos.unwrap())
    }
}

impl<T> Deref for CycleCursor<T> {
    type Target = Vec<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for CycleCursor<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeSet,
        panic::{self, UnwindSafe},
    };

    use super::*;

    fn assert_panic<F, R>(panic_fn: F)
    where
        F: FnOnce() -> R + UnwindSafe,
    {
        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(|_| {}));
        let result = panic::catch_unwind(panic_fn);
        assert!(result.is_err());
        panic::set_hook(panic_hook);
    }

    #[test]
    fn cursor_cycle_next() {
        let source = vec![1, 2, 3, 4];
        let mut cursor = CycleCursor::from(source);

        assert_eq!(cursor.pos, None);
        cursor.cycle_next();
        assert_eq!(cursor.get().unwrap(), &1);
        cursor.cycle_next();
        cursor.cycle_next();
        cursor.cycle_next();
        assert_eq!(cursor.get().unwrap(), &4);
        cursor.cycle_next();
        assert_eq!(cursor.get().unwrap(), &1);
    }

    #[test]
    fn cursor_cycle_next_empty_vec() {
        let source: Vec<usize> = vec![];
        let mut cursor = CycleCursor::from(source);

        assert_eq!(cursor.pos, None);
        cursor.cycle_next();
        assert_eq!(cursor.pos, None);
        assert_eq!(cursor.get(), None);
    }

    #[test]
    fn cursor_cycle_prev() {
        let source = BTreeSet::from([1, 2, 3, 4]);
        let mut cursor = CycleCursor::from(source);

        assert_eq!(cursor.pos, None);
        cursor.cycle_prev();
        assert_eq!(cursor.get().unwrap(), &4);
        cursor.cycle_prev();
        cursor.cycle_prev();
        cursor.cycle_prev();
        assert_eq!(cursor.get().unwrap(), &1);
        cursor.cycle_prev();
        assert_eq!(cursor.get().unwrap(), &4);
    }

    #[test]
    fn cursor_cycle_prev_empty_vec() {
        let source: Vec<usize> = vec![];
        let mut cursor = CycleCursor::from(source);

        assert_eq!(cursor.pos, None);
        cursor.cycle_prev();
        assert_eq!(cursor.pos, None);
        assert_eq!(cursor.get(), None);
    }

    #[test]
    fn cursor_peek() {
        let source = vec![1, 2, 3, 4];
        let mut cursor = CycleCursor::from(source);

        // Cursor on None
        assert_eq!(cursor.peek(3).unwrap(), &3);

        // Move to first element
        cursor.cycle_next();

        assert_eq!(cursor.peek(3).unwrap(), &4);

        cursor.cycle_next();
        cursor.cycle_next();

        assert_eq!(cursor.peek(3).unwrap(), &2);
        assert_eq!(cursor.peek(-3).unwrap(), &4);
    }

    #[test]
    fn cursor_peek_empty_vec() {
        let source: Vec<usize> = vec![];
        let cursor = CycleCursor::from(source);

        assert_eq!(cursor.pos, None);
        assert_eq!(cursor.peek(2), None);
        assert_eq!(cursor.get(), None);
    }

    #[test]
    fn cursor_seek() {
        let source = vec![1, 2, 3, 4];
        let mut cursor = CycleCursor::from(source);

        cursor.seek(6);
        assert_eq!(cursor.get().unwrap(), &2);

        cursor.seek(-3);
        assert_eq!(cursor.get().unwrap(), &3);
    }

    #[test]
    fn cursor_seek_empty_vec() {
        let source: Vec<usize> = vec![];
        let mut cursor = CycleCursor::from(source);

        assert_eq!(cursor.pos, None);
        cursor.seek(2);
        assert_eq!(cursor.pos, None);
        assert_eq!(cursor.get(), None);
    }

    #[test]
    fn cursor_get_undefined_behavior() {
        let source = vec![1, 2, 3, 4];
        let mut cursor = CycleCursor::from(source);

        cursor.seek(3);
        let _ = cursor.inner.remove(0);
        let _ = cursor.inner.remove(0);

        assert_panic(|| cursor.get());
    }
}
