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
	use frame_support::{pallet_prelude::*,traits::{Currency,ReservableCurrency,ExistenceRequirement::AllowDeath},PalletId};
	use frame_system::{pallet_prelude::*};
    use codec::{Decode, Encode, MaxEncodedLen};
    use sp_runtime::{RuntimeDebug, traits::{AccountIdConversion, AtLeast32BitUnsigned, CheckedAdd, One}};
    use scale_info::prelude::vec::Vec;
    use core::convert::TryInto;
    use frame_support::BoundedVec;
    use scale_info::TypeInfo;
    use sp_std::{collections::btree_map::BTreeMap,vec};
	use pallet_movie;
	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_movie::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type FestivalId: Member  + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
		type CategoryId: Member  + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		/// The maximum length of base uri stored on-chain.
		#[pallet::constant]
        type PalletId: Get<PalletId>;
        #[pallet::constant]
        type MinFestivalDuration : Get<Self::BlockNumber>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen,TypeInfo)]
    pub enum FestivalStatus {
        New,
        Approved,
        Declined,
        Active,
        Inactive,
    
    }
    
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen,TypeInfo)]
    pub struct Festival<AccountId,FestivalId,BoundedString,BalanceOf> {
        pub owner: AccountId,
        pub id: FestivalId,
        pub name:BoundedString,
        pub description:BoundedString,
        pub status:FestivalStatus,
        pub min_entry:BalanceOf,


    }


    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen,TypeInfo)]
    pub struct Category<BoundedString> {
		pub name: BoundedString,
		pub description: BoundedString

	}

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen,TypeInfo)]
    pub struct CuratingVote<BoundedString,BalanceOf> {
        pub  vote:BoundedString,
        pub  bet:BalanceOf,
    }

    #[pallet::storage]
	pub type Festivals<T: Config> =
		StorageMap<_, Blake2_128Concat, T::FestivalId, Festival<T::AccountId,T::FestivalId,BoundedVec<u8, T::StringLimit>,BalanceOf<T>>>;
	
	
    #[pallet::storage]
	pub type Categories<T: Config> =
		StorageMap<_, Blake2_128Concat, T::CategoryId, Category<BoundedVec<u8, T::StringLimit>>>;

    #[pallet::storage]
	#[pallet::getter(fn next_festival_id)]
	pub(super) type NextFestivalId<T: Config> = StorageValue<_, T::FestivalId, ValueQuery>;
	
	
    #[pallet::storage]
	#[pallet::getter(fn next_category_id)]
	pub(super) type NextCategoryId<T: Config> = StorageValue<_, T::CategoryId, ValueQuery>;
	
	#[pallet::storage]
    #[pallet::getter(fn movies_in_festival)]
    pub (super) type MoviesInFestival<T:Config>=
        StorageDoubleMap<_,
        Blake2_128Concat,T::FestivalId,
        Blake2_128Concat,BoundedVec<u8,T::MovieStringLimit>,
        ()
         >;

    #[pallet::storage]
    pub (super) type VotingTracker<T: Config> = 
        StorageDoubleMap<_,
        Blake2_128Concat,T::FestivalId,
        Blake2_128Concat,(T::AccountId,BoundedVec<u8,T::MovieStringLimit>),
        BalanceOf<T>,
        ValueQuery
        >;

	#[pallet::storage]
	#[pallet::getter(fn festival_start_time)]
	/// Index auctions by end time.
	pub type FestivalStartTime<T: Config> =
		StorageDoubleMap<_,
		Blake2_128Concat, T::BlockNumber,
		Blake2_128Concat, T::FestivalId,
		()
		>;

    #[pallet::storage]
	#[pallet::getter(fn festival_end_time)]
	/// Index auctions by end time.
	pub type FestivalEndTime<T: Config> =
		StorageDoubleMap<_,
		Blake2_128Concat,T::BlockNumber,
		Blake2_128Concat,T::FestivalId,
		()
		>;

    #[pallet::storage]
    #[pallet::getter(fn owned_festivals)]
    pub type OwnedFestivals<T:Config> =
        StorageDoubleMap<_,
        Blake2_128Concat, T::AccountId,
        Blake2_128Concat, T::FestivalId,
		Festival<T::AccountId,T::FestivalId,BoundedVec<u8, T::StringLimit>,BalanceOf<T>>>;
    
    #[pallet::storage]
    #[pallet::getter(fn total_lockup)]
    pub type TotalLockup<T:Config>=
        StorageMap<_,
        Blake2_128Concat, T::FestivalId,
        BalanceOf<T>,ValueQuery>;


	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
        FestivalCreated(T::FestivalId, T::AccountId),
		MovieAddedToFestival(T::FestivalId,BoundedVec<u8,T::MovieStringLimit>,T::AccountId),
        VotedForMovieInFestival(T::FestivalId,BoundedVec<u8,T::MovieStringLimit>,T::AccountId),
        TestEvent(BalanceOf<T>),
	}

    #[pallet::hooks]
        impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
            
            fn on_initialize(_now: T::BlockNumber) -> Weight {
                0
            }

            fn on_finalize(now: T::BlockNumber){
                Self::conclude_festival(now).unwrap();
                
                Self::activate_festival(now).unwrap();
            }
        

        }

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NoAvailableMovieId,
		Overflow,
		Underflow,
        BadMetadata,
        FestivalNotFound,
        MovieNotFound,
        MovieAlreadyInFestival,
        FestivalNotActive,
        LockedAmountInferiorToMinEntry,
        PastStartDate,
        FestivalPeriodTooShort,
        InsuficientBalance,
        FestivalHooksError,

	}


	#[pallet::call]
	impl<T: Config> Pallet<T> {
		
		#[pallet::weight(10_000)]
		pub fn create_festival(
             origin: OriginFor<T>,
             name:Vec<u8>,
             description:Vec<u8>,
			 categories:Vec<T::CategoryId>,
             min_entry: BalanceOf<T>,
             start_block: T::BlockNumber,
             end_block: T::BlockNumber,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_create_festival(&who, name,description,min_entry,start_block,end_block)?;

            Ok(())

		}

		#[pallet::weight(10_000)]
		pub fn create_category(
             origin: OriginFor<T>,
             name:Vec<u8>,
             description:Vec<u8>
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_create_category(&who, name,description)?;

            Ok(())
		}
        #[pallet::weight(10_000)]
        pub fn add_movie(
            origin: OriginFor<T>,
            festival_id: T::FestivalId,
            movie_id: Vec<u8>,
        )-> DispatchResult{
        
            let who = ensure_signed(origin)?;

            Self::do_add_movie(&who,festival_id,movie_id)?;

            Ok(())
        
        }

        #[pallet::weight(10_000)]
        pub fn vote_for(
            origin: OriginFor<T>,
            festival_id: T::FestivalId,
            movie_id : Vec<u8>,
            amount : BalanceOf<T>,
        )-> DispatchResult{
            
            let who = ensure_signed(origin)?;

            Self::do_vote_for(&who,festival_id,movie_id,amount)?;

            Ok(())
            
        }
		
	}

    impl<T: Config> Pallet<T> {

    	// The account ID of the vault
	    fn account_id() -> T::AccountId {
            <T as Config>::PalletId::get().into_account_truncating()
    	}
		pub fn do_create_category (
			who: &T::AccountId,
			name: Vec<u8>,
			description: Vec<u8>
		)->Result<T::CategoryId,DispatchError> {

            let category_id =
                NextCategoryId::<T>::try_mutate(|id| -> Result<T::CategoryId, DispatchError> {
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

			let category = Category {
				
				name:bounded_name,
				description:bounded_description
			
			};


            Categories::<T>::insert(category_id, category.clone());
			Ok(category_id)
		}

        pub fn do_create_festival(
             who: &T::AccountId,
             name:Vec<u8>,
             description:Vec<u8>,
             min_ticket_price:BalanceOf<T>,
             start_block : T::BlockNumber,
             end_block: T::BlockNumber

        ) -> Result<T::FestivalId, DispatchError> {
            let festival_id =
                NextFestivalId::<T>::try_mutate(|id| -> Result<T::FestivalId, DispatchError> {
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
            

            
            let festival = Festival {
                owner:who.clone(),
                id: festival_id.clone(),
                name:bounded_name,
                description:bounded_description,
                status:FestivalStatus::New,
                min_entry: min_ticket_price,
                
            };
    
               
                
            //verificar datas!!!!!!! TODO


            //ensure now > start
			ensure!(frame_system::Pallet::<T>::block_number() < start_block,Error::<T>::PastStartDate);

            //ensure (end-start)>MinAlivePeriod
            ensure!(end_block-start_block >= T::MinFestivalDuration::get(), Error::<T>::FestivalPeriodTooShort);
            FestivalEndTime::<T>::insert(end_block,festival_id,());

            Festivals::<T>::insert(festival_id, festival.clone());
            OwnedFestivals::<T>::insert(who.clone(),festival_id,festival.clone());
            FestivalStartTime::<T>::insert(start_block,festival_id,());
		
			Self::deposit_event(Event::FestivalCreated(festival_id, who.clone()));
            Ok(festival_id)
           
            }
    
        
    pub fn do_add_movie(
        who: &T::AccountId,
        festival_id: T::FestivalId,
        movie_id : Vec<u8>,
        )->DispatchResult{

        
        let bounded_movie_id: BoundedVec<u8, T::MovieStringLimit> =
          TryInto::try_into(movie_id).map_err(|_| Error::<T>::BadMetadata)?;
       
        //ensure festival exists
        ensure!(Festivals::<T>::contains_key(festival_id.clone()),Error::<T>::FestivalNotFound);
       
        // ensure movie exists
        ensure!(pallet_movie::Movies::<T>::contains_key(bounded_movie_id.clone()),Error::<T>::MovieNotFound);
        
        //ensure movie is not in list already
        ensure!(!MoviesInFestival::<T>::contains_key(festival_id.clone(),bounded_movie_id.clone()),Error::<T>::MovieAlreadyInFestival);
        
        MoviesInFestival::<T>::insert(festival_id, bounded_movie_id.clone(),());    
        

        Self::deposit_event(Event::MovieAddedToFestival(festival_id,bounded_movie_id,who.clone()));
        Ok(())
    }
    pub fn do_vote_for(
        who: &T::AccountId,
        festival_id: T::FestivalId,
        movie_id : Vec<u8>,
        amount: BalanceOf<T>,
        )->DispatchResult {
        
        
        Festivals::<T>::try_mutate_exists(festival_id, |festival| -> DispatchResult {
            
            let fest = festival.as_mut().ok_or(Error::<T>::FestivalNotFound)?;   

            let bounded_movie_id: BoundedVec<u8,T::MovieStringLimit>= 
                TryInto::try_into(movie_id).map_err(|_| Error::<T>::BadMetadata)?;

            //ensure festival status is active
            ensure!(fest.status == FestivalStatus::Active,Error::<T>::FestivalNotActive);

            //ensure movie_id exists in festival
            ensure!(MoviesInFestival::<T>::contains_key(festival_id.clone(),bounded_movie_id.clone()),Error::<T>::MovieNotFound);

            //create new entry or add to old entry
            
            VotingTracker::<T>::try_mutate(festival_id, (who,bounded_movie_id.clone()), |total_amount| -> DispatchResult{
                
                               
                //ensure amount is greater than min bet

                ensure!(fest.min_entry <= amount,Error::<T>::LockedAmountInferiorToMinEntry);

                ensure!(
                <T as Config>::Currency::transfer(
                     who,
                     &Self::account_id(),
                     amount,
                     AllowDeath,
                     ) 
                == Ok(()),Error::<T>::InsuficientBalance);
                
                *total_amount+=amount.clone();
                
                TotalLockup::<T>::try_mutate(festival_id,|total_festival_locked_amnt|->DispatchResult{

                    *total_festival_locked_amnt+=amount.clone();
                    Self::deposit_event(Event::VotedForMovieInFestival(festival_id,bounded_movie_id,who.clone()));
                    Ok(())
               
                })
                
           
            }) 


        })

    }

    fn activate_festival(now : T::BlockNumber)->DispatchResult{
        for (festival_id,_) in FestivalStartTime::<T>::iter_prefix(&now){
            Festivals::<T>::try_mutate_exists( festival_id,|festival| -> DispatchResult{
                let fest = festival.as_mut().ok_or(Error::<T>::FestivalNotFound)?;
            
                //TODO
                // verificar se estado atual de festival é aprovado

                fest.status = FestivalStatus::Active;
                
                Ok(())


            })?;

        }
        Ok(())
    
    }

    fn conclude_festival(now: T::BlockNumber)->DispatchResult{
        for (festival_id,_) in FestivalEndTime::<T>::drain_prefix(&now){
           Festivals::<T>::try_mutate_exists( festival_id, |festival| -> DispatchResult{ 
            
             let fest = festival.as_mut().ok_or(Error::<T>::FestivalNotFound)?;
            
            //TODO
            //Verificar se o estado atual é ativo
            ensure!(fest.status == FestivalStatus::Active , Error::<T>::FestivalNotActive);
            
            if VotingTracker::<T>::iter_prefix_values(festival_id).count()>0 {
                
                Self::resolve_market(festival_id.clone())?;
            
            }

            fest.status = FestivalStatus::Inactive;
            Ok(())
           })?;
         }


        Ok(())
    }

    fn resolve_market(festival_id : T::FestivalId) -> DispatchResult {
        
        let winning_opts = Self::get_winning_options(festival_id).unwrap();
        
        let winners_lockup = Self::get_winners_total_lockup(festival_id,winning_opts.clone()).unwrap();
        
        Self::deposit_event(Event::TestEvent(winners_lockup.into()));
        
        for ((acc,movie_id),amount) in VotingTracker::<T>::iter_prefix(festival_id){
                
            if winning_opts.contains(&movie_id.clone()) {
                
                <T as Config>::Currency::transfer(
                    &Self::account_id(),
                    &acc,
                    Self::calculate_simple_reward(TotalLockup::<T>::get(festival_id.clone()),amount,winners_lockup)?,
                    AllowDeath).unwrap();

            }
            
        }

        Ok(())
    }
/*
    fn get_winning_option(festival_id : T::FestivalId)-> Result<Vec<u8>,DispatchError>{
       
        let mut accumulator = BTreeMap::new();

        for ((_,movie_id),amount) in VotingTracker::<T>::iter_prefix(festival_id){
            // amount -amount = 0 with Balance trait
            let stat =  accumulator.entry(movie_id.clone().into_inner()).or_insert(amount - amount);
            *stat += amount;
            
        }
        
        Ok(
            (
             *    
             accumulator
             .iter().clone()
             .max_by_key(|p| p.1)
             .unwrap()
             .0
            ).to_vec()
        )
    }

*/
    fn get_winning_options(festival_id : T::FestivalId)-> Result<Vec<Vec<u8>>,DispatchError>{
       
        let mut accumulator = BTreeMap::new();

        for ((_,movie_id),amount) in VotingTracker::<T>::iter_prefix(festival_id){
            // amount -amount = 0 with Balance trait
            let stat =  accumulator.entry(movie_id.clone().into_inner()).or_insert(amount - amount);
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

    fn get_winners_total_lockup(festival_id: T::FestivalId, winning_movies:Vec<Vec<u8>>)->Result<BalanceOf<T>,DispatchError>{
        
        let mut winners_lockup : <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance=0u32.into();

        for ((_acc,movie_id),amnt) in VotingTracker::<T>::iter_prefix(festival_id){
            
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

