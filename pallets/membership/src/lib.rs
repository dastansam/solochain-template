//! Write a pallet for club member management
//! 1. two roles in the club, root can create a new club, club owner can add someone to
//! club
//! 2. need to pay some token to create a new club, the club owner can be transferred,
//! club owner can set the annual expense for club membership
//! 3. account needs to pay a token to be a member based on annual expenses, max is
//! 100 years
//! 4. membership will be expired, and need renewal

//! Write a pallet for club member management
//! 1. a club has an owner which can add members
//! 2. the club can be transferred by the owner to a different owner
//! 3. the owner sets the annual club membership fee
//! 4. members that haven't payed their yearly membership fee are inactive
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_system::pallet_prelude::BlockNumberFor;
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{CheckedDiv, CheckedMul},
    Saturating,
};

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

// FRAME pallets require their own "mock runtimes" to be able to run unit tests. This module
// contains a mock runtime specific for testing this pallet's functionality.
#[cfg(test)]
mod mock;

// This module contains the unit tests for this pallet.
// Learn about pallet unit testing here: https://docs.substrate.io/test/unit-testing/
#[cfg(test)]
mod tests;

// Every callable function or "dispatchable" a pallet exposes must have weight values that correctly
// estimate a dispatchable's execution time. The benchmarking module is used to calculate weights
// for each dispatchable and generates this pallet's weight.rs file. Learn more about benchmarking here: https://docs.substrate.io/test/benchmark/
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
use sp_core::Get;
use sp_runtime::BoundedVec;
pub use weights::*;

/// Represents the club
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, PartialEq, Eq, Clone, Debug)]
#[scale_info(skip_type_params(StringLimit))]
struct Club<AccountId, Balance, StringLimit: Get<u32>> {
    /// Name of the club, bounded by the `T::MaxClubNameLength`, since we can not store unbounded data on-chain
    name: BoundedVec<u8, StringLimit>,
    /// Annual fee to be a member of the club
    fee: Balance,
    /// Owner of the club
    owner: AccountId,
}

/// Status of the member
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, PartialEq, Eq, Clone, Debug)]
pub enum MemberStatus<BlockNumber> {
    Paid {
        /// The block number until the membership is valid
        until: BlockNumber,
    },
    Inactive,
}

/// A member of the club
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, PartialEq, Eq, Clone, Debug)]
#[scale_info(skip_type_params(StringLimit))]
struct ClubMemberMetadata<StringLimit: Get<u32>, BlockNumber> {
    /// Name of the club member
    name: BoundedVec<u8, StringLimit>,
    status: MemberStatus<BlockNumber>,
}

/// Action to be performed on the club
enum ClubAction<AccountId, Balance> {
    /// Change owner
    TransferOwnership(AccountId),
    /// Set annual fee
    SetAnnualFee(Balance),
}

/// Balance type of `Config`
type BalanceOf<T> = <<T as Config>::Currency as frame_support::traits::fungible::Inspect<
    <T as frame_system::Config>::AccountId,
>>::Balance;

/// `Club` type of `Config`
type ClubOf<T> =
    Club<<T as frame_system::Config>::AccountId, BalanceOf<T>, <T as Config>::StringLimit>;

/// `ClubMember` type of `Config`
type ClubMemberMetadataOf<T> = ClubMemberMetadata<<T as Config>::StringLimit, BlockNumberFor<T>>;

// All pallet logic is defined in its own module and must be annotated by the `pallet` attribute.
#[frame_support::pallet]
pub mod pallet {

