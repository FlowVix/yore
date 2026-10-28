use std::collections::HashMap;
use std::hash::Hash;

use crate::Seerious;

pub trait Yorenext<From>: Clone + 'static {
    type TargetType: Seerious;

    fn get<'f>(&self, from: &'f mut From) -> &'f mut Self::TargetType;
}

pub enum FieldPath<const F: u32> {}

impl<T: Seerious> Yorenext<Vec<T>> for usize {
    type TargetType = T;

    fn get<'f>(&self, from: &'f mut Vec<T>) -> &'f mut Self::TargetType {
        &mut from[*self]
    }
}

impl<K: Hash + Eq + Clone + 'static, V: Seerious> Yorenext<HashMap<K, V>> for K {
    type TargetType = V;

    fn get<'f>(&self, from: &'f mut HashMap<K, V>) -> &'f mut Self::TargetType {
        from.get_mut(self).unwrap()
    }
}
