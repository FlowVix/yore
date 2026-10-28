use crate::{
    SavedStack, SeeriousMutation,
    path::{SeerPath, StoredPathEnd},
    seer::{Seer, Seerious},
    yorer::YoreStacks,
};
use std::{
    cmp::Ordering,
    convert::Infallible,
    fmt,
    marker::PhantomData,
    mem,
    ops::{Bound, Deref, DerefMut, Index, IndexMut, Range, RangeBounds},
    slice::SliceIndex,
};

// pub enum VecChange<'a, T: Seerious> {
//     Replace { values: Vec<A> },
//     InsertAt { index: usize, value: A },
//     RemoveAt { index: usize },
//     Move { old_index: usize, new_index: usize },
//     Push { value: A },
//     Pop,
//     Clear,
// }

pub enum VecMutation<T> {
    Nop,
    Push { value: T },
    Pop,
    Insert { index: usize, value: T },
    Remove { index: usize },
    SwapRemove { index: usize },
    Truncate { len: usize },
    Append { other: Vec<T> },
    Swap { a: usize, b: usize },
    Reverse,
    RotLeft { mid: usize },
    RotRight { k: usize },
    Reorder { perm: Vec<usize> },
}
pub enum VecMutationUndo<T> {
    Nop,
    Push,
    Pop,
    Insert { index: usize },
    Remove { index: usize },
    SwapInsert { index: usize },
    Truncate { len: usize },
    Append { other: Vec<T> },
    Swap { a: usize, b: usize },
    Reverse,
    RotLeft { mid: usize },
    RotRight { k: usize },
    InvReorder { perm: Vec<usize> },
}

pub struct VecSeer<'a, T, P> {
    value: &'a mut Vec<T>,
    stacks: &'a mut YoreStacks,
    path: P,
}
impl<T: Seerious> Seerious for Vec<T> {
    type Seer<'a, P: SeerPath<Target = Self>> = VecSeer<'a, T, P>;

