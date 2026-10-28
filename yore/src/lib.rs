#![deny(unused_must_use, unnameable_types)]
#![allow(clippy::too_many_arguments, clippy::single_match)]

mod next;
mod path;
mod seer;
mod yorer;

pub use next::Yorenext;
pub use path::{DynStoredPath, SeerPath, StoredPath, StoredPathEnd, StoredPathNext};
pub use seer::{
    Seer, Seerious, SeeriousMutation,
    scalar::{Scalar, ScalarSeer},
};
pub use yorer::{SavedStack, YoreStacks, Yorer};

pub enum Never {}

mod seal {
    #[allow(unnameable_types)]
    pub trait Sealed {}
}

// pub enum Never {}

// struct Foo {
//     a: i32,
//     b: String,
// }
// impl Yorenext<Foo> for FieldPath<0> {
//     type TargetType = i32;

//     fn get<'f>(&self, from: &'f mut Foo) -> &'f mut Self::TargetType {
//         &mut from.a
//     }
// }
// impl Yorenext<Foo> for FieldPath<1> {
//     type TargetType = String;

//     fn get<'f>(&self, from: &'f mut Foo) -> &'f mut Self::TargetType {
//         &mut from.b
//     }
// }

// struct Bar {
//     a: Foo,
//     b: String,
// }
// impl Yorenext<Bar> for FieldPath<0> {
//     type TargetType = Foo;

//     fn get<'f>(&self, from: &'f mut Bar) -> &'f mut Self::TargetType {
//         &mut from.a
//     }
// }
// impl Yorenext<Bar> for FieldPath<1> {
//     type TargetType = String;

//     fn get<'f>(&self, from: &'f mut Bar) -> &'f mut Self::TargetType {
//         &mut from.b
//     }
// }
