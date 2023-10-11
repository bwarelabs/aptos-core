// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{
    types::{GroupReadResult, MVModulesOutput},
    utils::module_hash,
};
use aptos_crypto::hash::HashValue;
use aptos_types::{
    executable::{Executable, ExecutableDescriptor, ModulePath},
    write_set::TransactionWrite,
};
use aptos_vm_types::resource_group_adapter::group_size_as_sum;
use serde::Serialize;
use std::{cell::RefCell, collections::HashMap, fmt::Debug, hash::Hash, sync::Arc};

/// UnsyncMap is designed to mimic the functionality of MVHashMap for sequential execution.
/// In this case only the latest recorded version is relevant, simplifying the implementation.
/// The functionality also includes Executable caching based on the hash of ExecutableDescriptor
/// (i.e. module hash for modules published during the latest block - not at storage version).
pub struct UnsyncMap<
    K: ModulePath,
    T: Hash + Clone + Debug + Eq + Serialize,
    V: TransactionWrite,
    X: Executable,
> {
    // Only use Arc to provide unified interfaces with the MVHashMap / concurrent setting. This
    // simplifies the trait-based integration for executable caching. TODO: better representation.
    // Optional hash can store the hash of the module to avoid re-computations.
    map: RefCell<HashMap<K, (Arc<V>, Option<HashValue>)>>,
    group_cache: RefCell<HashMap<K, HashMap<T, Arc<V>>>>,
    executable_cache: RefCell<HashMap<HashValue, Arc<X>>>,
    executable_bytes: RefCell<usize>,
}

impl<
        K: ModulePath + Hash + Clone + Eq,
        T: Hash + Clone + Debug + Eq + Serialize,
        V: TransactionWrite,
        X: Executable,
    > Default for UnsyncMap<K, T, V, X>
{
    fn default() -> Self {
        Self {
            map: RefCell::new(HashMap::new()),
            group_cache: RefCell::new(HashMap::new()),
            executable_cache: RefCell::new(HashMap::new()),
            executable_bytes: RefCell::new(0),
        }
    }
}

impl<
        K: ModulePath + Hash + Clone + Eq,
        T: Hash + Clone + Debug + Eq + Serialize,
        V: TransactionWrite,
        X: Executable,
    > UnsyncMap<K, T, V, X>
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn provide_group_base_values(
        &self,
        group_key: K,
        base_values: impl IntoIterator<Item = (T, V)>,
    ) {
        self.group_cache.borrow_mut().insert(
            group_key,
            base_values
                .into_iter()
                .map(|(t, v)| (t, Arc::new(v)))
                .collect(),
        );
    }

    pub fn get_group_size(&self, group_key: &K) -> anyhow::Result<GroupReadResult> {
        Ok(match self.group_cache.borrow().get(group_key) {
            Some(group_map) => GroupReadResult::Size(group_size_as_sum(
                group_map
                    .iter()
                    .flat_map(|(t, v)| v.bytes().map(|bytes| (t, bytes))),
            )?),
            None => GroupReadResult::Uninitialized,
        })
    }

    pub fn get_value_from_group(&self, group_key: &K, value_tag: &T) -> GroupReadResult {
        self.group_cache.borrow().get(group_key).map_or(
            GroupReadResult::Uninitialized,
            |group_map| {
                GroupReadResult::Value(
                    group_map
                        .get(value_tag)
                        .map(|v| v.extract_raw_bytes())
                        .flatten(),
                )
            },
        )
    }

    /// Contains the latest group ops (excluding deletions) for the given group key.
    pub fn group_to_commit(&self, group_key: &K) -> Vec<(T, Arc<V>)> {
        self.group_cache
            .borrow()
            .get(group_key)
            .expect("Resource group must be cached")
            .iter()
            .filter_map(|(t, arc_v)| arc_v.bytes().map(|_| (t.clone(), arc_v.clone())))
            .collect()
    }

    pub fn insert_group_op(&self, group_key: &K, value_tag: T, v: V) {
        self.group_cache
            .borrow_mut()
            .get_mut(group_key)
            .expect("Resource group must be cached")
            .insert(value_tag, Arc::new(v));
    }

    pub fn fetch_data(&self, key: &K) -> Option<Arc<V>> {
        self.map.borrow().get(key).map(|entry| entry.0.clone())
    }

    pub fn fetch_module(&self, key: &K) -> Option<MVModulesOutput<V, X>> {
        use MVModulesOutput::*;
        debug_assert!(key.module_path().is_some());

        self.map.borrow_mut().get_mut(key).map(|entry| {
            let hash = entry.1.get_or_insert(module_hash(entry.0.as_ref()));

            self.executable_cache.borrow().get(hash).map_or_else(
                || Module((entry.0.clone(), *hash)),
                |x| Executable((x.clone(), ExecutableDescriptor::Published(*hash))),
            )
        })
    }

    pub fn write(&self, key: K, value: V) {
        self.map.borrow_mut().insert(key, (Arc::new(value), None));
    }

    /// We return false if the executable was already stored, as this isn't supposed to happen
    /// during sequential execution (and the caller may choose to e.g. log a message).
    /// Versioned modules storage does not cache executables at storage version, hence directly
    /// the descriptor hash in ExecutableDescriptor::Published is provided.
    pub fn store_executable(&self, descriptor_hash: HashValue, executable: X) -> bool {
        let size = executable.size_bytes();
        if self
            .executable_cache
            .borrow_mut()
            .insert(descriptor_hash, Arc::new(executable))
            .is_some()
        {
            *self.executable_bytes.borrow_mut() += size;
            true
        } else {
            false
        }
    }

    pub fn executable_size(&self) -> usize {
        *self.executable_bytes.borrow()
    }
}
