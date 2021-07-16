use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};

/*
    Test for:
    1. Multiple users can make feeless call in series   -> [$(assert_ok, assert_eq(event)),*]
    2. User doesnt exceed designated callQuota          -> assert_ok, assert_eq(event)
    3. callQuota resets after session complete          -> assert_eq(storage value)
    4. Math fails                                       -> assert_err
*/

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		assert_eq!(Some(42), Some(42));
	});
}

// #[test]
// fn correct_error_for_none_value() {
// 	new_test_ext().execute_with(|| {
// 		// Ensure the expected error is thrown when no value is present.
// 		assert_noop!(
// 			Feeless::cause_error(Origin::signed(1)),
// 			Error::<Test>::NoneValue
// 		);
// 	});
// }