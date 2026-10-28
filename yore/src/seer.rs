pub mod scalar;
pub mod vec;

use std::marker::PhantomData;

use crate::{SavedStack, Yorenext, path::SeerPath, seer::scalar::Scalar, yorer::YoreStacks};

pub trait Seerious: Sized + 'static {
    type Seer<'a, P: SeerPath<Target = Self>>: Seer<Self>;

    fn make_seer<'a, P: SeerPath<Target = Self> + 'a>(
        &'a mut self,
        stacks: &'a mut YoreStacks,
        path: P,
    ) -> Self::Seer<'a, P>;

    type CustomMutation;
    type CustomMutationUndo;
    fn apply_cmutation(
        &mut self,
        m: Self::CustomMutation,
        yore: &mut SavedStack,
    ) -> Self::CustomMutationUndo;
    fn unapply_cmutation(
        &mut self,
        m: Self::CustomMutationUndo,
        yore: &mut SavedStack,
    ) -> Self::CustomMutation;
}
pub enum SeeriousMutation<T: Seerious> {
    Replace(T),
    Custom(T::CustomMutation),
}

pub trait Seer<T: Seerious> {
    fn set(&mut self, to: T);
    fn child<
        'a,
        's: 'a,
        C: Seerious,
        P: SeerPath<Target = T>,
        Next: Yorenext<T, TargetType = C> + Clone,
    >(
        value: &'a mut C,
        next: Next,
        stacks: &'s mut YoreStacks,
        parent_path: &P,
    ) -> C::Seer<'a, P::Push<Next>> {
        C::make_seer(value, stacks, parent_path.push(next))
    }
}
