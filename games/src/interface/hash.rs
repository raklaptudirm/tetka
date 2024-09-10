// Copyright © 2024 Rak Laptudirm <rak@laptudirm.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{fmt, ops};

/// Hash represents the semi-unique checksum of a Position used to efficiently
/// check for Position equality. Some properties that a Hash should possess
/// include determinism, uniform distribution, avalanche effect, and collision
/// resistance.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct Hash(u64);

impl Hash {
    /// new creates a new Hash value from a raw u64.
    #[must_use]
    pub fn new(raw: u64) -> Hash {
        Hash(raw)
    }
}

impl From<Hash> for u64 {
    #[must_use]
    fn from(value: Hash) -> Self {
        value.0
    }
}

impl ops::Not for Hash {
    type Output = Self;

    /// Not operator (!) switches the side to move for the Hash.
    #[must_use]
    fn not(self) -> Self::Output {
        Hash(!self.0)
    }
}

impl fmt::Display for Hash {
    /// Display allows Hash to be formatted in a human-readable form.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#X}", self.0)
    }
}

impl fmt::Debug for Hash {
    /// Debug allows Hash to be formatted in a human-readable form.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
