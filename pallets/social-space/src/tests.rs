use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};

#[test]
fn test_create_space() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.

		assert_ok!(SocialModule::create_space(Origin::signed(1),vec![0u8,10],vec![0u8,10]));
		// Read pallet storage and assert an expected result.
		//assert_eq!(TemplateModule::something(), Some(42));
	});
}

#[test]
fn error_string_limit_space() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		let my_string: &str = "some stringasdshadsabdjasbdhsagduasgduyasgdyuagsdyugasyudgasuydasyudgasyugdyuasgduysagduyasgdyuasgduyasgdyuagsuydgasyudgasuygduygadsdgasuygdsauya";

	let my_bytes: Vec<u8> = my_string.as_bytes().to_vec();

		assert_noop!(
		SocialModule::create_space(Origin::signed(1).into(),my_bytes,vec![0u8,255]),
		Error::<Test>::BadMetadata
		);
	});
}


#[test]
fn test_create_post() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		
		SocialModule::create_space(Origin::signed(2),vec![0u8,10],vec![0u8,10]);
		assert_ok!(SocialModule::create_post(Origin::signed(2),vec![0u8,10],vec![0u8,10],vec![0u8,10]));
		// Read pallet storage and assert an expected result.
		//assert_eq!(TemplateModule::something(), Some(42));
	});
}

#[test]
fn error_create_post_without_space() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(
		SocialModule::create_post(Origin::signed(1).into(),vec![0u8,10],vec![0u8,255],vec![0u8,10]),
		Error::<Test>::InexistentProfile
		);
	
	});
}

