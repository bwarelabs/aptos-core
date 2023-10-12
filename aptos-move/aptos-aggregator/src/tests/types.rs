// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{
    aggregator_v1_extension::AggregatorID,
    bounded_math::{BoundedMath, SignedU128},
    delta_change_set::serialize,
    resolver::TDelayedFieldView,
    types::{expect_ok, DelayedFieldID, DelayedFieldValue, DelayedFieldsSpeculativeError, PanicOr},
};
use aptos_types::state_store::{state_key::StateKey, state_value::StateValue};
use std::{cell::RefCell, collections::HashMap};

pub fn aggregator_v1_id_for_test(key: u128) -> AggregatorID {
    AggregatorID(aggregator_v1_state_key_for_test(key))
}

pub fn aggregator_v1_state_key_for_test(key: u128) -> StateKey {
    StateKey::raw(key.to_le_bytes().to_vec())
}

pub struct FakeAggregatorView {
    // TODO: consider adding deltas to test different read modes.
    v1_store: HashMap<StateKey, StateValue>,
    v2_store: HashMap<DelayedFieldID, DelayedFieldValue>,
    counter: RefCell<u32>,
}

impl Default for FakeAggregatorView {
    fn default() -> Self {
        Self {
            v1_store: HashMap::new(),
            v2_store: HashMap::new(),
            counter: RefCell::new(0),
        }
    }
}

impl FakeAggregatorView {
    pub fn set_from_state_key(&mut self, state_key: StateKey, value: u128) {
        let state_value = StateValue::new_legacy(serialize(&value).into());
        self.v1_store.insert(state_key, state_value);
    }

    pub fn set_from_aggregator_id(&mut self, id: DelayedFieldID, value: u128) {
        self.v2_store
            .insert(id, DelayedFieldValue::Aggregator(value));
    }
}

impl TDelayedFieldView for FakeAggregatorView {
    type IdentifierV1 = StateKey;
    type IdentifierV2 = DelayedFieldID;

    fn get_aggregator_v1_state_value(
        &self,
        state_key: &Self::IdentifierV1,
    ) -> anyhow::Result<Option<StateValue>> {
        Ok(self.v1_store.get(state_key).cloned())
    }

    fn get_delayed_field_value(
        &self,
        id: &Self::IdentifierV2,
    ) -> Result<DelayedFieldValue, PanicOr<DelayedFieldsSpeculativeError>> {
        self.v2_store
            .get(id)
            .cloned()
            .ok_or(PanicOr::Or(DelayedFieldsSpeculativeError::NotFound(*id)))
    }

    fn delayed_field_try_add_delta_outcome(
        &self,
        id: &Self::IdentifierV2,
        base_delta: &SignedU128,
        delta: &SignedU128,
        max_value: u128,
    ) -> Result<bool, PanicOr<DelayedFieldsSpeculativeError>> {
        let base_value = self.get_delayed_field_value(id)?.into_aggregator_value()?;
        let math = BoundedMath::new(max_value);
        let base = expect_ok(math.unsigned_add_delta(base_value, base_delta))?;
        Ok(math.unsigned_add_delta(base, delta).is_ok())
    }

    fn generate_delayed_field_id(&self) -> Self::IdentifierV2 {
        let mut counter = self.counter.borrow_mut();
        let id = Self::IdentifierV2::new(*counter as u64);
        *counter += 1;
        id
    }
}
