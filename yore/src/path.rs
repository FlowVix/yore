use std::{any::Any, marker::PhantomData, mem};

use crate::{
    Seerious, SeeriousMutation,
    next::Yorenext,
    yorer::{SavedStack, YoreStacks},
};

enum MutationUndo<T: Seerious> {
    Replace(T),
    Custom(T::CustomMutationUndo),
}

pub struct StoredPathEnd<T: Seerious>(Option<SeeriousMutation<T>>);
pub struct StoredPathNext<T, Next, P> {
    next: Next,
    then: P,
    _p: PhantomData<fn(T) -> T>,
}
impl<T: Seerious> StoredPathEnd<T> {
    fn apply_direct(&mut self, to: &mut T, yore: &mut SavedStack) {
        let undo = match self.0.take().unwrap() {
            SeeriousMutation::Replace(with) => {
                let prev = mem::replace(to, with);
                MutationUndo::Replace(prev)
            }
            SeeriousMutation::Custom(m) => {
                let undo = to.apply_cmutation(m, yore);
                MutationUndo::Custom(undo)
            }
        };
        yore.push(undo);
    }
}

pub trait StoredPath: 'static {
    type From: Seerious;
    type Target: Seerious;

    fn apply(&mut self, to: &mut Self::From, yore: &mut SavedStack);
    fn unapply(&mut self, to: &mut Self::From, yore: &mut SavedStack);
    fn into_dyn(self) -> Box<dyn DynStoredPath>
    where
        Self: Sized,
    {
        Box::new((self, PhantomData))
    }
}
impl<T: Seerious> StoredPath for StoredPathEnd<T> {
    type From = T;
    type Target = T;

    fn apply(&mut self, to: &mut T, yore: &mut SavedStack) {
        self.apply_direct(to, yore);
    }
    fn unapply(&mut self, to: &mut T, yore: &mut SavedStack) {
        let undo: MutationUndo<T> = yore.pop();
        let m = match undo {
            MutationUndo::Replace(undo) => {
                let saved = mem::replace(to, undo);
                SeeriousMutation::Replace(saved)
            }
            MutationUndo::Custom(undo) => {
                let m = to.unapply_cmutation(undo, yore);
                SeeriousMutation::Custom(m)
            }
        };
        self.0 = Some(m);
    }
}
impl<T: Seerious, Next: Yorenext<T>, P: StoredPath<From = Next::TargetType>> StoredPath
    for StoredPathNext<T, Next, P>
{
    type From = T;
    type Target = P::Target;

    fn apply(&mut self, to: &mut T, yore: &mut SavedStack) {
        self.then.apply(self.next.get(to), yore);
    }

    fn unapply(&mut self, to: &mut T, yore: &mut SavedStack) {
        self.then.unapply(self.next.get(to), yore);
    }
}

pub trait DynStoredPath {
    fn dyn_apply(&mut self, to: &mut dyn Any, yore: &mut SavedStack);
    fn dyn_unapply(&mut self, to: &mut dyn Any, yore: &mut SavedStack);
}
impl<T: Seerious + 'static, P: StoredPath<From = T>> DynStoredPath for (P, PhantomData<T>) {
    fn dyn_apply(&mut self, to: &mut dyn Any, yore: &mut SavedStack) {
        let to = to.downcast_mut::<T>().unwrap();
        self.0.apply(to, yore);
    }

    fn dyn_unapply(&mut self, to: &mut dyn Any, yore: &mut SavedStack) {
        let to = to.downcast_mut::<T>().unwrap();
        self.0.unapply(to, yore);
    }
}

pub struct SeerPathEnd<T: Seerious>(pub(crate) PhantomData<T>);

pub struct SeerPathNext<T, Next, P> {
    next: Next,
    then: P,
    _p: PhantomData<fn(T) -> T>,
}
impl<T: Seerious> Clone for SeerPathEnd<T> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}
impl<T, Next: Clone, P: Clone> Clone for SeerPathNext<T, Next, P> {
    fn clone(&self) -> Self {
        Self {
            next: self.next.clone(),
            then: self.then.clone(),
            _p: PhantomData,
        }
    }
}

pub trait SeerPath: Clone + 'static {
    type From: Seerious;
    type Target: Seerious;
    type Completed: StoredPath<From = Self::From, Target = Self::Target>;
    fn complete_and_apply_direct(
        &self,
        target: &mut Self::Target,
        m: SeeriousMutation<Self::Target>,
        yore: &mut SavedStack,
    ) -> Self::Completed;
    type Push<Next: Yorenext<Self::Target>>: SeerPath<From = Self::From, Target = Next::TargetType>;
    fn push<Next: Yorenext<Self::Target>>(&self, next: Next) -> Self::Push<Next>;
}

impl<T: Seerious> SeerPath for SeerPathEnd<T> {
    type From = T;
    type Target = T;
    type Completed = StoredPathEnd<T>;

    fn complete_and_apply_direct(
        &self,
        target: &mut Self::Target,
        m: SeeriousMutation<Self::Target>,
        yore: &mut SavedStack,
    ) -> Self::Completed {
        let mut out = StoredPathEnd(Some(m));
        out.apply_direct(target, yore);
        out
    }

    type Push<Next: Yorenext<T>> = SeerPathNext<T, Next, SeerPathEnd<Next::TargetType>>;
    fn push<Next: Yorenext<Self::Target>>(&self, next: Next) -> Self::Push<Next> {
        SeerPathNext {
            next,
            then: SeerPathEnd(PhantomData),
            _p: PhantomData,
        }
    }
}
impl<T: Seerious, Next: Yorenext<T>, P: SeerPath<From = Next::TargetType>> SeerPath
    for SeerPathNext<T, Next, P>
{
    type From = T;
    type Target = P::Target;
    type Completed = StoredPathNext<T, Next, P::Completed>;

    fn complete_and_apply_direct(
        &self,
        target: &mut Self::Target,
        m: SeeriousMutation<Self::Target>,
        yore: &mut SavedStack,
    ) -> Self::Completed {
        StoredPathNext {
            next: self.next.clone(),
            then: self.then.complete_and_apply_direct(target, m, yore),
            _p: PhantomData,
        }
    }

    type Push<Next2: Yorenext<Self::Target>> = SeerPathNext<T, Next, P::Push<Next2>>;
    fn push<Next2: Yorenext<Self::Target>>(&self, next: Next2) -> Self::Push<Next2> {
        SeerPathNext {
            next: self.next.clone(),
            then: self.then.push(next),
            _p: PhantomData,
        }
    }
}
