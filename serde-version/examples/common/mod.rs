use serde_version::{DeserializeVersionedSeed, VersionMap, VersionedDeserializer};
use std::fmt::Debug;
use std::marker::PhantomData;

pub fn deserialize_test<'de, T, VM>(input: &'de str, v: T, version_map: VM)
where
    PhantomData<T>: DeserializeVersionedSeed<'de, Value = T>,
    T: PartialEq + Debug,
    VM: VersionMap,
{
    let mut ron_deserializer = ron::de::Deserializer::from_str(input).unwrap();
    let deserializer = VersionedDeserializer::new(&mut ron_deserializer, version_map.clone());
    let de =
        PhantomData::<T>::deserialize_versioned(PhantomData, deserializer, version_map).unwrap();

    assert_eq!(v, de);
}
