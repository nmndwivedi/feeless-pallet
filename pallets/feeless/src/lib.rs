#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

pub use pallet::*;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{Twox128, dispatch::{DispatchResult, GetDispatchInfo}, pallet_prelude::*, weights::Pays};
	use frame_system::pallet_prelude::*;
	use sp_std::prelude::*;
	use sp_runtime::traits::Dispatchable;

	type Count = u32;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Call: Parameter + Dispatchable<Origin=Self::Origin> + GetDispatchInfo;
	}


	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn user_calls)]
	pub type UserCalls<T: Config> = StorageMap<_, Twox128, T::AccountId, Count>;

	#[pallet::storage]
	#[pallet::getter(fn session)]
	pub type Session<T: Config> = StorageValue<Value = T::BlockNumber, QueryKind = ValueQuery>;

	#[pallet::type_value]
	pub(super) fn SessionLengthDefault<T: Config>() -> T::BlockNumber { 1000u32.into() }
	#[pallet::storage]
	#[pallet::getter(fn session_length)]
	pub(super) type SessionLength<T: Config> = StorageValue<Value = T::BlockNumber, QueryKind = ValueQuery, OnEmpty = SessionLengthDefault<T>>;

	#[pallet::type_value]
	pub(super) fn MaxCallsDefault<T: Config>() -> u32 { 100u32 }
	#[pallet::storage]
	#[pallet::getter(fn max_calls)]
	pub(super) type MaxCalls<T: Config> = StorageValue<Value = u32, QueryKind = ValueQuery, OnEmpty = MaxCallsDefault<T>>;
 


	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]

	pub enum Event<T: Config> {
		ExtrinsicResult(T::AccountId, DispatchResult),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		CallQuotaExhausted,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T:Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight({
			let dispatch_info = call.get_dispatch_info();
			(dispatch_info.weight + T::DbWeight::get().reads_writes(4, 1), dispatch_info.class, Pays::Yes)
		})]
		pub fn make_feeless(origin: OriginFor<T>, call: Box<<T as Config>::Call>) -> DispatchResultWithPostInfo {
			let user = ensure_signed(origin.clone())?;
			
			let block_number = frame_system::Pallet::<T>::block_number();
			let last_session = Session::<T>::get();
			let session_len = SessionLength::<T>::get();
			
			let mut user_calls = UserCalls::<T>::get(&user).unwrap_or(0);
			let max_calls = Pallet::<T>::max_calls();

			if block_number - last_session > session_len {
				let current_session = (block_number / session_len) * session_len;
				Session::<T>::put(current_session);
				user_calls = 0;
			}

			if user_calls >= max_calls { //, Error::<T>::CallQuotaExhausted);
				let check_weight = T::DbWeight::get().reads(4);
				return Ok(Some(check_weight).into());
			}

			UserCalls::<T>::insert(&user, user_calls.checked_add(1).ok_or("Value overflow")?);

			let r1 = call.dispatch(origin);
			// let _r2 = call.get_dispatch_info();

			Pallet::<T>::deposit_event(Event::<T>::ExtrinsicResult(user, r1.map(|_| ()).map_err(|e| e.error)));

			Ok(Pays::No.into())
		}
	}
}
