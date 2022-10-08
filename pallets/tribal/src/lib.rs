#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	
	use core::marker::PhantomData;
	use frame_support::{pallet_prelude::*, PalletId};
	use frame_system::{pallet_prelude::*};
	use frame_support::inherent::{Vec};
	use frame_support::sp_runtime::print;
	use frame_support::traits::{UnixTime, Randomness};	
	use nuuid;
	
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo)]
	pub struct ContentItem {
		content_key: Vec<u8>,
		fingerprint: Vec<u8>,
		block_number: Vec<u8>,
		created_date: u128
	}

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo)]
	pub struct LeaseItem {
		content_info: ContentItem,
		lease_date: u128
	}

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo)]
	pub enum ContentAccessPolicy {
		NotAccessible,
		ContentOwner,
		ContentLeaseAssigned,
		ContentLeaseRevoked,
	}

	impl Default for ContentAccessPolicy {
		fn default() -> Self { ContentAccessPolicy::NotAccessible }
	}
	
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {

		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type TimeProvider: UnixTime;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// content storage
	#[pallet::storage]
	#[pallet::getter(fn content_storage)]
	pub(super) type ContentStorage<T: Config> = StorageDoubleMap<
		_,		
		Blake2_128Concat, 	// hasher type 		
		T::AccountId, 		// key of map

		Blake2_128Concat,	// THIS IS THE CONTENT KEY!!
		Vec<u8>,
		
		ContentItem, 		// the content item itself		
		ValueQuery			// type of return value
	>;

	// lease storage 
	#[pallet::storage]
	#[pallet::getter(fn content_access_storage_user)]
	pub(super) type ContentAccessStorageByAccount<T: Config> = StorageDoubleMap<
		_, 

		Blake2_128Concat, 		// User record
		T::AccountId,

		Blake2_128Concat, 		// fingerprint
		Vec<u8>, 
		
		ContentAccessPolicy,	// Access Type
		ValueQuery
	>;

	// Pallets use events to inform users when important changes are made.	
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event parameters. [something, who]
		SomethingStored(u32, T::AccountId),

		/// ContentKey generated for create content extrensic. [content_key, who]
		CreateContentKey(Vec<u8>, T::AccountId),
		
		/// ContentAccessPolicy generated to determine access to a specific content. [who, fingerprint, policy]
		ContentAccessPolicyChange(T::AccountId, Vec<u8>, ContentAccessPolicy),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// A user attempted to upload an identical content 
		CouldNotDetermineBlockNumber,

		ContentNotFound,

		ContentNotAccessible,

		ContentAlreadyAccessibleByAccount
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(10_000)]
		pub fn create_content(origin: OriginFor<T>, fingerprint: Vec<u8>) -> DispatchResult {

			// ensure transaction is signed
			let who = ensure_signed(origin.clone())?;
			let time = T::TimeProvider::now().as_nanos();

			//// generate this first, so the time between validate and write to storage is fast
			let content_key: Vec<u8> = Self::generate_content_key(time);

			let mut bn: Vec<u8> = <frame_system::Pallet<T>>::block_number().encode();
			bn.reverse();

			// write ContentKey to chain as an event of this extrensic, and store it.
			<ContentStorage<T>>::insert(who.clone(), content_key.clone(), ContentItem {
				content_key: content_key.clone(),
				fingerprint: fingerprint.clone(),
				block_number: bn,
				created_date: time
			});
			Self::deposit_event(Event::CreateContentKey(content_key.clone(), who.clone()));
			
			// assign content access policy to content owner (for now, its just the content creator)
			<ContentAccessStorageByAccount<T>>::insert(
				who.clone(),
				content_key.clone(),
				ContentAccessPolicy::ContentOwner
			);
			Self::deposit_event(Event::ContentAccessPolicyChange(who.clone(), content_key.clone(), ContentAccessPolicy::ContentOwner));
			Ok(())
		}

		

		#[pallet::weight(10_000)]
		pub fn lease_content(origin: OriginFor<T>, content_key: Vec<u8>, tribe_public_key: T::AccountId) -> DispatchResult {

			let who = ensure_signed(origin)?;

			let _content = match <ContentStorage<T>>::try_get(who.clone(), content_key.clone()) {
				Ok(v) => v,
				Err(_) => {
					return Err(Error::<T>::ContentNotFound.into())
				}
			};

			


            // is there an active lease against this tribe for this content key?
			let _lease = match <ContentAccessStorageByAccount<T>>::try_get(tribe_public_key.clone(), content_key.clone()) {
				Ok(v) => {
					if v == ContentAccessPolicy::NotAccessible {
						return Err(Error::<T>::ContentNotAccessible.into())
					}
					return Err(Error::<T>::ContentAlreadyAccessibleByAccount.into())
				},
				Err(_) => false
			};

			// by now, we're ok with creating the lease.
			<ContentAccessStorageByAccount<T>>::insert(tribe_public_key.clone(), content_key.clone(), ContentAccessPolicy::ContentLeaseAssigned);
			Self::deposit_event(Event::ContentAccessPolicyChange(tribe_public_key.clone(), content_key.clone(), ContentAccessPolicy::ContentLeaseAssigned));

            // generate signature of content_key using contract's identity (tribe_content_signature)

            // call content server with content_signature to symlink with tribe_content_signature
			Ok(())
		}


		// todo 
		// #[pallet::weight(10_000)]
		// pub fn revoke_content_lease(origin: OriginFor<T>, content_key: Vec<u8>, tribe_public_key: Vec<u8>) -> DispatchResult {


		// 	Ok(())
		// }

		
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}
	}

	impl <T: Config> Pallet<T> {
		fn generate_random_number(seed: u128) -> u32 {
			let (random_seed, _) = T::Randomness::random(&(T::PalletId::get(), seed).encode());
			let random_number = <u32>::decode(&mut random_seed.as_ref())
				.expect("secure hashes should always be bigger than u32; qed");
			random_number
		}

		/*
		pub fn get_all_storage_for_content_owner(origin: OriginFor<T>) -> Vec<ContentStorage<T>> {
			let who = ensure_signed(origin);

			let matches = Vec::<ContentStorage<T>>::new();
			// let results = <ContentStorage<T>>::iter_keys().filter_map(|key {
			// 	key
			// });

			matches
		}
		*/

		fn generate_content_key(seed: u128) -> Vec<u8> {
			let content_key_seed = Self::generate_random_number(seed).to_le_bytes();
			let content_key: Vec<u8> = nuuid::Uuid::new_v5(nuuid::NAMESPACE_DNS, &content_key_seed).to_bytes().to_vec();
			content_key
		}
	}
}
