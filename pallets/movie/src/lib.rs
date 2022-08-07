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

	use frame_support::{pallet_prelude::*,BoundedVec,traits::{Randomness,Currency,ReservableCurrency}};
	use sp_io::hashing::blake2_128;
	use frame_system::pallet_prelude::*;
    use codec::{Decode, Encode, MaxEncodedLen};
    use sp_runtime::{RuntimeDebug, traits::{ AtLeast32BitUnsigned, CheckedAdd, One}};
    use scale_info::{TypeInfo};
    use scale_info::prelude::vec::Vec;
    use core::convert::TryInto;
	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		type MovieId: Member  + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
        /// The maximum length of base uri stored on-chain.
		#[pallet::constant]
		type StringLimit: Get<u32>;
		type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
		type MovieStringLimit:Get<u32>;
		type MovieCollateral: Get<BalanceOf<Self>>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

    
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen,TypeInfo)]
    pub struct Movie<AccountId,BoundedString> {
        pub	owner: AccountId,
        pub name:BoundedString,
        pub description: BoundedString,
		pub link:BoundedString,
            
    }

    #[pallet::storage]
	pub type Movies<T: Config> =
		StorageMap<_, Blake2_128Concat, BoundedVec<u8,T::MovieStringLimit>, Movie<T::AccountId,BoundedVec<u8, T::StringLimit>>>;

    #[pallet::storage]
	#[pallet::getter(fn next_movie_id)]
	pub(super) type NextMovieId<T: Config> = StorageValue<_, T::MovieId, ValueQuery>;

	
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
        MovieCreated(T::MovieId, T::AccountId),
		
	}
    #[pallet::hooks]
        impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NoAvailableMovieId,
		Overflow,
		Underflow,
        BadMetadata,
	}


	#[pallet::call]
	impl<T: Config> Pallet<T> {
		
		#[pallet::weight(10_000)]
		pub fn create_movie(
             origin: OriginFor<T>,
             name:Vec<u8>,
             synopsis:Vec<u8>,
             movie_description:Vec<u8>,
             classification:u32,
             release:Vec<u8>,
             director:Vec<u8>,
             lang:Vec<u8>,
             country:Vec<u8>,
             rating:u32,
             aspect_ratio:Vec<u8>,
             tags:Vec<u8>,
             trailer:Vec<u8>,
             imdb:Vec<u8>,
             social:Vec<u8>,
             ipfs:Vec<u8>,
             link:Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_create_movie(&who, name,synopsis,movie_description,classification,release,director,lang,country,rating,aspect_ratio,tags,trailer,imdb,social,ipfs,link)?;

			Ok(())
		}


		
	}

    impl<T: Config> Pallet<T> {


        pub fn do_create_movie(
             who: &T::AccountId,
             name:Vec<u8>,
             synopsis:Vec<u8>,
             movie_description:Vec<u8>,
             classification:u32,
             release:Vec<u8>,
             director:Vec<u8>,
             lang:Vec<u8>,
             country:Vec<u8>,
             rating:u32,
             aspect_ratio:Vec<u8>,
             tags:Vec<u8>,
             trailer:Vec<u8>,
             imdb:Vec<u8>,
             social:Vec<u8>,
             ipfs:Vec<u8>,
             link:Vec<u8>,
        ) -> Result<T::MovieId, DispatchError> {
    
            let movie_id =
                NextMovieId::<T>::try_mutate(|id| -> Result<T::MovieId, DispatchError> {
                    let current_id = *id;
                    *id = id
                        .checked_add(&One::one())
                        .ok_or(Error::<T>::Overflow)?;
                    Ok(current_id)
                })?;
    
            
            let bounded_name: BoundedVec<u8, T::StringLimit> =TryInto::try_into(name).map_err(|_| Error::<T>::BadMetadata)?;
            
            
            let bounded_synopsis: BoundedVec<u8, T::StringLimit> =
                TryInto::try_into(synopsis).map_err(|_|Error::<T>::BadMetadata)?;
            
            let bounded_movie_description: BoundedVec<u8, T::StringLimit> =
                TryInto::try_into(movie_description).map_err(|_| Error::<T>::BadMetadata)?;
            
            let bounded_release: BoundedVec<u8, T::StringLimit> =
                TryInto::try_into(release).map_err(|_| Error::<T>::BadMetadata)?;

            let bounded_director: BoundedVec<u8, T::StringLimit> =
                TryInto::try_into(director).map_err(|_| Error::<T>::BadMetadata)?;
            
            let bounded_lang: BoundedVec<u8, T::StringLimit> =
                TryInto::try_into(lang).map_err(|_| Error::<T>::BadMetadata)?;

            let bounded_country: BoundedVec<u8, T::StringLimit> =
                TryInto::try_into(country).map_err(|_| Error::<T>::BadMetadata)?;
            let bounded_aspect_ratio: BoundedVec<u8, T::StringLimit> =
                TryInto::try_into(aspect_ratio).map_err(|_| Error::<T>::BadMetadata)?;

            let bounded_tags: BoundedVec<u8, T::StringLimit> =
                TryInto::try_into(tags).map_err(|_| Error::<T>::BadMetadata)?;

            let bounded_trailer: BoundedVec<u8, T::StringLimit> =
                TryInto::try_into(trailer).map_err(|_|Error::<T>::BadMetadata)?;
            
            let bounded_imdb: BoundedVec<u8, T::StringLimit> =
                TryInto::try_into(imdb).map_err(|_|Error::<T>::BadMetadata)?;

            let bounded_social: BoundedVec<u8, T::StringLimit> =
                TryInto::try_into(social).map_err(|_|Error::<T>::BadMetadata)?;

            let bounded_link: BoundedVec<u8, T::StringLimit> =
                TryInto::try_into(link).map_err(|_|Error::<T>::BadMetadata)?;

            let bounded_ipfs: BoundedVec<u8, T::StringLimit> =
               TryInto::try_into(ipfs).map_err(|_|Error::<T>::BadMetadata)?;

            
            let movie = Movie {
                owner:who.clone(),
                name:bounded_name,
                description:bounded_movie_description,
                link:bounded_ipfs,
            };
    
            let bounded_id = Self::random_value(&movie_id);

			//ReservableCurrency::reserve(who,T::MovieCollateral::get())?;

			Movies::<T>::insert(bounded_id, movie.clone());
    
            Self::deposit_event(Event::MovieCreated(movie_id, who.clone()));
            Ok(movie_id)
        } 

	 fn random_value(nonce: &T::MovieId) -> BoundedVec<u8,T::MovieStringLimit> {
             let payload = (
                   T::Randomness::random_seed().0,
				   &nonce,
                   <frame_system::Pallet<T>>::extrinsic_index(),
             );
			 let aux : BoundedVec<u8,T::MovieStringLimit> =
                   payload.using_encoded(blake2_128).to_vec().try_into().map_err(|_| Error::<T>::BadMetadata).unwrap();

            aux
		}

}
}
