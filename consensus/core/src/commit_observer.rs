// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use crate::commit::Commit;

#[derive(Default)]
pub struct CommitObserverRecoveredState {
    /// All previously committed subdags as stored on disk.
    pub commits: Vec<Commit>,
}
