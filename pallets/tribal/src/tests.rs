use crate::{mock::*, Error, ContentStorage};
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
		assert_ok!(TribalModule::create_content(Origin::signed(7), test_fingerprint.to_vec()));
		println!("{}", System::events().len());
		// for event in System::events() {
		// 	println!("{:?}", event.event);


		// }

		// let z = System::events().into_iter().map(|r| r.event).filter_map(|e| {
		// 	// if let 
		// 	if let Event::TribalModule(inner) = e {
		// 		Some(inner)
		// 	} else {
		// 		None
		// 	}
		// }).next().unwrap();

		// TribalModule::deposit_event(z)
		// println!("{:?}", z);
		<<Test as Config>::



		
		
		// let val1 = &System::events()[0].event;
			

	});
}
