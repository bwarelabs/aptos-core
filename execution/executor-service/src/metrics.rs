// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use aptos_metrics_core::{exponential_buckets, HistogramVec, register_histogram_vec, IntCounterVec, register_int_counter_vec};
use once_cell::sync::Lazy;

pub static REMOTE_EXECUTOR_TIMER: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        // metric name
        "remote_executor_timer",
        // metric description
        "The time spent in remote shard on: \
         1. cmd_rx: after receiving the command from the coordinator; \
         2. cmd_rx_bcs_deser: deserializing the received command; \
         3. init_prefetch: initializing the prefetching of remote state values \
         4. kv_responses: processing the remote key value responses; \
         5. kv_deser: deserializing the remote key value responses; \
         6. prefetch_wait: waiting (approx) for the remote state values to be prefetched; \
         7. non_prefetch_wait: waiting for the remote state values that were not prefetched; ",
        // metric labels (dimensions)
        &["shard_id", "name"],
        exponential_buckets(/*start=*/ 1e-3, /*factor=*/ 2.0, /*count=*/ 20).unwrap(),
    ).unwrap()
});

pub static REMOTE_EXECUTOR_REMOTE_KV_COUNT: Lazy<IntCounterVec> = Lazy::new(|| {
    register_int_counter_vec!(
        // metric name
        "remote_executor_remote_kv_count",
        // metric description
        "KV counts on a shard for: \
         1. kv_responses: the number of remote key value responses received on a shard; \
         2. non_prefetch_kv: the number of remote key value responses received on a shard that were not prefetched; \
         3. prefetch_kv: the number of remote key value responses received on a shard that were prefetched; ",
        // metric labels (dimensions)
        &["shard_id", "name"],
    )
    .unwrap()
});