    // Import various useful types required by all FRAME pallets.
    use super::*;
    use codec::FullCodec;
    use core::fmt::Debug;
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::{StorageDoubleMap, *},
        traits::{fungible::Mutate, tokens::Preservation, Incrementable},
        Blake2_128Concat, PalletId,
    };
    use frame_system::pallet_prelude::{OriginFor, *};
    use sp_core::Get;
    use sp_runtime::traits::AccountIdConversion;

    // The `Pallet` struct serves as a placeholder to implement traits, methods and dispatchables
    // (`Call`s) in this pallet.
    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// The pallet's configuration trait.
    ///
    /// All our types and constants a pallet depends on must be declared here.
    /// These types are defined generically and made concrete when the pallet is declared in the
    /// `runtime/src/lib.rs` file of your chain.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// A type representing the weights required by the dispatchables of this pallet.
        type WeightInfo: WeightInfo;
        /// Currency type for charging fees for the membership in the club
        type Currency: Mutate<Self::AccountId>;
        /// Club id type
        type ClubId: FullCodec
            + TypeInfo
            + MaxEncodedLen
            + Clone
            + PartialEq
            + Debug
            + Default
            + Incrementable;
        /// The pallet's id, used for deriving its sovereign account ID.
        #[pallet::constant]
        type PalletId: Get<PalletId>;
        /// Bound for the string length in the name of club/member
        #[pallet::constant]
        type StringLimit: Get<u32>;
        /// Max number of years of a membership
        #[pallet::constant]
        type MaxMembershipYears: Get<u8>;
        /// Club creation deposit
        #[pallet::constant]
        type ClubCreationDeposit: Get<BalanceOf<Self>>;

        /// One year in blocks
        const YEAR_IN_BLOCKS: BlockNumberFor<Self>;
    }

    /// Recent club id
    #[pallet::storage]
    pub(crate) type LastClubId<T: Config> = StorageValue<_, T::ClubId, ValueQuery>;

    /// Tracks clubs
    #[pallet::storage]
    pub(crate) type Clubs<T: Config> = StorageMap<_, Blake2_128Concat, T::ClubId, ClubOf<T>>;

    /// Tracks clus to members mapping.
    ///
    /// Use `T::AccountId` as a second key to efficiently retrieve member metadata.
    #[pallet::storage]
    pub(crate) type ClubMembers<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::ClubId,
        Blake2_128Concat,
        T::AccountId,
        ClubMemberMetadataOf<T>,
    >;

    /// Events that functions in this pallet can emit.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new club is created
        ClubCreated {
            club_id: T::ClubId,
            owner: T::AccountId,
            name: BoundedVec<u8, T::StringLimit>,
            fee: BalanceOf<T>,
        },
        /// Club transferred
        ClubTransferred {
            club_id: T::ClubId,
            old_owner: T::AccountId,
            new_owner: T::AccountId,
        },
        /// Club new annual fee is set
        ClubAnnualFeeSet {
            club_id: T::ClubId,
            old_fee: BalanceOf<T>,
            new_fee: BalanceOf<T>,
        },
        /// A new member is added to the club. `since` and `until` can be `None` if the member can not pay the fee
        MemberAdded {
            club_id: T::ClubId,
            member: T::AccountId,
            status: MemberStatus<BlockNumberFor<T>>,
        },
        /// Membership is extended
        MembershipExtended {
            club_id: T::ClubId,
            member: T::AccountId,
            new_status: MemberStatus<BlockNumberFor<T>>,
        },
    }

    /// Errors that can be returned by this pallet.
    #[pallet::error]
    pub enum Error<T> {
        /// Maximum number of members reached
        MaxMembersReached,
        /// Club not found
        ClubNotFound,
        /// Member not found
        MemberNotFound,
        /// Member can not pay the fee
        MemberCanNotPayFee,
        /// Generic arithmetic error, could be an under/overflow, divizion by zero, etc.
        ArithmeticError,
        /// Caller is not authorised to perform the action, i.e only owner can add members, only owner can transfer the ownership, etc.
        NotAuthorized,
        /// Membership can not be extended beyond `T::MaxMembershipYears` or be zero
        InvalidMembershipPeriod,
    }

    /// The pallet's dispatchable functions ([`Call`]s).
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new club
        ///
        /// Only root/sudo can create a new club
        ///
        /// # Arguments
        ///
        /// * `origin` - sender of the transaction
        /// * `name` - name of the club
        /// * `fee` - annual fee to be a member of the club
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::do_something())]
        pub fn create_club(
            origin: OriginFor<T>,
            owner: T::AccountId,
            name: BoundedVec<u8, T::StringLimit>,
            fee: BalanceOf<T>,
        ) -> DispatchResult {
            // Check that the extrinsic was signed by root
            ensure_root(origin)?;

            // Get the last club id
            let club_id = LastClubId::<T>::get();

            let club = Club {
                name: name.clone(),
                fee: fee.clone(),
                owner: owner.clone(),
            };
            Clubs::<T>::insert(&club_id, club);

            let next_club_id = club_id.increment().ok_or(Error::<T>::ArithmeticError)?;

            LastClubId::<T>::put(next_club_id);

            // Charge the club creation deposit
            T::Currency::transfer(
                &owner,
                &Self::account_id(),
                T::ClubCreationDeposit::get(),
                Preservation::Preserve,
            )?;

            // Emit an event that a new club is created
            Self::deposit_event(Event::ClubCreated {
                club_id,
                owner,
                name,
                fee,
            });

            Ok(())
        }

        /// Add new member to the club. Only owner can add new members.
        ///
        /// If the account of the member does not have enough balance to pay the fee, the member is added but is inactive.
        ///
        /// # Arguments
        ///
        /// * `origin` - sender of the transaction
        /// * `club_id` - id of the club
        /// * `name` - name of the member
        /// * `role` - role of the member
        /// * `fee` - fee to be paid
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::cause_error())]
        pub fn add_member(
            origin: OriginFor<T>,
            club_id: T::ClubId,
            member: T::AccountId,
            name: BoundedVec<u8, T::StringLimit>,
            years: u8,
        ) -> DispatchResult {
            // no need to access storage, validate signature, etc. if this doesn't hold
            ensure!(
                years <= T::MaxMembershipYears::get() && years != 0,
                Error::<T>::InvalidMembershipPeriod
            );

            let who = ensure_signed(origin)?;

            let club = Clubs::<T>::get(&club_id).ok_or(Error::<T>::ClubNotFound)?;

            ensure!(who == club.owner, Error::<T>::NotAuthorized);

            // TODO: this looks clunky but safe, find a better way to handle conversion
            let years_as_balance: BalanceOf<T> = (years as u32).into();
            let total_fees = club
                .fee
                .checked_mul(&years_as_balance)
                .ok_or(Error::<T>::ArithmeticError)?;

            // Try to charge the fees from the member's account. If the member can not pay the fees, the member is added but is inactive
            let status = match T::Currency::transfer(
                &member,
                &Self::account_id(),
                total_fees,
                Preservation::Preserve,
            ) {
                Ok(_) => {
                    let current_block_number = frame_system::Pallet::<T>::block_number();
                    let expiration_block_number = current_block_number
                        .saturating_add(T::YEAR_IN_BLOCKS.saturating_mul(years.into()));
                    MemberStatus::Paid {
                        until: expiration_block_number,
                    }
                }
                Err(_) => MemberStatus::Inactive,
            };

            let club_member = ClubMemberMetadata {
                name,
                status: status.clone(),
            };

            ClubMembers::<T>::insert(&club_id, member.clone(), club_member);

            Self::deposit_event(Event::MemberAdded {
                club_id,
                member,
                status,
            });

            Ok(())
        }

        /// Transfer the ownership of the club
        /// Only the owner of the club can transfer the ownership
        ///
        /// # Arguments
        ///
        /// * `origin` - sender of the transaction
        /// * `club_id` - id of the club
        /// * `new_owner` - new owner of the club
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::cause_error())]
        pub fn transfer_club(
            origin: OriginFor<T>,
            club_id: T::ClubId,
            new_owner: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::modify_club(club_id, who, ClubAction::TransferOwnership(new_owner))?;

            Ok(())
        }

        /// Set annual fee for membership in the club
        ///
        /// Only the owner of the club can set the annual fee
        ///
        /// # Arguments
        ///
        /// * `origin` - sender of the transaction
        /// * `club_id` - id of the club
        /// * `new_fee` - new fee value
        #[pallet::call_index(3)]
        #[pallet::weight(T::WeightInfo::cause_error())]
        pub fn set_annual_fee(
            origin: OriginFor<T>,
            club_id: T::ClubId,
            new_fee: BalanceOf<T>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            Self::modify_club(club_id, who, ClubAction::SetAnnualFee(new_fee))?;

            Ok(())
        }

        /// Extend the membership of the member. Both the club owner and member himself can extend the membership.
        ///
        /// But fees are deducted from the member's account.
        ///
        /// # Arguments
        ///
        /// * `origin` - sender of the transaction
        /// * `club_id` - id of the club
        /// * `member` - member to extend the membership
        /// * `years` - number of years to extend the membership
        #[pallet::call_index(4)]
        #[pallet::weight(T::WeightInfo::cause_error())]
        pub fn extend_membership(
            origin: OriginFor<T>,
            club_id: T::ClubId,
            member: T::AccountId,
            years: u8,
        ) -> DispatchResult {
            // no need to access storage, validate signature, etc. if this doesn't hold
            ensure!(
                years <= T::MaxMembershipYears::get() && years != 0,
                Error::<T>::InvalidMembershipPeriod
            );

            let who = ensure_signed(origin)?;

            let club = Clubs::<T>::get(&club_id).ok_or(Error::<T>::ClubNotFound)?;

            ensure!(
                who == club.owner || who == member,
                Error::<T>::NotAuthorized
            );

            let years_as_balance: BalanceOf<T> = (years as u32).into();

            let total_fees = club
                .fee
                .checked_mul(&years_as_balance)
                .ok_or(Error::<T>::ArithmeticError)?;

            T::Currency::transfer(
                &member,
                &Self::account_id(),
                total_fees,
                Preservation::Preserve,
            )?;

            let club_member =
                ClubMembers::<T>::get(&club_id, &member).ok_or(Error::<T>::MemberNotFound)?;
            let current_block_number = frame_system::Pallet::<T>::block_number();

            // membership might not be expired yet, in that case we extend from the expiration date in the future
            let status = match club_member.status {
                MemberStatus::Paid { until } => {
                    // ensure we respect `T::MaxMembershipYears` limit
                    let extend_from = until.max(current_block_number);
                    // This is safe because `extend_from` is at least `block_number`
                    let blocks_left = extend_from.saturating_sub(current_block_number);
                    let years_left = blocks_left
                        .checked_div(&T::YEAR_IN_BLOCKS)
                        .ok_or(Error::<T>::ArithmeticError)?;
                    ensure!(
                        years_left.saturating_add(years.into())
                            < T::MaxMembershipYears::get().into(),
                        Error::<T>::InvalidMembershipPeriod
                    );

                    let extend_to =
                        extend_from.saturating_add(T::YEAR_IN_BLOCKS.saturating_mul(years.into()));
                    MemberStatus::Paid { until: extend_to }
                }
                MemberStatus::Inactive => {
                    let extend_to = current_block_number
                        .saturating_add(T::YEAR_IN_BLOCKS.saturating_mul(years.into()));
                    MemberStatus::Paid { until: extend_to }
                }
            };

            ClubMembers::<T>::insert(
                &club_id,
                &member,
                ClubMemberMetadata {
                    name: club_member.name,
                    status: status.clone(),
                },
            );

            Self::deposit_event(Event::MembershipExtended {
                club_id,
                member,
                new_status: status,
            });

            Ok(())
        }

        /// Withdraw the accumulated fees from the protocol
        ///
        /// Only the root can withdraw the fees
        ///
        /// # Arguments
        ///
        /// * `origin` - sender of the transaction
        /// * `to` - account to withdraw the fees
        /// * `amount` - amount to withdraw
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::cause_error())]
        pub fn withdraw_fees(
            origin: OriginFor<T>,
            to: T::AccountId,
            amount: BalanceOf<T>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            // Withdraw the fees, if the amount is higher than the accumulated fees, it will simply fail
            T::Currency::transfer(&Self::account_id(), &to, amount, Preservation::Preserve)?;

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// Get the account id of the pallet
        fn account_id() -> T::AccountId {
            T::PalletId::get().into_account_truncating()
        }

        /// Perform some action on the club, like transfer ownership, set annual fee, etc.
        ///
        /// Checks if the caller is authorized to perform the action
        fn modify_club(
            club_id: T::ClubId,
            who: T::AccountId,
            action: ClubAction<T::AccountId, BalanceOf<T>>,
        ) -> DispatchResult {
            let mut club = Clubs::<T>::get(&club_id).ok_or(Error::<T>::ClubNotFound)?;

            ensure!(who == club.owner, Error::<T>::NotAuthorized);

            match action {
                ClubAction::TransferOwnership(new_owner) => {
                    Self::deposit_event(Event::ClubTransferred {
                        club_id: club_id.clone(),
                        old_owner: club.owner,
                        new_owner: new_owner.clone(),
                    });
                    club.owner = new_owner;
                }
                ClubAction::SetAnnualFee(fee) => {
                    Self::deposit_event(Event::ClubAnnualFeeSet {
                        club_id: club_id.clone(),
                        old_fee: club.fee,
                        new_fee: fee.clone(),
                    });
                    club.fee = fee;
                }
            }

            Clubs::<T>::insert(&club_id, club);

            Ok(())
        }
    }
}