    fn make_seer<'a, P: SeerPath<Target = Self> + 'a>(
        &'a mut self,
        stacks: &'a mut YoreStacks,
        path: P,
    ) -> Self::Seer<'a, P> {
        VecSeer {
            stacks,
            value: self,
            path,
        }
    }

    type CustomMutation = VecMutation<T>;

    type CustomMutationUndo = VecMutationUndo<T>;

    fn apply_cmutation(
        &mut self,
        m: Self::CustomMutation,
        yore: &mut SavedStack,
    ) -> Self::CustomMutationUndo {
        match m {
            VecMutation::Nop => VecMutationUndo::Nop,
            VecMutation::Push { value } => {
                self.push(value);
                VecMutationUndo::Pop
            }
            VecMutation::Pop => match self.pop() {
                Some(v) => {
                    yore.push(v);
                    VecMutationUndo::Push
                }
                None => VecMutationUndo::Nop,
            },
            VecMutation::Insert { index, value } => {
                self.insert(index, value);
                VecMutationUndo::Remove { index }
            }
            VecMutation::Remove { index } => {
                yore.push(self.remove(index));
                VecMutationUndo::Insert { index }
            }
            VecMutation::SwapRemove { index } => {
                yore.push(self.swap_remove(index));
                VecMutationUndo::SwapInsert { index }
            }
            VecMutation::Truncate { len } => VecMutationUndo::Append {
                other: self.split_off(len),
            },
            VecMutation::Append { mut other } => {
                let len = self.len();
                self.append(&mut other);
                VecMutationUndo::Truncate { len }
            }
            VecMutation::Swap { a, b } => {
                self.swap(a, b);
                VecMutationUndo::Swap { a, b }
            }
            VecMutation::Reverse => {
                self.reverse();
                VecMutationUndo::Reverse
            }
            VecMutation::RotLeft { mid } => {
                self.rotate_left(mid);
                VecMutationUndo::RotRight { k: mid }
            }
            VecMutation::RotRight { k } => {
                self.rotate_right(k);
                VecMutationUndo::RotLeft { mid: k }
            }
            VecMutation::Reorder { perm } => {
                apply_permutation(self, &perm);
                VecMutationUndo::InvReorder { perm }
            }
        }
    }

    fn unapply_cmutation(
        &mut self,
        m: Self::CustomMutationUndo,
        yore: &mut SavedStack,
    ) -> Self::CustomMutation {
        match m {
            VecMutationUndo::Push => {
                self.push(yore.pop());
                VecMutation::Pop
            }
            VecMutationUndo::Pop => VecMutation::Push {
                value: self.pop().unwrap(),
            },
            VecMutationUndo::Insert { index } => {
                self.insert(index, yore.pop());
                VecMutation::Remove { index }
            }
            VecMutationUndo::Remove { index } => VecMutation::Insert {
                index,
                value: self.remove(index),
            },
            VecMutationUndo::SwapInsert { index } => {
                let last = self.len();
                self.push(yore.pop());
                self.swap(index, last);
                VecMutation::SwapRemove { index }
            }
            VecMutationUndo::Truncate { len } => VecMutation::Append {
                other: self.split_off(len),
            },
            VecMutationUndo::Append { mut other } => {
                let len = self.len();
                self.append(&mut other);
                VecMutation::Truncate { len }
            }
            VecMutationUndo::Swap { a, b } => {
                self.swap(a, b);
                VecMutation::Swap { a, b }
            }
            VecMutationUndo::Reverse => {
                self.reverse();
                VecMutation::Reverse
            }
            VecMutationUndo::RotLeft { mid } => {
                self.rotate_left(mid);
                VecMutation::RotRight { k: mid }
            }
            VecMutationUndo::RotRight { k } => {
                self.rotate_right(k);
                VecMutation::RotLeft { mid: k }
            }
            VecMutationUndo::InvReorder { perm } => {
                apply_inverse_permutation(self, &perm);
                VecMutation::Reorder { perm }
            }
            VecMutationUndo::Nop => VecMutation::Nop,
        }
    }
}
impl<'a, T: Seerious, P: SeerPath<Target = Vec<T>>> Seer<Vec<T>> for VecSeer<'a, T, P> {
    fn set(&mut self, to: Vec<T>) {
        self.stacks
            .apply_mutation(self.value, SeeriousMutation::Replace(to), &self.path);
    }
}

type ElemSeer<'a, T: Seerious, P: SeerPath> = T::Seer<'a, P::Push<usize>>;

impl<'a, T: Seerious, P: SeerPath<Target = Vec<T>>> VecSeer<'a, T, P> {
    pub fn reserve(&mut self, additional: usize) {
        self.value.reserve(additional)
    }

    pub fn reserve_exact(&mut self, additional: usize) {
        self.value.reserve_exact(additional)
    }

