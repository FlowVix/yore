pub mod scalar;
pub mod vec;

use std::marker::PhantomData;

use crate::{SavedStack, path::SeerPath, seer::scalar::Scalar, yorer::YoreStacks};

pub trait Seerious: Sized {
    type Seer<'a, P: SeerPath<Target = Self>>: Seer<Self>
    where
        Self: 'a,
        P: 'a;

    fn make_seer<'a>(&'a mut self, stacks: &'a mut YoreStacks) -> Self::Seer<'a, PhantomData<Self>>
    where
        Self: Sized + 'static;

    type CustomMutation;
    type CustomMutationSaved;
    fn apply_cmutation(
        &mut self,
        m: Self::CustomMutation,
        yore: &mut SavedStack,
    ) -> Self::CustomMutationSaved;
    fn unapply_cmutation(
        &mut self,
        saved: Self::CustomMutationSaved,
        yore: &mut SavedStack,
    ) -> Self::CustomMutation;
}
pub enum SeeriousMutation<T: Seerious> {
    Replace(T),
    Custom(T::CustomMutation),
}

pub trait Seer<T: Seerious> {
    fn set(&mut self, to: T);
}
