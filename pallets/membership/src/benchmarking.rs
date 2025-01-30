//! Benchmarking setup for pallet-template

use super::*;

#[allow(unused)]
use crate::Pallet as Membership;
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use scale_info::prelude::vec;

type BalanceOf<T> = <<T as Config>::Currency as frame_support::traits::fungible::Inspect<
    <T as frame_system::Config>::AccountId,
>>::Balance;

fn create_club<T: Config>(
    owner: T::AccountId,
    name: BoundedVec<u8, T::StringLimit>,
    fees: BalanceOf<T>,
) -> T::ClubId {
    let club_id = LastClubId::<T>::get();
    Membership::<T>::create_club(RawOrigin::Root.into(), owner.clone(), name, fees).unwrap();

    assert_eq!(Clubs::<T>::get(&club_id).unwrap().owner, owner);
    club_id
}

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn create_club_works() {
        let owner: T::AccountId = whitelisted_caller();
        let name: BoundedVec<u8, T::StringLimit> = vec![b'a'; 255].try_into().unwrap();
        let fee = 100u32.into();

        #[extrinsic_call]
        Membership::<T>::create_club(RawOrigin::Root, owner.clone(), name.clone(), fee);

        let club_id = LastClubId::<T>::get();
        let club = Clubs::<T>::get(club_id).unwrap();

        assert_eq!(club.owner, owner);
    }

    #[benchmark]
    fn add_member() {
        let owner: T::AccountId = whitelisted_caller();
        let name: BoundedVec<u8, T::StringLimit> = vec![b'a'; 255].try_into().unwrap();
        let fee = 100u32.into();
        let club_id = create_club::<T>(owner.clone(), name.clone(), fee);

        let member: T::AccountId = account("member", 0, 0);
        let member_name: BoundedVec<u8, T::StringLimit> = vec![b'b'; 255].try_into().unwrap();
        let years = 1;

        #[extrinsic_call]
        Membership::<T>::add_member(
            RawOrigin::Signed(owner),
            club_id.clone(),
            member.clone(),
            member_name,
            years,
        );

        let member_data = ClubMembers::<T>::get(club_id, member).unwrap();
        assert!(matches!(member_data.status, MemberStatus::Paid { .. }));
    }

    #[benchmark]
    fn transfer_club() {
        let owner: T::AccountId = whitelisted_caller();
        let name: BoundedVec<u8, T::StringLimit> = vec![b'a'; 255].try_into().unwrap();
        let fee = 100u32.into();
        let club_id = create_club::<T>(owner.clone(), name.clone(), fee);

        let new_owner: T::AccountId = account("new_owner", 0, 0);

        #[extrinsic_call]
        Membership::<T>::transfer_club(
            RawOrigin::Signed(owner),
            club_id.clone(),
            new_owner.clone(),
        );

        let club = Clubs::<T>::get(&club_id).unwrap();
        assert_eq!(club.owner, new_owner);
    }

    #[benchmark]
    fn set_annual_fee() {
        let owner: T::AccountId = whitelisted_caller();
        let name: BoundedVec<u8, T::StringLimit> = vec![b'a'; 255].try_into().unwrap();
        let fee = 100u32.into();
        let club_id = create_club::<T>(owner.clone(), name.clone(), fee);

        let new_fee = 200u32.into();

        #[extrinsic_call]
        Membership::<T>::set_annual_fee(RawOrigin::Signed(owner), club_id.clone(), new_fee);

        let club = Clubs::<T>::get(&club_id).unwrap();
        assert_eq!(club.fee, new_fee);
    }

    #[benchmark]
    fn extend_membership() {
        let owner: T::AccountId = whitelisted_caller();
        let name: BoundedVec<u8, T::StringLimit> = vec![b'a'; 255].try_into().unwrap();
        let fee = 100u32.into();
        let club_id = create_club::<T>(owner.clone(), name.clone(), fee);

        let member: T::AccountId = account("member", 0, 0);
        let member_name: BoundedVec<u8, T::StringLimit> = vec![b'b'; 255].try_into().unwrap();
        let years = 1;
        Membership::<T>::add_member(
            RawOrigin::Signed(owner.clone()).into(),
            club_id.clone(),
            member.clone(),
            member_name,
            years,
        )
        .unwrap();

        let extension_years = 1;

        #[extrinsic_call]
        Membership::<T>::extend_membership(
            RawOrigin::Signed(owner),
            club_id.clone(),
            member.clone(),
            extension_years,
        );

        let member_data = ClubMembers::<T>::get(club_id, member).unwrap();
        assert!(matches!(member_data.status, MemberStatus::Paid { .. }));
    }

    #[benchmark]
    fn withdraw_fees() {
        let owner: T::AccountId = whitelisted_caller();
        let name: BoundedVec<u8, T::StringLimit> = vec![b'a'; 255].try_into().unwrap();
        let fee = 100u32.into();
        let club_id = create_club::<T>(owner.clone(), name.clone(), fee);

        let member: T::AccountId = account("member", 0, 0);
        let member_name: BoundedVec<u8, T::StringLimit> = vec![b'b'; 255].try_into().unwrap();
        let years = 1;
        Membership::<T>::add_member(
            RawOrigin::Signed(owner.clone()).into(),
            club_id.clone(),
            member.clone(),
            member_name,
            years,
        )
        .unwrap();

        let amount = 50u32.into();

        #[extrinsic_call]
        Membership::<T>::withdraw_fees(RawOrigin::Root, owner.clone(), amount);

        // Verify the balance change (this depends on the runtime's currency implementation)
    }

    impl_benchmark_test_suite!(Membership, crate::mock::new_test_ext(), crate::mock::Test);
}
