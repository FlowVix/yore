use std::{any::Any, marker::PhantomData, mem};

use crate::{
    Seerious, SeeriousMutation,
    next::Yorenext,
    yorer::{SavedStack, YoreStacks},
};

enum SavedMutation<T: Seerious> {
    Replace(T),
    Custom(T::CustomMutationSaved),
}

pub struct StoredPathEnd<T: Seerious>(Option<SeeriousMutation<T>>);
pub struct StoredPathNext<T, Next, P> {
    next: Next,
    then: P,
    _p: PhantomData<fn(T) -> T>,
}
impl<T: Seerious + 'static> StoredPathEnd<T> {
    fn apply_direct(&mut self, to: &mut T, yore: &mut SavedStack) {
        let saved = match self.0.take().unwrap() {
            SeeriousMutation::Replace(with) => {
                let prev = mem::replace(to, with);
                SavedMutation::Replace(prev)
            }
            SeeriousMutation::Custom(m) => {
                let saved = to.apply_cmutation(m, yore);
                SavedMutation::Custom(saved)
            }
        };
        yore.push_saved(saved);
    }
}

pub trait StoredPath: 'static {
    type From: Seerious;
    type Target: Seerious;

    fn apply(&mut self, to: &mut Self::From, yore: &mut SavedStack);
    fn unapply(&mut self, to: &mut Self::From, yore: &mut SavedStack);
    fn into_dyn(self) -> Box<dyn DynStoredPath>
    where
        Self: Sized + 'static,
    {
        Box::new((self, PhantomData))
    }
}
impl<T: Seerious + 'static> StoredPath for StoredPathEnd<T> {
    type From = T;
    type Target = T;

    fn apply(&mut self, to: &mut T, yore: &mut SavedStack) {
        self.apply_direct(to, yore);
    }
    fn unapply(&mut self, to: &mut T, yore: &mut SavedStack) {
        let saved: SavedMutation<T> = yore.pop_saved();
        let m = match saved {
            SavedMutation::Replace(saved) => {
                let saved = mem::replace(to, saved);
                SeeriousMutation::Replace(saved)
            }
            SavedMutation::Custom(saved) => {
                let m = to.unapply_cmutation(saved, yore);
                SeeriousMutation::Custom(m)
            }
        };
        self.0 = Some(m);
    }
}
impl<T: Seerious + 'static, Next: Yorenext<T> + 'static, P: StoredPath<From = Next::TargetType>>
    StoredPath for StoredPathNext<T, Next, P>
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

pub trait SeerPath: Clone {
    type From: Seerious;
    type Target: Seerious;
    type Completed: StoredPath<From = Self::From, Target = Self::Target>;
    fn complete_and_apply_direct(
        self,
        target: &mut Self::Target,
        m: SeeriousMutation<Self::Target>,
        yore: &mut SavedStack,
    ) -> Self::Completed;
}

impl<T: Seerious + 'static> SeerPath for PhantomData<T> {
    type From = T;
    type Target = T;
    type Completed = StoredPathEnd<T>;

    fn complete_and_apply_direct(
        self,
        target: &mut Self::Target,
        m: SeeriousMutation<Self::Target>,
        yore: &mut SavedStack,
    ) -> Self::Completed {
        let mut out = StoredPathEnd(Some(m));
        out.apply_direct(target, yore);
        out
    }
}
impl<
    T: Seerious + 'static,
    Next: Yorenext<T> + Clone + 'static,
    P: SeerPath<From = Next::TargetType>,
> SeerPath for (PhantomData<T>, Next, P)
{
    type From = T;
    type Target = P::Target;
    type Completed = StoredPathNext<T, Next, P::Completed>;

    fn complete_and_apply_direct(
        self,
        target: &mut Self::Target,
        m: SeeriousMutation<Self::Target>,
        yore: &mut SavedStack,
    ) -> Self::Completed {
        StoredPathNext {
            next: self.1,
            then: self.2.complete_and_apply_direct(target, m, yore),
            _p: PhantomData,
        }
    }
}
