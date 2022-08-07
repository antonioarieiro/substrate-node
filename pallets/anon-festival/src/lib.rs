#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;


#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*,traits::{Randomness,Currency,ReservableCurrency,ExistenceRequirement::{KeepAlive,AllowDeath}},PalletId};
	use frame_system::pallet_prelude::*;
  //  use rsa::{RsaPrivateKey,BigUint,RsaPublicKey,PaddingScheme,PublicKey,pkcs1::FromRsaPublicKey};
    use frame_support::sp_std::{vec,collections::btree_map::BTreeMap};
    use codec::{Decode, Encode, MaxEncodedLen};
    use scale_info::TypeInfo;
    use frame_support::BoundedVec;
    use sp_runtime::{sp_std::str,RuntimeDebug, traits::{AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, One}};
    use frame_support::inherent::Vec;
    use core::convert::TryInto;
	use sp_io::hashing::blake2_128;
	use pallet_movie;
type BalanceOf<T> = <<T as pallet_movie::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_movie::Config{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type PollingStationId: Member  + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
        type AnonFestivalId: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
	    type AnonVoteId: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
		type VoteStringLimit : Get<u32>;
		type MinFestivalDuration : Get<Self::BlockNumber>;
		type CooldownPeriod: Get<Self::BlockNumber>;
		type MaxDecryptionPeriod: Get<Self::BlockNumber>;
		/// The minimum balance to become a station member (slashable)
		#[pallet::constant]
		type MinMemberSlashAmount: Get<BalanceOf<Self>>;

		#[pallet::constant]
		type StationMemberPayout: Get<BalanceOf<Self>>;
		type PalletId : Get<PalletId>;

		//type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;	
 }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen,TypeInfo)]
    pub enum AnonFestivalStatus{
        RecentlyCreated,
        WaitingForStationMember,
        Open,
        OnDecryption,
        Counting,
        NormalFinish,
        AbnormalFinish,
    }

    
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen,TypeInfo)]
    pub enum AnonVoteStatus{
        Encrypted,
        Decrypted,
    }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen,TypeInfo)]
    pub struct AnonFestival<AccountId,BoundedString,BalanceOf> {
        pub creator : AccountId,
        pub name : BoundedString,
		pub description : BoundedString,
        pub status : AnonFestivalStatus,
		pub min_entry: BalanceOf
    }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen,TypeInfo)]
    pub struct AnonVote<AccountId,AnonFestivalId, PoolingStationId,AnonVoteStatus,Vec,BalanceOf>{
        pub owner : AccountId,
		pub owner_pub_key: Vec,
        pub anon_fest_id : AnonFestivalId,
        pub pooling_station : PoolingStationId,
        pub delegated_encryptor : AccountId,
        pub enc_vote : Vec,
		pub raw_vote : Vec,
        pub vote_status: AnonVoteStatus,
		pub amount: BalanceOf,
    }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen,TypeInfo)]
    pub struct PollingStation<AccountId,BoundedVec,BoundedString> {
        pub owner: AccountId,
        pub anon_fest_id : BoundedVec,
        pub name:BoundedString,
        pub description:BoundedString
    }

	#[pallet::storage]
	#[pallet::getter(fn festival_start_time)]
	/// Index auctions by end time.
	pub type FestivalStartTime<T: Config> =
		StorageDoubleMap<_,
		Blake2_128Concat, T::BlockNumber,
		Blake2_128Concat, T::AnonFestivalId,
		()
		>;

    #[pallet::storage]
	#[pallet::getter(fn festival_end_time)]
	/// Index auctions by end time.
	pub type FestivalEndTime<T: Config> =
		StorageDoubleMap<_,
		Blake2_128Concat,T::BlockNumber,
		Blake2_128Concat,T::AnonFestivalId,
		()
		>;
	
	#[pallet::storage]
	#[pallet::getter(fn festival_expiration)]
	/// Index auctions by end time.
	pub type FestivalExpiration<T: Config> =
		StorageDoubleMap<_,
		Blake2_128Concat, T::BlockNumber,
		Blake2_128Concat, T::AnonFestivalId,
		()
		>;

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

    #[pallet::storage]
	pub type PollingStations<T: Config> =
		StorageMap<_, Blake2_128Concat, T::PollingStationId, PollingStation<T::AccountId,T::AnonFestivalId,BoundedVec<u8, T::StringLimit>>>;
    
    
    #[pallet::storage]
	pub type PollingStationMembers<T: Config> =
		StorageDoubleMap<_, 
            Blake2_128Concat, T::PollingStationId, 
            Blake2_128Concat, T::AccountId,
            Vec<u8>     >;
    
    #[pallet::storage]
	pub type AnonFestivals<T: Config> =
		StorageMap<_, 
            Blake2_128Concat, T::AnonFestivalId, 
            AnonFestival<T::AccountId,BoundedVec<u8,T::StringLimit>,BalanceOf<T>>>;
  
    #[pallet::storage]
	pub type AnonVotes<T: Config> =
		StorageDoubleMap<_,
						Blake2_128Concat, T::AnonFestivalId,
                        Blake2_128Concat, T::AnonVoteId,
                        AnonVote<T::AccountId,T::AnonFestivalId,T::PollingStationId,AnonVoteStatus,Vec<u8>,BalanceOf<T>>>;

	
    #[pallet::storage]
	pub type DecryptedVotes<T: Config> =
		StorageDoubleMap<_,
						Blake2_128Concat, T::AnonFestivalId,
                        Blake2_128Concat, (T::AnonVoteId,Vec<u8>),
						BalanceOf<T>,
						ValueQuery
						>;
    #[pallet::storage]
    pub type MoviesInFestival<T:Config>=
        StorageDoubleMap<_,
        Blake2_128Concat,T::AnonFestivalId,
        Blake2_128Concat,BoundedVec<u8,T::MovieStringLimit>,
        ()
         >;

	#[pallet::storage]
    #[pallet::getter(fn owned_festivals)]
    pub type OwnedFestivals<T:Config> =
        StorageMap<_,
        Blake2_128Concat, T::AccountId,
         T::AnonFestivalId,
        >;

    #[pallet::storage]
	#[pallet::getter(fn next_polling_station_id)]
	pub(super) type NextPollingStationId<T: Config> = StorageValue<_, T::PollingStationId, ValueQuery>;
    
    #[pallet::storage]
	#[pallet::getter(fn next_anon_festival_id)]
	pub(super) type NextAnonFestivalId<T: Config> = StorageValue<_, T::AnonFestivalId, ValueQuery>;
	

    #[pallet::storage]
	#[pallet::getter(fn next_anon_vote_id)]
	pub(super) type NextAnonVoteId<T: Config> = StorageValue<_, T::AnonVoteId, ValueQuery>;
    
    #[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SomethingStored(u32, T::AccountId),
		MovieAddedToFestival(T::AnonFestivalId,BoundedVec<u8,T::MovieStringLimit>,T::AccountId),
		FestivalCreated(T::AnonFestivalId,T::AccountId),
	}

	#[pallet::storage]
    #[pallet::getter(fn total_lockup)]
    pub type TotalLockup<T:Config>=
        StorageMap<_,
        Blake2_128Concat, T::AnonFestivalId,
        BalanceOf<T>,ValueQuery>;

	#[pallet::error]
	pub enum Error<T> {
		NoneValue,
		StorageOverflow,
	    BadMetadata,
        Overflow,
        InvalidFestivalId,
        AnonFestivalNotFound,
		FestivalNotFound,
		MovieNotFound,
		MovieAlreadyInFestival,
		PastStartDate,
		FestivalPeriodTooShort,
		InsuficientBalance,
		FestivalNotActive,
		InvalidVote,
		NoPermission,
		AlreadyDecrypted,
		
		
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
            
		fn on_initialize(now: T::BlockNumber) -> Weight {

			10_000
	   }
	  
	  fn on_finalize(now: T::BlockNumber){
		Self::conclude_festival(now).unwrap();
        
		Self::activate_festival(now).unwrap();
		
		Self::count_votes(now).unwrap();
		
            
         }
        

      }

	#[pallet::call]
	impl<T:Config> Pallet<T> {

        #[pallet::weight(10_000)]
		pub fn create_polling_station(
             origin: OriginFor<T>,
             anon_festival_id: T::AnonFestivalId,
             name:Vec<u8>,
             description:Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_create_polling_station(&who,anon_festival_id, name,description)?;

            Ok(())

		}
        #[pallet::weight(10_000)]
        pub fn create_anon_festival(
            origin: OriginFor<T>,
            name: Vec<u8>,
			description:Vec<u8>,
            min_entry: BalanceOf<T>,
            start_block: T::BlockNumber,
            end_block: T::BlockNumber,
        )-> DispatchResult{
            let who = ensure_signed(origin)?;

            Self::do_create_anon_festival(&who,name,description,min_entry,start_block,end_block)?;
            Ok(())
        }
		#[pallet::weight(10_000)]
		pub fn add_movie(
			origin: OriginFor<T>,
			anon_fest_id: T::AnonFestivalId,
			movie_id : Vec<u8>,
			)-> DispatchResult{

				Ok(())
			}
			

        #[pallet::weight(10_000)]
        pub fn add_station_member(
            origin: OriginFor<T>,
            station_id: T::PollingStationId,
            pub_key_hex_string: Vec<u8>,
			member_collateral: BalanceOf<T>,
        )-> DispatchResult{
            let who = ensure_signed(origin)?;
            
            ensure!(PollingStations::<T>::contains_key(station_id),Error::<T>::BadMetadata);
			ensure!(!PollingStationMembers::<T>::contains_key(station_id,who.clone()),Error::<T>::BadMetadata);
			ensure!(member_collateral >= T::MinMemberSlashAmount::get(),Error::<T>::InsuficientBalance);
			
				   
            let b : [u8;32] = pub_key_hex_string.clone().try_into().unwrap();

			T::Currency::reserve(&who,member_collateral)?;
            
			//let pk = PublicKey::decode(&b).unwrap();
            
            PollingStationMembers::<T>::insert(station_id,who,pub_key_hex_string);

            Ok(())
        }

		#[pallet::weight(10_000)]
		pub fn decrypt_vote(
			origin: OriginFor<T>,
			fest_id: T::AnonFestivalId,
			vote_id: T::AnonVoteId,
			raw_vote: Vec<u8>,
		)-> DispatchResult{

			let who = ensure_signed(origin)?;

			AnonVotes::<T>::try_mutate_exists(fest_id,vote_id,|vote|->DispatchResult{


				let anon_vote = vote.as_mut().ok_or(Error::<T>::InvalidVote)?;

				ensure!(who == anon_vote.delegated_encryptor,Error::<T>::NoPermission);
				ensure!(anon_vote.vote_status == AnonVoteStatus::Encrypted, Error::<T>::AlreadyDecrypted);

				anon_vote.raw_vote = raw_vote.clone();

				anon_vote.vote_status = AnonVoteStatus::Decrypted;

				DecryptedVotes::<T>::insert(fest_id,(vote_id,raw_vote),anon_vote.amount);

				Ok(())

			})


		}

        #[pallet::weight(10_000)]
        pub fn vote2(
            origin: OriginFor<T>,
            anon_fest_id: T::AnonFestivalId,
            polling_station_id: T::PollingStationId,
			delegated_encryptor: T::AccountId,
			pub_key_hex: Vec<u8>,
            encrypted_vote: Vec<u8>,
			amount: BalanceOf<T>,
            )-> DispatchResult{
			
				
            let who = ensure_signed(origin)?;

            let b : [u8;32] = pub_key_hex.clone().try_into().unwrap();

			AnonFestivals::<T>::try_mutate_exists(anon_fest_id.clone(), |fest| -> DispatchResult{
				
				let mut aux = fest.as_mut().ok_or(Error::<T>::FestivalNotFound)?;

				ensure!(aux.status == AnonFestivalStatus::Open,Error::<T>::FestivalNotActive);

				ensure!(PollingStations::<T>::contains_key(polling_station_id.clone()),Error::<T>::InvalidFestivalId);


				let anon_vote_id = 
					NextAnonVoteId::<T>::try_mutate(|id| -> Result<T::AnonVoteId, DispatchError> {
						let current_id = *id;
						*id = id
                        .checked_add(&One::one())
                        .ok_or(Error::<T>::Overflow)?;
                    Ok(current_id)
                })?;

				
			let anon_vote = AnonVote {
                owner:who.clone(),
				owner_pub_key: pub_key_hex.clone(),
                anon_fest_id: anon_fest_id,
                pooling_station:polling_station_id.clone(),
                delegated_encryptor: delegated_encryptor.clone(),
                enc_vote:encrypted_vote.clone(),
				raw_vote:encrypted_vote,
                vote_status: AnonVoteStatus::Encrypted,
				amount:amount
			};

			//T::Currency::transfer(&who,&delegated_encryptor,T::StationMemberPayout::get(),AllowDeath)?;


			T::Currency::transfer(&who.clone(),	&delegated_encryptor,T::StationMemberPayout::get(),AllowDeath)?;

			TotalLockup::<T>::try_mutate(anon_fest_id,|total_festival_locked_amnt|->DispatchResult{

                    *total_festival_locked_amnt+=amount.clone();

					let treasury = Self::account_id();

					T::Currency::transfer(&who,&Self::account_id(),amount.clone(),AllowDeath)?;
					//pallet_balances::Pallet::<T>::transfer(origin,treasury.into(),T::StationMemberPayout::get().into())?;

					AnonVotes::<T>::insert(anon_fest_id,anon_vote_id,anon_vote);
                    Ok(())

                })?;


				Ok(())
            


			})


			}


        #[pallet::weight(10_000)]
        pub fn vote(
            origin: OriginFor<T>,
            anon_fest_id: T::AnonFestivalId,
            polling_station_id: T::PollingStationId,
			delegated_encryptor: T::AccountId,
			pub_key_hex: Vec<u8>,
            encrypted_vote: Vec<u8>,
			amount: BalanceOf<T>,
            )-> DispatchResult{
            let who = ensure_signed(origin)?;
			

			AnonFestivals::<T>::try_mutate_exists(anon_fest_id.clone(),|fest| -> DispatchResult{
			
			let mut aux=fest.as_mut().ok_or(Error::<T>::FestivalNotFound)?;

			//ensure status == active

			ensure!(aux.status == AnonFestivalStatus::Open,Error::<T>::FestivalNotActive);

            ensure!(PollingStations::<T>::contains_key(polling_station_id.clone()),Error::<T>::InvalidFestivalId);
            
            //ensure!(PollingStationMembers::<T>::iter().count() > 0 , Error::<T>::Overflow);
            
            let b : [u8;32] = pub_key_hex.clone().try_into().unwrap();

            let anon_vote_id = 
                NextAnonVoteId::<T>::try_mutate(|id| -> Result<T::AnonVoteId, DispatchError> {
                    let current_id = *id;
                    *id = id
                        .checked_add(&One::one())
                        .ok_or(Error::<T>::Overflow)?;
                    Ok(current_id)
                })?;
            
			ensure!(PollingStationMembers::<T>::contains_key(polling_station_id.clone(),delegated_encryptor.clone()),Error::<T>::Overflow);

			let anon_vote = AnonVote {
                owner:who.clone(),
				owner_pub_key: pub_key_hex.clone(),
                anon_fest_id: anon_fest_id,
                pooling_station:polling_station_id.clone(),
                delegated_encryptor: delegated_encryptor.clone(),
                enc_vote:encrypted_vote.clone(),
				raw_vote:encrypted_vote,
                vote_status: AnonVoteStatus::Encrypted,
				amount:amount
            };

			//Transfer to delegated encryptor reserve
			//T::Currency::transfer(&who,&delegated_encryptor,T::StationMemberPayout::get(),KeepAlive)?;

			//Transfer to amout to anon-fest-treasury
		/*
			ensure!(
				T::Currency::transfer(
					&who,
					&Self::account_id(),
					amount,
					KeepAlive,
				)==Ok(()),Error::<T>::InsuficientBalance);

		*/		TotalLockup::<T>::try_mutate(anon_fest_id,|total_festival_locked_amnt|->DispatchResult{

                    *total_festival_locked_amnt+=amount.clone();

					AnonVotes::<T>::insert(anon_fest_id,anon_vote_id,anon_vote);
                    Ok(())

                })

			})
        }
    }

      impl<T: Config> Pallet<T> {
		
		fn random_value(sender: &T::AccountId) -> BoundedVec<u8,T::VoteStringLimit> {
			let payload = (
				T::Randomness::random_seed().0,
				&sender,
				<frame_system::Pallet<T>>::extrinsic_index(),
			);
			let aux : BoundedVec<u8,T::VoteStringLimit> = 
				payload.using_encoded(blake2_128).to_vec().try_into().map_err(|_| Error::<T>::BadMetadata).unwrap();
			
			aux
		}

        pub fn do_create_anon_festival(
             who: &T::AccountId,
             name:Vec<u8>,
			 description:Vec<u8>,
			  min_ticket_price:BalanceOf<T>,
             start_block : T::BlockNumber,
             end_block: T::BlockNumber
        ) -> Result<T::AnonFestivalId,DispatchError>{
            
            let anon_fest_id = 
                NextAnonFestivalId::<T>::try_mutate(|id| -> Result<T::AnonFestivalId, DispatchError> {
                    let current_id = *id;
                    *id = id
                        .checked_add(&One::one())
                        .ok_or(Error::<T>::Overflow)?;
                    Ok(current_id)
                })?;

            let bounded_name: BoundedVec<u8, T::StringLimit> =
                name.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;

			 let bounded_description: BoundedVec<u8, T::StringLimit> =
                description.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;	
            
            let anon_fest = AnonFestival {
                creator:who.clone(),
                name:bounded_name,
				description: bounded_description,
				min_entry: min_ticket_price,
                status: AnonFestivalStatus::RecentlyCreated,
             };


			OwnedFestivals::<T>::try_mutate(who.clone(),|fest_id|-> Result<T::AnonFestivalId,DispatchError>{

				
                *fest_id = Some(anon_fest_id);

               
                AnonFestivals::<T>::insert(anon_fest_id, anon_fest.clone());
                
                //verificar datas!!!!!!! TODO

                FestivalStartTime::<T>::insert(start_block,anon_fest_id,());

                //ensure now > start
                ensure!(frame_system::Pallet::<T>::block_number() < start_block,Error::<T>::PastStartDate);

                //ensure (end-start)>MinAlivePeriod
                ensure!(end_block-start_block >= T::MinFestivalDuration::get(), Error::<T>::FestivalPeriodTooShort);

                FestivalEndTime::<T>::insert(end_block,anon_fest_id,());
				FestivalExpiration::<T>::insert(end_block+T::MaxDecryptionPeriod::get(),anon_fest_id,());
				

                Self::deposit_event(Event::FestivalCreated(anon_fest_id, who.clone()));
                Ok(anon_fest_id)
           
            })
    
		}

		pub fn do_add_movie(
			who : &T::AccountId,
			anon_fest_id: T::AnonFestivalId,
			movie_id : Vec<u8>,

		) -> DispatchResult {
			
		let bounded_movie_id: BoundedVec<u8, T::MovieStringLimit> =
          TryInto::try_into(movie_id).map_err(|_| Error::<T>::BadMetadata)?;
       
        //ensure festival exists
        ensure!(AnonFestivals::<T>::contains_key(anon_fest_id.clone()),Error::<T>::FestivalNotFound);

        //TODO ensure movie exists
        ensure!(pallet_movie::Movies::<T>::contains_key(bounded_movie_id.clone()),Error::<T>::MovieNotFound);
        
        //ensure movie is not in list already
        ensure!(!MoviesInFestival::<T>::contains_key(anon_fest_id.clone(),bounded_movie_id.clone()),Error::<T>::MovieAlreadyInFestival);
        
        MoviesInFestival::<T>::insert(anon_fest_id, bounded_movie_id.clone(),());    
        

        Self::deposit_event(Event::MovieAddedToFestival(anon_fest_id,bounded_movie_id,who.clone()));
        Ok(())
	
		}



        pub fn do_create_polling_station(
             who: &T::AccountId,
             anon_festival_id: T::AnonFestivalId,
             name:Vec<u8>,
             description:Vec<u8>,
        ) -> Result<T::PollingStationId, DispatchError> {
            
            ensure!(AnonFestivals::<T>::contains_key(anon_festival_id.clone()),Error::<T>::InvalidFestivalId);

            let polling_station_id =
                NextPollingStationId::<T>::try_mutate(|id| -> Result<T::PollingStationId, DispatchError> {
                    let current_id = *id;
                    *id = id
                        .checked_add(&One::one())
                        .ok_or(Error::<T>::Overflow)?;
                    Ok(current_id)
                })?;


            let bounded_name: BoundedVec<u8, T::StringLimit> =
                name.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
            
            let bounded_description: BoundedVec<u8, T::StringLimit> =
                description.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;

            let polling_station = PollingStation {
                owner:who.clone(),
                anon_fest_id:anon_festival_id,
                name:bounded_name,
                description:bounded_description,
            };
            
            PollingStations::<T>::insert(polling_station_id,polling_station);

            Ok(polling_station_id)
        }


	fn activate_festival(now : T::BlockNumber)->DispatchResult{
        for (festival_id,_) in FestivalStartTime::<T>::drain_prefix(&now){
            AnonFestivals::<T>::try_mutate_exists( festival_id,|festival| -> DispatchResult{
                let mut fest = festival.as_mut().ok_or(Error::<T>::FestivalNotFound)?;

                fest.status = AnonFestivalStatus::Open;
                
                Ok(())


            })?;

        }
		Ok(())
    
    }	

	  fn conclude_festival(now: T::BlockNumber)->DispatchResult{
        for (festival_id,_) in FestivalEndTime::<T>::drain_prefix(&now){
           AnonFestivals::<T>::try_mutate_exists( festival_id, |festival| -> DispatchResult{

             let mut fest = festival.as_mut().ok_or(Error::<T>::FestivalNotFound)?;

            //TODO
            //Verificar se o estado atual é ativo
            ensure!(fest.status == AnonFestivalStatus::Open , Error::<T>::FestivalNotActive);

            fest.status = AnonFestivalStatus::OnDecryption;
            Ok(())
           })?;
         }


        Ok(())
      }

	  fn count_votes(now: T::BlockNumber)->DispatchResult{
		
		for (festival_id,_) in FestivalExpiration::<T>::drain_prefix(&now){
			AnonFestivals::<T>::try_mutate_exists(festival_id, |festival|-> DispatchResult{
			
			let mut fest = festival.as_mut().ok_or(Error::<T>::FestivalNotFound)?;

			//verificar se estado = OnDecryption
			ensure!(fest.status == AnonFestivalStatus::OnDecryption, Error::<T>::FestivalNotActive);

			fest.status = AnonFestivalStatus::Counting;

			if DecryptedVotes::<T>::iter_prefix_values(festival_id).count()>0 {

				Self::resolve_market(festival_id.clone())?;
				

			}

			fest.status = AnonFestivalStatus::NormalFinish;

			Ok(())
			

			})?;

		}
		

		Ok(())
	  }

	  fn account_id()->T::AccountId{
		<T as Config>::PalletId::get().into_account_truncating()
	  }

	  fn resolve_market(fest_id : T::AnonFestivalId)-> DispatchResult {

		let winning_opts = Self::get_winning_options(fest_id).unwrap();
		let winners_lockup = Self::get_winners_total_lockup(fest_id,winning_opts.clone()).unwrap();

		for ((vote_id,movie_id),amount) in DecryptedVotes::<T>::iter_prefix(fest_id){

            if winning_opts.contains(&movie_id.clone()) {

				AnonVotes::<T>::try_mutate_exists(fest_id,vote_id, |vote|-> DispatchResult {

					let mut mut_vote = vote.as_mut().ok_or(Error::<T>::FestivalNotFound)?;


				

					T::Currency::transfer(
						 &Self::account_id(),
						 &mut_vote.owner,
						 Self::calculate_simple_reward(TotalLockup::<T>::get(fest_id.clone()),amount,winners_lockup)?,
						 KeepAlive).unwrap();

					Ok(())

				})?;
            }
        }

		Ok(())

	  }

	  fn get_winning_options(fest_id : T::AnonFestivalId)-> Result<Vec<Vec<u8>>,DispatchError> {
		
		let mut accumulator = BTreeMap::new();

        for ((anon_vote_id,movie_id),amount) in DecryptedVotes::<T>::iter_prefix(fest_id){
            // amount -amount = 0 with Balance trait
            let stat =  accumulator.entry(movie_id.clone()).or_insert(amount - amount);
            *stat += amount;
            
        }

        let first_winner= 
				accumulator
				.iter()
				.clone()
				.max_by_key(|p| p.1)
				.unwrap();
		
		let mut winners = vec![first_winner.0.to_vec()];
		
		for (movie, lockup) in &accumulator {
					
			if(lockup == first_winner.1 && movie != first_winner.0){

				winners.push(movie.to_vec());
			}

		}
		
		Ok(winners)
	  }
	fn get_winners_total_lockup(festival_id : T::AnonFestivalId,winning_movies: Vec<Vec<u8>>)-> Result<BalanceOf<T>,DispatchError>{
	 let mut winners_lockup : <<T as pallet_movie::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance=0u32.into();

        for ((anon_vote,movie_id),amnt) in DecryptedVotes::<T>::iter_prefix(festival_id){
            
			if winning_movies.contains(&movie_id.clone()) {
    
             winners_lockup +=amnt;
            
			}   

        }
    
        Ok(winners_lockup)

	}

	fn calculate_simple_reward(
        total_lockup: BalanceOf<T>,
        user_lockup: BalanceOf<T>,
        winner_lockup: BalanceOf<T>) ->Result<BalanceOf<T>,DispatchError>{

        Ok(
           ( user_lockup * 1000u32.into() )/( winner_lockup)*(total_lockup/1000u32.into())

            )
         }
    }

}