    pub fn shrink_to_fit(&mut self) {
        self.value.shrink_to_fit()
    }

    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.value.shrink_to(min_capacity)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<ElemSeer<'_, T, P>> {
        self.value
            .get_mut(index)
            .map(|v| Self::child(v, index, self.stacks, &self.path))
    }

    /// Like [`get_mut`](Self::get_mut), but panics if `index` is out of bounds,
    /// as indexing does. (`IndexMut` can't be implemented: it would have to
    /// return a reference to an element seer, which only exists on demand.)
    #[track_caller]
    pub fn at(&mut self, index: usize) -> ElemSeer<'_, T, P> {
        let len = self.value.len();
        let Some(value) = self.value.get_mut(index) else {
            panic!("index out of bounds: the len is {len} but the index is {index}");
        };
        Self::child(value, index, self.stacks, &self.path)
    }

    pub fn first_mut(&mut self) -> Option<ElemSeer<'_, T, P>> {
        self.value
            .first_mut()
            .map(|v| Self::child(v, 0, self.stacks, &self.path))
    }

    pub fn last_mut(&mut self) -> Option<ElemSeer<'_, T, P>> {
        let idx = self.value.len().checked_sub(1)?;
        self.value
            .last_mut()
            .map(|v| Self::child(v, idx, self.stacks, &self.path))
    }

    #[inline]
    fn emit(&mut self, change: VecMutation<T>) {
        self.stacks
            .apply_mutation(self.value, SeeriousMutation::Custom(change), &self.path);
    }

    /// `perm[k]` is the current index of the element that should end up at `k`.
    fn reorder(&mut self, perm: Vec<usize>) {
        self.emit(VecMutation::Reorder { perm });
    }

    pub fn push(&mut self, value: T) {
        self.emit(VecMutation::Push { value });
    }

    pub fn insert(&mut self, index: usize, value: T) {
        self.emit(VecMutation::Insert { index, value });
    }

    pub fn append(&mut self, other: &mut Vec<T>) {
        self.emit(VecMutation::Append {
            other: mem::take(other),
        });
    }

    // pub fn extend_from_slice(&mut self, other: &[T])
    // where
    //     T: Clone,
    // {
    //     self.extend(other.iter().cloned());
    // }

    // pub fn extend_from_within<R: RangeBounds<usize>>(&mut self, src: R)
    // where
    //     T: Clone,
    // {
    //     let start = self.value.len();
    //     self.value.extend_from_within(src);
    //     self.emit_adds(start..self.value.len());
    // }

    // pub fn resize(&mut self, new_len: usize, value: T)
    // where
    //     T: Clone,
    // {
    //     let len = self.value.len();
    //     if new_len <= len {
    //         self.truncate(new_len);
    //         return;
    //     }
    //     self.value.reserve(new_len - len);
    //     for _ in len + 1..new_len {
    //         self.push(value.clone());
    //     }
    //     self.push(value);
    // }

    // pub fn resize_with<F: FnMut() -> T>(&mut self, new_len: usize, mut f: F) {
    //     let len = self.value.len();
    //     if new_len <= len {
    //         self.truncate(new_len);
    //         return;
    //     }
    //     self.value.reserve(new_len - len);
    //     for _ in len..new_len {
    //         self.push(f());
    //     }
    // }

    // ---------------------------------------------------------------------------
    // Removing
    // ---------------------------------------------------------------------------

    pub fn pop(&mut self) -> Option<&T> {
        let has = self.value.is_empty();
        self.emit(VecMutation::Pop);
        has.then(|| self.stacks.saved().top())
    }

    pub fn remove(&mut self, index: usize) -> &T {
        self.emit(VecMutation::Remove { index });
        self.stacks.saved().top()
    }

    pub fn swap_remove(&mut self, index: usize) -> &T {
        self.emit(VecMutation::SwapRemove { index });
        self.stacks.saved().top()
    }

    pub fn truncate(&mut self, len: usize) {
        self.emit(VecMutation::Truncate { len });
    }

    pub fn clear(&mut self) {
        self.truncate(0);
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        self.emit(VecMutation::Swap { a, b });
    }

    pub fn reverse(&mut self) {
        self.emit(VecMutation::Reverse);
    }

    pub fn rotate_left(&mut self, mid: usize) {
        self.emit(VecMutation::RotLeft { mid });
    }

    pub fn rotate_right(&mut self, k: usize) {
        self.emit(VecMutation::RotRight { k });
    }

    pub fn sort(&mut self)
    where
        T: Ord,
    {
        self.sort_by(T::cmp);
    }

    pub fn sort_by<F: FnMut(&T, &T) -> Ordering>(&mut self, mut compare: F) {
        if self.value.len() < 2 {
            return;
        }
        let mut perm: Vec<usize> = (0..self.value.len()).collect();
        perm.sort_by(|&a, &b| compare(&self.value[a], &self.value[b]));
        self.reorder(perm);
    }

    pub fn sort_by_key<K: Ord, F: FnMut(&T) -> K>(&mut self, mut f: F) {
        if self.value.len() < 2 {
            return;
        }
        let mut perm: Vec<usize> = (0..self.value.len()).collect();
        perm.sort_by_key(|&i| f(&self.value[i]));
        self.reorder(perm);
    }

    pub fn sort_by_cached_key<K: Ord, F: FnMut(&T) -> K>(&mut self, mut f: F) {
        if self.value.len() < 2 {
            return;
        }
        let mut perm: Vec<usize> = (0..self.value.len()).collect();
        perm.sort_by_cached_key(|&i| f(&self.value[i]));
        self.reorder(perm);
    }

    pub fn sort_unstable(&mut self)
    where
        T: Ord,
    {
        self.sort_unstable_by(T::cmp);
    }

    pub fn sort_unstable_by<F: FnMut(&T, &T) -> Ordering>(&mut self, mut compare: F) {
        if self.value.len() < 2 {
            return;
        }
        let mut perm: Vec<usize> = (0..self.value.len()).collect();
        perm.sort_unstable_by(|&a, &b| compare(&self.value[a], &self.value[b]).then(a.cmp(&b)));
        self.reorder(perm);
    }

    pub fn sort_unstable_by_key<K: Ord, F: FnMut(&T) -> K>(&mut self, mut f: F) {
        if self.value.len() < 2 {
            return;
        }
        let mut perm: Vec<usize> = (0..self.value.len()).collect();
        perm.sort_unstable_by(|&a, &b| f(&self.value[a]).cmp(&f(&self.value[b])).then(a.cmp(&b)));
        self.reorder(perm);
    }
}

