use core::clone;

use crate::{mock::*, Error, ContentStorage, Config, ContentAccessPolicy};
use frame_support::{assert_noop, assert_ok};
use frame_support::sp_runtime::print;

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(TribalModule::do_something(Origin::signed(1), 42));
		// Read pallet storage and assert an expected result.
		assert_eq!(TribalModule::something(), Some(42));


	});
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(TribalModule::cause_error(Origin::signed(1)), Error::<Test>::NoneValue);
	});
}

#[test]
fn create_content_ensure_exists() {
	new_test_ext().execute_with(|| {
		let test_fingerprint = b"OMG";
		let signed_origin = Origin::signed(7);

		assert_ok!(TribalModule::create_content(signed_origin.clone(), test_fingerprint.to_vec()));
		
		//let all_content =TribalModule::get_all_storage_for_content_owner(signed_origin.clone());

		let events =  System::events();
		let z = events.into_iter().map(|r| r.event).filter_map(|e| {
		 	// if let 

		 	if let Event::TribalModule(inner) = e {				
		 		Some(inner)
		 	} else {
		 		None
		 	}
		 }).next().unwrap();

		 println!("{:?}", z);		 

		 let pallet_event: crate::Event<Test> = z.try_into().unwrap();
		 println!("{:?}", pallet_event);


	});
}
