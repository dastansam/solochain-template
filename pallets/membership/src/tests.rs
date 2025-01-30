use crate::{mock::*, ClubMembers, Clubs, Error, Event, LastClubId, MemberStatus};
use frame_support::{assert_noop, assert_ok, traits::fungible::Inspect};
use sp_runtime::{
    traits::{AccountIdConversion, BadOrigin},
    BoundedVec,
};

/// Convenience functions
impl<BlockNumber: Clone> MemberStatus<BlockNumber> {
    fn until(&self) -> Option<BlockNumber> {
        match self {
            MemberStatus::Paid { until } => Some(until.clone()),
            MemberStatus::Inactive => None,
        }
    }
}

type Name = BoundedVec<u8, StringLimit>;

/// Gets runtime events
fn events() -> Vec<RuntimeEvent> {
    let evt = System::events()
        .into_iter()
        .map(|evt| evt.event)
        .collect::<Vec<_>>();
    System::reset_events();
    evt
}

/// Creates a club
fn create_club(owner: u64, name: Name, fee: u64) -> u32 {
    let club_id = LastClubId::<Test>::get();
    Membership::create_club(RuntimeOrigin::root(), owner, name, fee).unwrap();

    assert_eq!(Clubs::<Test>::get(club_id).unwrap().owner, owner);
    club_id
}

/// Add new member to a club
fn add_member(club_id: u32, member: u64, name: Name, years: u8) {
    let club = Clubs::<Test>::get(club_id).unwrap();
    Membership::add_member(
        RuntimeOrigin::signed(club.owner),
        club_id,
        member,
        name,
        years,
    )
    .unwrap();
}

#[test]
fn create_club_works() {
    new_test_ext().execute_with(|| {
        let club_id = LastClubId::<Test>::get();

        let owner = 2;
        let fee = 100;
        let name: Name = b"Aclub".to_vec().try_into().unwrap();

        // Non root call fails
        assert_noop!(
            Membership::create_club(RuntimeOrigin::signed(2), owner, name.clone(), fee),
            BadOrigin
        );

        assert_ok!(Membership::create_club(
            RuntimeOrigin::root(),
            owner,
            name.clone(),
            fee
        ));
        assert_eq!(Clubs::<Test>::get(club_id).unwrap().owner, owner);

        // club_id is incremented by 1
        assert_eq!(LastClubId::<Test>::get(), club_id + 1);

        let mut events = events();
        assert_eq!(
            events.pop().unwrap(),
            RuntimeEvent::Membership(Event::<Test>::ClubCreated {
                club_id: club_id,
                owner,
                name,
                fee
            })
        );

        // Check the event is emitted
        assert_eq!(
            events.pop().unwrap(),
            RuntimeEvent::Balances(pallet_balances::Event::<Test>::Transfer {
                from: 2,
                to: MembershipPalletId::get().into_account_truncating(),
                amount: ClubCreationDeposit::get()
            }),
        );

        // create 4 more clubs
        for i in 0..4 {
            let name: Name = b"test".to_vec().try_into().unwrap();
            assert_ok!(Membership::create_club(
                RuntimeOrigin::root(),
                i + 4,
                name.clone(),
                i * 5 + 1
            ));
        }

        // Check the deposit is charged
        let membership_pallet_pool: u64 = MembershipPalletId::get().into_account_truncating();
        assert_eq!(
            Balances::free_balance(membership_pallet_pool),
            ClubCreationDeposit::get() * 5
        )
    });
}

#[test]
fn adding_member_works() {
    new_test_ext().execute_with(|| {
        let club_name: Name = b"AClub".to_vec().try_into().unwrap();
        // make it sufficiently high, so that user does not have enough balance
        let club_fee = 5;
        let club_owner = 2;
        let club_id = create_club(club_owner, club_name, club_fee);

        let member_name: Name = b"AMember".to_vec().try_into().unwrap();
        let member = 3;

        // non club owner can not add member
        assert_noop!(
            Membership::add_member(
                RuntimeOrigin::signed(member),
                club_id,
                member,
                member_name.clone(),
                3
            ),
            Error::<Test>::NotAuthorized
        );

        assert_ok!(Membership::add_member(
            RuntimeOrigin::signed(club_owner),
            club_id,
            member,
            member_name.clone(),
            5
        ));

        let block_number = System::block_number();

        let mut events = events();
        assert_eq!(
            events.pop().unwrap(),
            RuntimeEvent::Membership(Event::<Test>::MemberAdded {
                club_id,
                member,
                status: MemberStatus::Paid {
                    until: block_number + YEARS * 5,
                }
            })
        );

        // member is charged the club fee
        assert_eq!(
            events.pop().unwrap(),
            RuntimeEvent::Balances(pallet_balances::Event::<Test>::Transfer {
                from: member,
                to: MembershipPalletId::get().into_account_truncating(),
                amount: club_fee * 5
            })
        );

        // assert!(ClubMembers::get(&club_id, &member).unwrap().name == member_name,);
        assert_eq!(
            ClubMembers::<Test>::get(&club_id, &member).unwrap().status,
            MemberStatus::Paid {
                until: block_number + YEARS * 5,
            },
        );

        for i in 6..10 {
            let member_name: Name = b"user".to_vec().try_into().unwrap();
            assert_ok!(Membership::add_member(
                RuntimeOrigin::signed(club_owner),
                club_id,
                i,
                member_name,
                10 + i as u8
            ));
        }

        // fees are being paid
        let membership_pallet_pool: u64 = MembershipPalletId::get().into_account_truncating();
        let balance = Balances::free_balance(membership_pallet_pool);

        assert_eq!(
            balance,
            ClubCreationDeposit::get()
                + club_fee * 5
                + club_fee * 16
                + club_fee * 17
                + club_fee * 18
                + club_fee * 19 // 4 more members
        );
    });
}

