// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use aptos_push_metrics::{register_histogram_vec, register_int_gauge, HistogramVec, IntGauge};
use once_cell::sync::Lazy;

pub static HEARTBEAT_TS: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "aptos_db_backup_coordinator_heartbeat_timestamp_s",
        "Timestamp when the backup coordinator successfully updates state from the backup service."
    )
    .unwrap()
});

pub static EPOCH_ENDING_EPOCH: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "aptos_db_backup_coordinator_epoch_ending_epoch",
        "Epoch of the latest epoch ending backed up."
    )
    .unwrap()
});

pub static STATE_SNAPSHOT_EPOCH: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "aptos_db_backup_coordinator_state_snapshot_epoch",
        "The epoch at the end of which the latest state snapshot was taken."
    )
    .unwrap()
});

pub static TRANSACTION_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "aptos_db_backup_coordinator_transaction_version",
        "Version of the latest transaction backed up."
    )
    .unwrap()
});

pub static COMPACTED_TXN_VERSION: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "aptos_db_backup_coordinator_compacted_version",
        "Version of the latest transaction metadata compacted."
    )
    .unwrap()
});

pub static EPOCH_ENDING_DURATIONS: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "aptos_db_backup_coordinator_epoch_ending_durations_s",
        "processing time for a step in epoch ending backup",
        &["Step"]
    )
    .unwrap()
});

pub static STATE_SNAPSHOT_DURATIONS: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "aptos_db_backup_coordinator_state_snapshot_durations_s",
        "processing time for a step in state snapshot backup",
        &["Step"]
    )
    .unwrap()
});

pub static TXN_DURATIONS: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "aptos_db_backup_coordinator_transaction_durations_s",
        "processing time for a step in transaction backup",
        &["Step"]
    )
    .unwrap()
});
