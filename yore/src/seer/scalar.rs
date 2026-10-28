use std::{convert::Infallible, marker::PhantomData, ops::Deref};

use crate::{
    SavedStack, SeeriousMutation,
    path::{SeerPath, StoredPathEnd},
    seer::{Seer, Seerious},
    yorer::YoreStacks,
};

pub trait Scalar: Clone + 'static {}
impl<S: Scalar> Seerious for S {
    type Seer<'a, P: SeerPath<Target = Self>> = ScalarSeer<'a, S, P>;

    fn make_seer<'a, P: SeerPath<Target = Self> + 'a>(
        &'a mut self,
        stacks: &'a mut YoreStacks,
        path: P,
    ) -> Self::Seer<'a, P> {
        ScalarSeer {
            stacks,
            value: self,
            path,
        }
    }

    type CustomMutation = Infallible;
    type CustomMutationUndo = Infallible;

    fn apply_cmutation(
        &mut self,
        _m: Self::CustomMutation,
        _yore: &mut SavedStack,
    ) -> Self::CustomMutationUndo {
        unreachable!()
    }

    fn unapply_cmutation(
        &mut self,
        _m: Self::CustomMutationUndo,
        _yore: &mut SavedStack,
    ) -> Self::CustomMutation {
        unreachable!()
    }
}
pub struct ScalarSeer<'a, T, P> {
    value: &'a mut T,
    stacks: &'a mut YoreStacks,
    path: P,
}
impl<'a, T: Scalar, P: SeerPath<Target = T>> Seer<T> for ScalarSeer<'a, T, P> {
    fn set(&mut self, to: T) {
        self.stacks
            .apply_mutation(self.value, SeeriousMutation::Replace(to), &self.path);
    }
}
impl<'a, T: Scalar, P: SeerPath<Target = T>> ScalarSeer<'a, T, P> {
    // pub fn m<'s>(&'s mut self) -> ScalarMut<'a, 's, T, Ctx, Act> {
    //     ScalarMut {
    //         from: self.cast_whatever(),
    //         old: None,
    //     }
    // }
}
impl<'a, T: Scalar, P> Deref for ScalarSeer<'a, T, P> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}

macro_rules! impl_scalar {
    ($($t:ty),* $(,)?) => {
        $(
            impl Scalar for $t {}
        )*
    };
}
macro_rules! impl_scalar_num {
    ($($t:ty),* $(,)?) => {
        $(
            impl Scalar for $t {}
            impl Scalar for ::core::num::NonZero<$t> {}
            impl Scalar for ::core::num::Wrapping<$t> {}
            impl Scalar for ::core::num::Saturating<$t> {}
        )*
    };
}

impl_scalar_num!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize,
);

impl_scalar!(
    bool,
    char,
    f32,
    f64,
    String,
    ::core::convert::Infallible,
    ::core::num::FpCategory,
    ::core::num::ParseIntError,
    ::core::num::ParseFloatError,
    ::core::num::TryFromIntError,
    ::core::num::IntErrorKind,
    ::core::time::Duration,
    ::core::time::TryFromFloatSecsError,
    ::std::time::Instant,
    ::std::time::SystemTime,
    ::core::net::Ipv4Addr,
    ::core::net::Ipv6Addr,
    ::core::net::SocketAddrV4,
    ::core::net::SocketAddrV6,
    ::core::net::IpAddr,
    ::core::net::SocketAddr,
    ::std::net::Shutdown,
    ::core::cmp::Ordering,
    ::core::sync::atomic::Ordering,
    ::std::io::ErrorKind,
    ::std::io::SeekFrom,
    ::core::fmt::Alignment,
    ::core::any::TypeId,
    ::core::alloc::Layout,
    ::core::alloc::LayoutError,
    ::std::thread::ThreadId,
    ::core::marker::PhantomPinned,
    ::core::fmt::Error,
);