/// Transfer ownership edge cases
#[test]
fn transfer_ownership_works() {
    new_test_ext().execute_with(|| {
        let club_name = b"AClub".to_vec().try_into().unwrap();
        let club_fee = 5;
        let club_owner = 2;

        let club_id = create_club(club_owner, club_name, club_fee);

        let new_owner = 8;

        // cannot transfer non-existing club
        assert_noop!(
            Membership::transfer_club(RuntimeOrigin::signed(club_owner), club_id + 1, new_owner,),
            Error::<Test>::ClubNotFound,
        );

        let not_owner = 5;
        // cannot transfer someone else's club
        assert_noop!(
            Membership::transfer_club(RuntimeOrigin::signed(not_owner), club_id, new_owner),
            Error::<Test>::NotAuthorized
        );

        assert_ok!(Membership::transfer_club(
            RuntimeOrigin::signed(club_owner),
            club_id,
            new_owner
        ));

        let club = Clubs::<Test>::get(club_id).unwrap();

        assert_eq!(club.owner, new_owner)
    })
}

/// Set annual fee edge cases
#[test]
fn set_annual_fee_works() {
    new_test_ext().execute_with(|| {
        let club_name = b"AClub".to_vec().try_into().unwrap();
        let club_fee = 5;
        let club_owner = 2;

        let club_id = create_club(club_owner, club_name, club_fee);

        // cannot change annual fee of a non-existing club

        let new_fee = club_fee * 10;
        // cannot transfer non-existing club
        assert_noop!(
            Membership::set_annual_fee(RuntimeOrigin::signed(club_owner), club_id + 1, new_fee,),
            Error::<Test>::ClubNotFound,
        );

        let not_owner = 5;
        // cannot set annual fee of someone else's club
        assert_noop!(
            Membership::set_annual_fee(RuntimeOrigin::signed(not_owner), club_id, new_fee),
            Error::<Test>::NotAuthorized
        );

        assert_ok!(Membership::set_annual_fee(
            RuntimeOrigin::signed(club_owner),
            club_id,
            new_fee
        ));

        let club = Clubs::<Test>::get(club_id).unwrap();

        assert_eq!(club.fee, new_fee)
    })
}

/// Withdrawal of accumulated fees works
#[test]
fn fee_charging_and_withdraw_fees_works() {
    new_test_ext().execute_with(|| {
        let membership_pallet_pool: u64 = MembershipPalletId::get().into_account_truncating();

        let fees_dest = 1;

        // should fail when there is nothing accumulated
        assert_noop!(
            Membership::withdraw_fees(RuntimeOrigin::root(), fees_dest, 1),
            sp_runtime::TokenError::FundsUnavailable,
        );

        let club_name = b"AClub".to_vec().try_into().unwrap();
        let club_fee = 5;
        let club_owner = 2;

        let club_id = create_club(club_owner, club_name, club_fee);
        let balance = Balances::free_balance(membership_pallet_pool);

        // there should be a `T::ClubCreationDeposit`
        assert_eq!(balance, ClubCreationDeposit::get());

        let member = 4;
        let name = b"Alice".to_vec().try_into().unwrap();
        let years = 1;
        add_member(club_id, member, name, years);

        let balance = Balances::free_balance(membership_pallet_pool);
        assert_eq!(
            balance,
            ClubCreationDeposit::get() + years as u64 * club_fee,
        );

        let ed = <Balances as Inspect<u64>>::minimum_balance();

        assert_noop!(
            Membership::withdraw_fees(
                RuntimeOrigin::signed(club_owner),
                fees_dest,
                balance - ed // make sure account doesn't die
            ),
            BadOrigin,
        );

        // root can
        assert_ok!(Membership::withdraw_fees(
            RuntimeOrigin::root(),
            fees_dest,
            balance - ed // make sure account doesn't die
        ),);
    })
}

