use crate::{mock::*, Club, ClubMemberMetadata, Error, Event, MemberStatus};
use frame_support::{assert_noop, assert_ok};

#[test]
fn non_root_create_club_fails() {
    new_test_ext().execute_with(|| {
        let owner = 2;
        let name = "AClub".try_into().unwrap();
        let fee = 100;
        // Non root call fails
        assert_noop!(
            Membership::create_club(RuntimeOrigin::signed(2), owner, name, fee),
            sp_runtime::DispatchError::BadOrigin.into()
        );
    });
}
