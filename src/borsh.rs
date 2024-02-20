#![cfg_attr(docsrs, doc(cfg(feature = "borsh")))]

use alloc::vec::Vec;
use core::hash::BuildHasher;
use core::hash::Hash;
use core::mem::size_of;

use borsh::error::ERROR_ZST_FORBIDDEN;
use borsh::io::{Error, ErrorKind, Read, Result, Write};
use borsh::{BorshDeserialize, BorshSerialize};
#[cfg(feature = "borsh-schema")]
use ::{
    alloc::collections::btree_map::BTreeMap,
    alloc::format,
    borsh::schema::{add_definition, Declaration, Definition},
    borsh::BorshSchema,
};

use crate::map::IndexMap;
use crate::set::IndexSet;

impl<K, V, H> BorshSerialize for IndexMap<K, V, H>
where
    K: BorshSerialize + Ord,
    V: BorshSerialize,
    H: BuildHasher,
{
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        check_zst::<K>()?;

        let mut vec = self.iter().collect::<Vec<_>>();
        vec.sort_by(|(a, _), (b, _)| a.cmp(b));
        u32::try_from(vec.len())
            .map_err(|_| ErrorKind::InvalidData)?
            .serialize(writer)?;
        for (key, value) in vec {
            key.serialize(writer)?;
            value.serialize(writer)?;
        }
        Ok(())
    }
}

impl<K, V, H> BorshDeserialize for IndexMap<K, V, H>
where
    K: BorshDeserialize + Eq + Hash + Ord,
    V: BorshDeserialize,
    H: BuildHasher + Default,
{
    #[inline]
    fn deserialize_reader<R: Read>(reader: &mut R) -> Result<Self> {
        check_zst::<K>()?;
        let vec = <Vec<(K, V)>>::deserialize_reader(reader)?;
        Ok(vec.into_iter().collect::<IndexMap<K, V, H>>())
    }
}

#[cfg(feature = "borsh-schema")]
impl<K, V, H> BorshSchema for IndexMap<K, V, H>
where
    K: BorshSchema,
    V: BorshSchema,
{
    fn add_definitions_recursively(definitions: &mut BTreeMap<Declaration, Definition>) {
        let definition = Definition::Sequence {
            length_width: Definition::DEFAULT_LENGTH_WIDTH,
            length_range: Definition::DEFAULT_LENGTH_RANGE,
            elements: <(K, V)>::declaration(),
        };
        add_definition(Self::declaration(), definition, definitions);
        <(K, V)>::add_definitions_recursively(definitions);
    }

    fn declaration() -> Declaration {
        format!(r#"IndexMap<{}, {}>"#, K::declaration(), V::declaration())
    }
}

impl<T, H> BorshSerialize for IndexSet<T, H>
where
    T: BorshSerialize + Ord,
    H: BuildHasher,
{
    #[inline]
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        check_zst::<T>()?;

        let mut vec = self.iter().collect::<Vec<_>>();
        vec.sort();
        u32::try_from(vec.len())
            .map_err(|_| ErrorKind::InvalidData)?
            .serialize(writer)?;
        for item in vec {
            item.serialize(writer)?;
        }
        Ok(())
    }
}

impl<T, H> BorshDeserialize for IndexSet<T, H>
where
    T: BorshDeserialize + Eq + Hash + Ord,
    H: BuildHasher + Default,
{
    #[inline]
    fn deserialize_reader<R: Read>(reader: &mut R) -> Result<Self> {
        let vec = <Vec<T>>::deserialize_reader(reader)?;
        Ok(vec.into_iter().collect::<IndexSet<T, H>>())
    }
}

#[cfg(feature = "borsh-schema")]
impl<T, H> BorshSchema for IndexSet<T, H>
where
    T: BorshSchema,
{
    fn add_definitions_recursively(definitions: &mut BTreeMap<Declaration, Definition>) {
        let definition = Definition::Sequence {
            length_width: Definition::DEFAULT_LENGTH_WIDTH,
            length_range: Definition::DEFAULT_LENGTH_RANGE,
            elements: <T>::declaration(),
        };
        add_definition(Self::declaration(), definition, definitions);
        <T>::add_definitions_recursively(definitions);
    }

    fn declaration() -> Declaration {
        format!(r#"IndexSet<{}>"#, T::declaration())
    }
}

fn check_zst<T>() -> Result<()> {
    if size_of::<T>() == 0 {
        return Err(Error::new(ErrorKind::InvalidData, ERROR_ZST_FORBIDDEN));
    }
    Ok(())
}
