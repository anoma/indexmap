#![cfg_attr(docsrs, doc(cfg(feature = "borsh")))]

use core::hash::BuildHasher;
use core::hash::Hash;
use core::iter::ExactSizeIterator;
use core::mem::size_of;

use borsh::error::ERROR_ZST_FORBIDDEN;
use borsh::io::{Error, ErrorKind, Read, Result, Write};
use borsh::{BorshDeserialize, BorshSerialize};

use crate::map::IndexMap;
use crate::set::IndexSet;

impl<K, V, H> BorshSerialize for IndexMap<K, V, H>
where
    K: BorshSerialize,
    V: BorshSerialize,
    H: BuildHasher,
{
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        check_zst::<K>()?;

        let iterator = self.iter();

        u32::try_from(iterator.len())
            .map_err(|_| ErrorKind::InvalidData)?
            .serialize(writer)?;

        for (key, value) in iterator {
            key.serialize(writer)?;
            value.serialize(writer)?;
        }

        Ok(())
    }
}

impl<K, V, H> BorshDeserialize for IndexMap<K, V, H>
where
    K: BorshDeserialize + Eq + Hash,
    V: BorshDeserialize,
    H: BuildHasher + Default,
{
    #[inline]
    fn deserialize_reader<R: Read>(reader: &mut R) -> Result<Self> {
        check_zst::<K>()?;

        let elems = u32::deserialize_reader(reader)?;
        let mut map = Self::with_capacity_and_hasher(elems as _, Default::default());

        for _ in 0..elems {
            let (key, value) = <(K, V)>::deserialize_reader(reader)?;
            map.insert(key, value);
        }

        Ok(map)
    }
}

impl<T, H> BorshSerialize for IndexSet<T, H>
where
    T: BorshSerialize,
    H: BuildHasher,
{
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        check_zst::<T>()?;

        let iterator = self.iter();

        u32::try_from(iterator.len())
            .map_err(|_| ErrorKind::InvalidData)?
            .serialize(writer)?;

        for item in iterator {
            item.serialize(writer)?;
        }

        Ok(())
    }
}

impl<T, H> BorshDeserialize for IndexSet<T, H>
where
    T: BorshDeserialize + Eq + Hash,
    H: BuildHasher + Default,
{
    #[inline]
    fn deserialize_reader<R: Read>(reader: &mut R) -> Result<Self> {
        check_zst::<T>()?;

        let elems = u32::deserialize_reader(reader)?;
        let mut set = Self::with_capacity_and_hasher(elems as _, Default::default());

        for _ in 0..elems {
            let member = T::deserialize_reader(reader)?;
            set.insert(member);
        }

        Ok(set)
    }
}

fn check_zst<T>() -> Result<()> {
    if size_of::<T>() == 0 {
        return Err(Error::new(ErrorKind::InvalidData, ERROR_ZST_FORBIDDEN));
    }
    Ok(())
}

#[cfg(test)]
mod borsh_tests {
    use super::*;

    #[test]
    fn map_borsh_roundtrip() {
        let original_map: IndexMap<i32, i32> = {
            let mut map = IndexMap::new();
            map.insert(1, 2);
            map.insert(3, 4);
            map.insert(5, 6);
            map
        };
        let serialized_map = borsh::to_vec(&original_map).unwrap();
        let deserialized_map: IndexMap<i32, i32> =
            BorshDeserialize::try_from_slice(&serialized_map).unwrap();
        assert_eq!(original_map, deserialized_map);
    }

    #[test]
    fn set_borsh_roundtrip() {
        let original_map: IndexSet<i32> = [1, 2, 3, 4, 5, 6].into_iter().collect();
        let serialized_map = borsh::to_vec(&original_map).unwrap();
        let deserialized_map: IndexSet<i32> =
            BorshDeserialize::try_from_slice(&serialized_map).unwrap();
        assert_eq!(original_map, deserialized_map);
    }
}