impl<'a, T: Seerious, P: SeerPath<Target = Vec<T>>> Deref for VecSeer<'a, T, P> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        self.value
    }
}

impl<'a, T: Seerious, P: SeerPath<Target = Vec<T>>> AsRef<[T]> for VecSeer<'a, T, P> {
    fn as_ref(&self) -> &[T] {
        self.value
    }
}

impl<'a, T: Seerious, P: SeerPath<Target = Vec<T>>> Extend<T> for VecSeer<'a, T, P> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.emit(VecMutation::Append {
            other: iter.into_iter().collect(),
        });
    }
}

impl<'a, T: Seerious + Copy + 'a, P: SeerPath<Target = Vec<T>>> Extend<&'a T>
    for VecSeer<'a, T, P>
{
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.extend(iter.into_iter().copied());
    }
}

impl<'a, T: Seerious, P: SeerPath<Target = Vec<T>>> IntoIterator for &'a VecSeer<'a, T, P> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.value.iter()
    }
}

impl<'a, T: Seerious + fmt::Debug, P: SeerPath<Target = Vec<T>>> fmt::Debug
    for &'a VecSeer<'a, T, P>
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.value, f)
    }
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

/// Resolves a range against `len`, panicking like slice indexing does.
fn resolve_range<R: RangeBounds<usize>>(range: R, len: usize) -> Range<usize> {
    let start = match range.start_bound() {
        Bound::Included(&s) => s,
        Bound::Excluded(&s) => s
            .checked_add(1)
            .expect("attempted to index slice from after maximum usize"),
        Bound::Unbounded => 0,
    };
    let end = match range.end_bound() {
        Bound::Included(&e) => e
            .checked_add(1)
            .expect("attempted to index slice up to maximum usize"),
        Bound::Excluded(&e) => e,
        Bound::Unbounded => len,
    };
    assert!(
        start <= end,
        "slice index starts at {start} but ends at {end}"
    );
    assert!(
        end <= len,
        "range end index {end} out of range for slice of length {len}"
    );
    start..end
}

/// Rearranges `vec` so that `new[k] = old[perm[k]]`, in place, by following cycles.
fn apply_permutation<T>(vec: &mut [T], perm: &[usize]) {
    let mut done = vec![false; perm.len()];
    for start in 0..perm.len() {
        if done[start] {
            continue;
        }
        done[start] = true;
        let mut j = start;
        loop {
            let k = perm[j];
            if k == start {
                break;
            }
            vec.swap(j, k);
            done[k] = true;
            j = k;
        }
    }
}
/// Rearranges `vec` so that `new[perm[k]] = old[k]`, i.e. undoes `apply_permutation(vec, perm)`.
fn apply_inverse_permutation<T>(vec: &mut [T], perm: &[usize]) {
    let mut done = vec![false; perm.len()];
    for start in 0..perm.len() {
        if done[start] {
            continue;
        }
        done[start] = true;
        let mut j = start;
        loop {
            let k = perm[j];
            if k == start {
                break;
            }
            vec.swap(start, k);
            done[k] = true;
            j = k;
        }
    }
}
