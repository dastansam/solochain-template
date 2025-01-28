use crate::{self as pallet_membership};
use frame_support::{derive_impl, parameter_types, traits::AsEnsureOriginWithArg, PalletId};
use sp_core::{ConstU32, ConstU8};
use sp_runtime::BuildStorage;

type Block = frame_system::mocking::MockBlock<Test>;
type AssetId = u32;

#[frame_support::runtime]
mod runtime {
    // The main runtime
    #[runtime::runtime]
    // Runtime Types to be generated
    #[runtime::derive(
        RuntimeCall,
        RuntimeEvent,
        RuntimeError,
        RuntimeOrigin,
        RuntimeFreezeReason,
        RuntimeHoldReason,
        RuntimeSlashReason,
        RuntimeLockId,
        RuntimeTask
    )]
    pub struct Test;

    #[runtime::pallet_index(0)]
    pub type System = frame_system::Pallet<Test>;
    #[runtime::pallet_index(1)]
    pub type Sudo = pallet_sudo::Pallet<Test>;
    #[runtime::pallet_index(2)]
    pub type Balances = pallet_balances::Pallet<Test>;
    #[runtime::pallet_index(3)]
    pub type Membership = pallet_membership::Pallet<Test>;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
    type Block = Block;
    type AccountData = pallet_balances::AccountData<u64>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
    type AccountStore = System;
}

#[derive_impl(pallet_sudo::config_preludes::TestDefaultConfig)]
impl pallet_sudo::Config for Test {}

parameter_types! {
    /// Pallet ID of the membership pallet
    pub const MembershipPalletId: PalletId = PalletId(*b"membersp");
    /// Club creation deposit
    pub const ClubCreationDeposit: u64 = 10;
}

/// 1 year in blocks.
/// This is only in the mock runtime, in a real runtime this would be calculated from the block time.
pub const YEARS: u32 = 100;

impl pallet_membership::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = ();
    /// Used for charging membership fees
    type Currency = Balances;
    /// The type of the club ID
    type ClubId = u32;
    /// The maximum number of years a membership can be valid for
    type MaxMembershipYears = ConstU8<100>;
    /// The pallet ID of the membership pallet, used to derive the account ID of the membership pallet
    type PalletId = MembershipPalletId;
    /// The maximum length of a string in this pallet
    type StringLimit = ConstU32<256>;
    /// Club creation deposit
    type ClubCreationDeposit = ClubCreationDeposit;
    /// The block number representing a year
    const YEAR_IN_BLOCKS: frame_system::pallet_prelude::BlockNumberFor<Self> = YEARS;
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
    // endowed balances for accounts
    let endowed_accounts = vec![
        (1, 100),
        (2, 100),
        (3, 100),
        (4, 100),
        (5, 100),
        (6, 100),
        (7, 100),
        (8, 100),
    ];

    let mut t = frame_system::GenesisConfig::default()
        .build_storage()
        .unwrap();

    pallet_balances::GenesisConfig::<Test> {
        balances: endowed_accounts.iter().cloned().collect(),
    }
    .assimilate_storage(&mut t)
    .unwrap();

    pallet_sudo::GenesisConfig::<Test> { key: Some(1) }
        .assimilate_storage(&mut t)
        .unwrap();

    t.into()
}
