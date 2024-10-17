use std::collections::HashMap;

/// Maps the version number for each deserialization type name
pub trait VersionMap: Clone + Sync + for<'a> VersionMapIter<'a> {
    fn get(&self, type_id: &str) -> Option<usize>;
}
/// Has an iter method
pub trait VersionMapIter<'a> {
    type Iter: Iterator<Item = (&'a str, usize)>;
    fn iter(&'a self) -> Self::Iter;
}
pub type DefaultVersionMap<'a> = HashMap<&'a str, usize>;

mod version_map_impls {
    use crate::version_map::VersionMapIter;
    use crate::VersionMap;
    use std::borrow::Borrow;
    use std::collections::HashMap;
    use std::hash::{BuildHasher, Hash};

    impl<T: Borrow<str> + Hash + Eq + Sync + Clone + 'static, S: BuildHasher + Sync + Clone>
        VersionMap for HashMap<T, usize, S>
    {
        fn get(&self, type_id: &str) -> Option<usize> {
            std::collections::HashMap::get(self, type_id).cloned()
        }
    }

    type Iter<'i, T> = std::iter::Map<
        std::collections::hash_map::Iter<'i, T, usize>,
        fn((&'i T, &'i usize)) -> (&'i str, usize),
    >;
    impl<'i, T: Borrow<str> + Hash + Eq + 'i, S: BuildHasher + Sync> VersionMapIter<'i>
        for HashMap<T, usize, S>
    {
        type Iter = Iter<'i, T>;

        fn iter(&'i self) -> Self::Iter {
            HashMap::<T, usize, S>::iter(self).map(|(k, v)| (k.borrow(), *v))
        }
    }

    impl<'a, T: VersionMap> VersionMap for &'a T
    where
        &'a T: Clone,
    {
        fn get(&self, type_id: &str) -> Option<usize> {
            <T as VersionMap>::get(self, type_id)
        }
    }
    impl<'a, 'i, T: VersionMapIter<'i>> VersionMapIter<'i> for &'a T {
        type Iter = <T as VersionMapIter<'i>>::Iter;

        fn iter(&'i self) -> Self::Iter {
            <T as VersionMapIter<'i>>::iter(self)
        }
    }

    impl<'a, T: VersionMap> VersionMap for &'a mut T
    where
        &'a mut T: Clone,
    {
        fn get(&self, type_id: &str) -> Option<usize> {
            <T as VersionMap>::get(self, type_id)
        }
    }
    impl<'a, 'i, T: VersionMapIter<'i>> VersionMapIter<'i> for &'a mut T {
        type Iter = <T as VersionMapIter<'i>>::Iter;

        fn iter(&'i self) -> Self::Iter {
            <T as VersionMapIter<'i>>::iter(self)
        }
    }
}
