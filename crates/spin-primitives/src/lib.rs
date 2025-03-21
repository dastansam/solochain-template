// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Primitives for Aura.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use codec::{Codec, Decode, Encode};
use sp_runtime::ConsensusEngineId;

pub mod digests;
pub mod inherents;

pub mod sr25519 {
    mod app_sr25519 {
        use sp_application_crypto::{app_crypto, key_types::AURA, sr25519};
        app_crypto!(sr25519, AURA);
    }

    sp_application_crypto::with_pair! {
        /// An Aura authority keypair using S/R 25519 as its crypto.
        pub type AuthorityPair = app_sr25519::Pair;
    }

    /// An Aura authority signature using S/R 25519 as its crypto.
    pub type AuthoritySignature = app_sr25519::Signature;

    /// An Aura authority identifier using S/R 25519 as its crypto.
    pub type AuthorityId = app_sr25519::Public;
}

pub mod ed25519 {
    mod app_ed25519 {
        use sp_application_crypto::{app_crypto, ed25519, key_types::AURA};
        app_crypto!(ed25519, AURA);
    }

    sp_application_crypto::with_pair! {
        /// An Aura authority keypair using Ed25519 as its crypto.
        pub type AuthorityPair = app_ed25519::Pair;
    }

    /// An Aura authority signature using Ed25519 as its crypto.
    pub type AuthoritySignature = app_ed25519::Signature;

    /// An Aura authority identifier using Ed25519 as its crypto.
    pub type AuthorityId = app_ed25519::Public;
}

pub use sp_consensus_slots::{Slot, SlotDuration};

/// The `ConsensusEngineId` of AuRa.
pub const AURA_ENGINE_ID: ConsensusEngineId = [b'a', b'u', b'r', b'a'];

/// The index of an authority.
pub type AuthorityIndex = u32;

/// The index of the session in Aura.
pub type SessionIndex = u32;

/// Auxilary data for Aura.
pub type AuraAuxData<A> = (Vec<A>, SessionIndex);

/// An consensus log item for Aura.
#[derive(Decode, Encode)]
pub enum ConsensusLog<AuthorityId: Codec> {
    /// The authorities have changed.
    #[codec(index = 1)]
    AuthoritiesChange(Vec<AuthorityId>),
    /// Disable the authority with given index.
    #[codec(index = 2)]
    OnDisabled(AuthorityIndex),
}

sp_api::decl_runtime_apis! {
    /// API necessary for block authorship with aura.
    pub trait AuraApi<AuthorityId: Codec> {
        /// Returns the slot duration for Aura.
        ///
        /// Currently, only the value provided by this type at genesis will be used.
        fn slot_duration() -> SlotDuration;

        /// Return the current set of authorities.
        fn aux_data() -> AuraAuxData<AuthorityId>;

        /// List of authorities
        fn authorities() -> Vec<AuthorityId>;
    }
}
