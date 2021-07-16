#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{Twox128, dispatch::{DispatchResult, GetDispatchInfo}, pallet_prelude::*, weights::Pays};
	use frame_system::pallet_prelude::*;
	use sp_std::prelude::*;
	use sp_std::convert::TryInto;
	use sp_runtime::traits::Dispatchable;

	type CallCount = u32;
	type Session<T> = BlockNumberFor<T>;

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
	#[pallet::getter(fn user_record)]
	pub type UserRecord<T: Config> = StorageMap<_, Twox128, T::AccountId, (Session<T>, CallCount)>;

	#[pallet::type_value]
	pub(super) fn SessionLengthDefault<T: Config>() -> Session<T> { 1000u32.into() }
	#[pallet::storage]
	#[pallet::getter(fn session_length)]
	pub(super) type SessionLength<T: Config> = StorageValue<Value = Session<T>, QueryKind = ValueQuery, OnEmpty = SessionLengthDefault<T>>;

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
		CallQuotaExhausted(T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
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
			(dispatch_info.weight + T::DbWeight::get().reads_writes(3, 1), dispatch_info.class, Pays::Yes)
		})]
		pub fn make_feeless(origin: OriginFor<T>, call: Box<<T as Config>::Call>) -> DispatchResultWithPostInfo {
			let user = ensure_signed(origin.clone())?;
			
			let block_number = TryInto::<u64>::try_into(frame_system::Pallet::<T>::block_number()).unwrap_or_default();
			let session_len = TryInto::<u64>::try_into(SessionLength::<T>::get()).unwrap_or_default();
			
			let (last_session, mut user_calls) = UserRecord::<T>::get(&user).unwrap_or((0u32.into(), 0));
			let max_calls = Pallet::<T>::max_calls();

			let last_session = TryInto::<u64>::try_into(last_session).unwrap_or_default();

			let current_session = block_number.checked_div(session_len).ok_or("Division failed")?.checked_mul(session_len).ok_or("Multiplication failed")?;

			if block_number.checked_sub(last_session).unwrap_or_default() > session_len {
				user_calls = 0;
			}

			if user_calls >= max_calls {
				let check_weight = T::DbWeight::get().reads(3);
				Pallet::<T>::deposit_event(Event::<T>::CallQuotaExhausted(user));
				return Ok(Some(check_weight).into());
			}

			let current_session: Session<T> = (current_session as u32).into();

			UserRecord::<T>::insert(&user, (current_session, user_calls.checked_add(1).ok_or("Value overflow")?));

			let r1 = call.dispatch(origin);
			// let _r2 = call.get_dispatch_info();

			Pallet::<T>::deposit_event(Event::<T>::ExtrinsicResult(user, r1.map(|_| ()).map_err(|e| e.error)));

			Ok(Pays::No.into())
		}
	}
}
