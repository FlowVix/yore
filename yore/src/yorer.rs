use std::{any::Any, marker::PhantomData, ops::Deref};

use crate::{
    SeeriousMutation,
    path::{DynStoredPath, SeerPath, StoredPath, StoredPathEnd},
    seer::Seerious,
};

pub struct SavedStack {
    stack: Vec<Box<dyn Any>>,
}
pub struct YoreStacks {
    saved: SavedStack,
    paths: Vec<Box<dyn DynStoredPath>>,
    applied_paths: usize,
}
impl SavedStack {
    pub(crate) fn push_saved<T: Any>(&mut self, v: T) {
        self.stack.push(Box::new(v));
    }
    pub(crate) fn pop_saved<T: Any>(&mut self) -> T {
        *self.stack.pop().unwrap().downcast().unwrap()
    }
}
impl YoreStacks {
    pub(crate) fn apply_mutation<T: Seerious + 'static, P: SeerPath<Target = T>>(
        &mut self,
        target: &mut T,
        m: SeeriousMutation<T>,
        path: P,
    ) {
        self.paths.truncate(self.applied_paths);
        let path = path.complete_and_apply_direct(target, m, &mut self.saved);
        self.paths.push(path.into_dyn());
        self.applied_paths += 1;
    }
}

pub struct Yorer<T> {
    pub(crate) value: T,

    pub(crate) stacks: YoreStacks,
}

impl<T: Seerious + 'static> Yorer<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            stacks: YoreStacks {
                saved: SavedStack { stack: vec![] },
                paths: vec![],
                applied_paths: 0,
            },
        }
    }
    pub fn seer(&mut self) -> T::Seer<'_, PhantomData<T>> {
        T::make_seer(&mut self.value, &mut self.stacks)
    }
    pub fn undo(&mut self) {
        let Some(last) = self.stacks.paths.get_mut(self.stacks.applied_paths - 1) else {
            return;
        };
        last.dyn_unapply(&mut self.value, &mut self.stacks.saved);
        self.stacks.applied_paths -= 1;
    }
    pub fn redo(&mut self) {
        let Some(next) = self.stacks.paths.get_mut(self.stacks.applied_paths) else {
            return;
        };
        next.dyn_apply(&mut self.value, &mut self.stacks.saved);
        self.stacks.applied_paths += 1;
        self.stacks.paths.truncate(self.stacks.applied_paths);
    }
}

impl<T> Deref for Yorer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