/// Comprehensive test that imitates the lifecycle of the pallet
///
/// New club is created
/// 	New member added
/// 		- Normal flow where `member` has enough balance to pay for membership fees,
/// 		member's status is marked as `Paid`, marking `from` which block and `until` which block
/// 		the membership is valid
/// 	New member added
/// 		- `member` does not have enough funds, they should be marked as inactive
/// 	Member tries to extend his membership
/// 		- His membership is not expired yet, but this should not fail, simply extend the membership
/// 		from the current `until` value
/// 		- His membership is actually expired, then both `from` and `until` should be updated.
/// 		- His membership is in the future, then membership should be extended from `until`
///
#[test]
fn test_full_lifecycle() {
    new_test_ext().execute_with(|| {
        let club_name = b"AClub".to_vec().try_into().unwrap();
        let club_fee = 5;
        let club_owner = 2;

        let club_id = create_club(club_owner, club_name, club_fee);

        let member_name = b"AMember".to_vec().try_into().unwrap();
        let member = 3;
        let membership_years = 5;

        let block_number = System::block_number();
        add_member(club_id, member, member_name, membership_years);
        System::set_block_number(block_number + 1);

        // get member metadata
        let member_metadata = ClubMembers::<Test>::get(&club_id, &member).unwrap();

        assert_eq!(
            member_metadata.status,
            MemberStatus::Paid {
                until: block_number + YEARS * membership_years as u64,
            }
        );

        let new_club_name = b"NewClub".to_vec().try_into().unwrap();
        let new_club_fee = 1000;
        let new_club_owner = 6;
        let new_club_id = create_club(new_club_owner, new_club_name, new_club_fee);

        let new_member_name = b"NewMember".to_vec().try_into().unwrap();
        let new_member = 7;

        add_member(new_club_id, new_member, new_member_name, 10);

        // get member metadata
        let new_member_metadata = ClubMembers::<Test>::get(&new_club_id, &new_member).unwrap();

        // not enough funds to pay for the membership fee
        assert_eq!(new_member_metadata.status, MemberStatus::Inactive);

        System::set_block_number(block_number + 2);
        // new member now tries to activate himself, he has enough funds for 1 year (1000)
        Membership::extend_membership(
            RuntimeOrigin::signed(new_member),
            new_club_id,
            new_member,
            1,
        )
        .unwrap();

        let new_member_metadata = ClubMembers::<Test>::get(&new_club_id, &new_member).unwrap();

        // not enough funds to pay for the membership fee
        assert_eq!(
            new_member_metadata.status,
            MemberStatus::Paid {
                until: block_number + 2 + YEARS
            }
        );

        System::set_block_number(block_number + 3);

        // trying to extend membership that's not expired
        Membership::extend_membership(RuntimeOrigin::signed(member), club_id, member, 2).unwrap();

        let member_metadata_updated = ClubMembers::<Test>::get(&club_id, &member).unwrap();

        assert_eq!(
            member_metadata_updated.status,
            MemberStatus::Paid {
                until: member_metadata.status.until().unwrap() + 2 * YEARS, // `until` is extended though
            }
        );

        // set block number to expiration block
        System::set_block_number(member_metadata_updated.status.until().unwrap() + 1);
        let current_block_number = System::block_number();

        // this works
        Membership::extend_membership(RuntimeOrigin::signed(member), club_id, member, 2).unwrap();

        let member_metadata_latest = ClubMembers::<Test>::get(&club_id, &member).unwrap();
        assert_eq!(
            member_metadata_latest.status,
            MemberStatus::Paid {
                until: current_block_number + YEARS * 2,
            }
        );

        // owner can renew the membership, but `member` pays the fee
        Membership::extend_membership(
            RuntimeOrigin::signed(new_club_owner),
            new_club_id,
            new_member,
            1,
        )
        .unwrap();

        let new_member_metadata_latest =
            ClubMembers::<Test>::get(&new_club_id, &new_member).unwrap();

        assert_eq!(
            new_member_metadata_latest.status,
            MemberStatus::Paid {
                until: current_block_number + YEARS * 1
            }
        );
        let mut events = events();
        // just get rid of the last event
        events.pop().unwrap();

        let membership_pallet_pool: u64 = MembershipPalletId::get().into_account_truncating();
        assert_eq!(
            events.pop().unwrap(),
            RuntimeEvent::Balances(pallet_balances::Event::Transfer {
                from: new_member, // should be the `new_member`
                to: MembershipPalletId::get().into_account_truncating(),
                amount: 1 * new_club_fee
            })
        );

        let another_owner = 8;
        // Owner can transfer the club
        Membership::transfer_club(
            RuntimeOrigin::signed(new_club_owner),
            new_club_id,
            another_owner,
        )
        .unwrap();

        let club = Clubs::<Test>::get(new_club_id).unwrap();
        assert_eq!(club.owner, another_owner,);

        // New owner can now set the annual fee
        let new_fee = 100;
        Membership::set_annual_fee(RuntimeOrigin::signed(another_owner), new_club_id, new_fee)
            .unwrap();

        let pool_balance = Balances::free_balance(membership_pallet_pool);

        // Adding new member now charges new fee
        let final_member = 9;
        let final_member_name: Name = b"final".to_vec().try_into().unwrap();
        let years = 9;

        Membership::add_member(
            RuntimeOrigin::signed(another_owner),
            new_club_id,
            final_member,
            final_member_name,
            years,
        )
        .unwrap();

        let pool_balance_updated = Balances::free_balance(membership_pallet_pool);
        assert_eq!(pool_balance_updated, pool_balance + new_fee * years as u64,);
    })
}
