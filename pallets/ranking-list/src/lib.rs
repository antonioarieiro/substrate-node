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
	use frame_system::{pallet_prelude::*,RawOrigin};
    use sp_runtime::{RuntimeDebug, traits::{AtLeast32BitUnsigned, CheckedAdd, One,Saturating,Zero,AccountIdConversion}};
    use scale_info::prelude::vec::Vec;
    use scale_info::TypeInfo;
    use frame_support::BoundedVec;
    use codec::{MaxEncodedLen};
    use core::convert::TryInto;
    use sp_runtime::traits::StaticLookup;
	use sp_std::vec;
   type BalanceOf<T> =
	    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
	


    #[pallet::config]
	pub trait Config: frame_system::Config{
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type RankingListId: Member  + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
        /// The maximum length of base uri stored on-chain.
		#[pallet::constant]
		type StringLimit: Get<u32>;
        type PalletId : Get<PalletId>;
		type Currency : Currency<Self::AccountId> + ReservableCurrency<Self::AccountId> ;

	}

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

    #[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug,TypeInfo)]
    pub enum RankingListStatus {
        New,
        Approved,
        Declined,
        Active,
        Inactive,
    }


    #[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, RuntimeDebug,TypeInfo)]
    pub enum RestakeFunds {
        Yes,
        No,
    }


    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug,TypeInfo)]//MaxEncodedLen
    pub struct RankingList<AccountId,RankingListId,BoundedString,RankingListStatus> {
        pub	creator: AccountId,
        pub id : RankingListId,
        pub name:BoundedString,
        pub description:BoundedString,
        pub status:RankingListStatus,
        pub category: BoundedString,            
        
    }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug,TypeInfo)]//MaxEncodedLen
    pub struct Vote<BoundedString,BalanceOf> {
        pub movie_id: BoundedString,
        pub locked_amount:BalanceOf,
    }

    #[pallet::storage]
	pub type RankingLists<T: Config> =
		StorageMap<_, Blake2_128Concat, T::RankingListId, RankingList<T::AccountId,T::RankingListId,BoundedVec<u8, T::StringLimit>,RankingListStatus>>;
    #[pallet::storage]
	#[pallet::getter(fn next_ranking_list_id)]
	pub(super) type NextRankingListId<T: Config> = StorageValue<_, T::RankingListId, ValueQuery>;

    #[pallet::storage]
    pub type MoviesInList<T:Config> = 
        StorageDoubleMap<_,
        Blake2_128Concat, T::RankingListId,
        Blake2_128Concat,BoundedVec<u8,T::StringLimit>,
        BoundedVec<u8,T::StringLimit>
            >;

    #[pallet::storage]
    pub type Votes<T:Config>=
        StorageDoubleMap<_,
        Blake2_128Concat,T::RankingListId,
        Blake2_128Concat,(T::AccountId,BoundedVec<u8,T::StringLimit>),
        BalanceOf<T>,
        ValueQuery
        >;

    #[pallet::storage]
    #[pallet::getter(fn next_derivate_nounce)]
    pub type DerivativeNounce<T:Config> = 
        StorageValue < _,
                      u16,
                      ValueQuery>;

	
    #[pallet::storage]
    pub type LastKnownEra<T:Config> = 
        StorageValue < _,
                      u16,
                      ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
        RankingListCreated(T::RankingListId, T::AccountId),
	    MovieAddedToList(T::RankingListId,BoundedVec<u8,T::StringLimit>,T::AccountId),	
        TestEvent(T::AccountId,u32),
    }




    #[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(now: T::BlockNumber) -> Weight {
				
				if !LastKnownEra::<T>::exists(){

				}

				10_000
		 }
	  
		 fn on_finalize(now: T::BlockNumber){
		
		
         }

	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		NoAvailableMovieId,
		Overflow,
		Underflow,
        BadMetadata,
        MovieAlreadyInList,
        RankingListNotFound,
        MovieNotFound,
        StakingWithNoValue,
        NotEnoughBalance,
	}


	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000)]
		pub fn create_ranking_list(
             origin: OriginFor<T>,
             name:Vec<u8>,
             description:Vec<u8>,
             category:Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_create_ranking_list(&who, name,description,category)?;

			Ok(())
		}

        #[pallet::weight(10_000)]
        pub fn add_movie(
            origin: OriginFor<T>,
            ranking_list_id:T::RankingListId,
            title_id:Vec<u8>,
            title_name:Vec<u8>,
            ) -> DispatchResult{
            let who = ensure_signed(origin)?;
    
            Self::do_add_movie(&who,ranking_list_id,title_id,title_name)?;
            
            Ok(())
            
        }

        // vote = bond + stake
        //
        #[pallet::weight(10_000)]
        pub fn vote_for(
            origin:OriginFor<T>,
            ranking_list_id:T::RankingListId,
            title_id:Vec<u8>,
            amount: BalanceOf<T>,
            restake: RestakeFunds,
			targets: Vec<<T::Lookup as StaticLookup>::Source>
            ) -> DispatchResult{

            let _who = ensure_signed(origin.clone())?;

            Self::do_vote_for2(origin,ranking_list_id,title_id,amount,restake,targets)?;

            Ok(())
        }


		
	}

    impl<T: Config> Pallet<T> {

       //vault acc id
       fn account_id()->T::AccountId{
            <T as Config>::PalletId::get().into_account_truncating()
        
        }


       fn do_create_ranking_list(
             who: &T::AccountId,
             name:Vec<u8>,
             description:Vec<u8>,
             category:Vec<u8>,
        ) -> Result<T::RankingListId, DispatchError> {
           
            let ranking_list_id =
                NextRankingListId::<T>::try_mutate(|id| -> Result<T::RankingListId, DispatchError> {
                    let current_id = *id;
                    *id = id
                        .checked_add(&One::one())
                        .ok_or(Error::<T>::Overflow)?;
                    Ok(current_id)
                })?;
    
                

            let bounded_name: BoundedVec<u8, T::StringLimit> =
                TryInto::try_into(name).map_err(|_| Error::<T>::BadMetadata)?;
            
            let bounded_description: BoundedVec<u8, T::StringLimit> =
                TryInto::try_into(description).map_err(|_| Error::<T>::BadMetadata)?;
            
            
            let bounded_category: BoundedVec<u8, T::StringLimit> =
                TryInto::try_into(category).map_err(|_| Error::<T>::BadMetadata)?;
            
            let ranked_list = RankingList {
                creator:who.clone(),
                id:ranking_list_id.clone(),
                name:bounded_name,
                description:bounded_description,
                status:RankingListStatus::New,
                category:bounded_category
            };
    
        
            RankingLists::<T>::insert(ranking_list_id.clone(), ranked_list);
    
            Self::deposit_event(Event::RankingListCreated(ranking_list_id, who.clone()));
            Ok(ranking_list_id)
        } 
    
    fn do_add_movie(
        who: &T::AccountId,
        list_id: T::RankingListId,
        title_id: Vec<u8>,
        title_name: Vec<u8>,
        )->DispatchResult{
                        
            let bounded_title_id: BoundedVec<u8, T::StringLimit> =
                    TryInto::try_into(title_id).map_err(|_| Error::<T>::BadMetadata)?;
            
            let bounded_name: BoundedVec<u8, T::StringLimit> =
                    TryInto::try_into(title_name).map_err(|_| Error::<T>::BadMetadata)?;

            
            //TODO!!!!!!!!!!!!!!!!!!!!!!!!!!
            //verificar se um movie existe
    
            //ensure id exists
            ensure!(NextRankingListId::<T>::get()>list_id.clone(),Error::<T>::RankingListNotFound);

            //ensure not in list
            ensure!(!MoviesInList::<T>::contains_key(list_id.clone(),bounded_title_id.clone()),Error::<T>::MovieAlreadyInList);

            MoviesInList::<T>::insert(list_id.clone(),bounded_title_id.clone(),bounded_name);

            
            Self::deposit_event(Event::MovieAddedToList(list_id,bounded_title_id, who.clone()));
            Ok(())           
        }

    fn do_vote_for2(
        origin: OriginFor<T>,
        ranking_list_id: T::RankingListId,
        title_id:Vec<u8>,
        amount: BalanceOf<T>,
        restake: RestakeFunds,
		targets: Vec<<T::Lookup as StaticLookup>::Source>
        )-> DispatchResult{
            
            let staker = &ensure_signed(origin.clone())?;

            //ensure ranking list id exists
            ensure!(RankingLists::<T>::contains_key(ranking_list_id.clone()),Error::<T>::RankingListNotFound);
            
            
            let bounded_title_id: BoundedVec<u8, T::StringLimit> =
                    TryInto::try_into(title_id).map_err(|_| Error::<T>::BadMetadata)?;

            //ensure ranking list contains movie
            ensure!(MoviesInList::<T>::contains_key(ranking_list_id.clone(),bounded_title_id.clone()),Error::<T>::MovieNotFound);
            
    
            // Ensure that staker has enough balance to bond & stake.
            ensure!(amount > 0u32.into() , Error::<T>::StakingWithNoValue);

			<T as Config>::Currency::reserve(&staker,amount)?;
        
            
        
            Votes::<T>::try_mutate(ranking_list_id,(staker.clone(),bounded_title_id), |amnt| -> DispatchResult {
                //transfer amount to vault
 /*               ensure!(
                <T as pallet_staking::Config>::Currency::transfer(staker,&Self::account_id(),amount.clone(),AllowDeath)
                 ==
                Ok(()),Error::<T>::NotEnoughBalance);*/

                *amnt += amount;
                
				Ok(())

            })


            
       }
/*
    fn _do_vote_for(
        origin: OriginFor<T>,
        ranking_list_id: T::RankingListId,
        title_id:Vec<u8>,
        amount: BalanceOf<T>,
        )->DispatchResult{

            let staker = &ensure_signed(origin.clone())?;
            
         
            //ensure ranking list id exists
            ensure!(RankingLists::<T>::contains_key(ranking_list_id.clone()),Error::<T>::RankingListNotFound);
            
            let bounded_title_id: BoundedVec<u8, T::StringLimit> =
                    TryInto::try_into(title_id).map_err(|_| Error::<T>::BadMetadata)?;

            //ensure ranking list contains movie
            ensure!(MoviesInList::<T>::contains_key(ranking_list_id.clone(),bounded_title_id),Error::<T>::MovieNotFound);
            
    
            // Ensure that staker has enough balance to bond & stake.
            let free_balance = T::Currency::free_balance(staker);

             // Remove already locked funds from the free balance
            let available_balance = free_balance.saturating_sub(0u32.into());   //ledger);

            let value_to_stake = amount.min(available_balance);

            ensure!(!value_to_stake.is_zero(), Error::<T>::StakingWithNoValue);
        
             // update the ledger value by adding the newly bonded funds
            //ledger += value_to_stake;

            let controller_lookup: <T::Lookup as StaticLookup>::Source = T::Lookup::unlookup(staker.clone());
             
            pallet_staking::Pallet::<T>::bond(origin.clone(),controller_lookup,amount,pallet_staking::RewardDestination::Staked)?;

            
            //let validator_set  =  pallet_staking::SessionInterface::<<T as frame_system::Config>::AccountId>::validators();
            

            let validator_set = Self::validators();
        
            Self::deposit_event(Event::TestEvent(staker.clone(),validator_set.len() as u32));
            
            let mut selected_validators: Vec<<T::Lookup as StaticLookup>::Source> =
			    Vec::with_capacity(validator_set.len());

            for i in validator_set.iter() {
                
               let j = i.clone();
               selected_validators.push(<T::Lookup as StaticLookup>::unlookup(j));
                

            }

            pallet_staking::Pallet::<T>::nominate(
		    	origin.clone(),
		    	selected_validators,
		    )

            

    }
*/

	fn _convert(account: T::AccountId) -> Option<T::AccountId> {
		Some(account)
	}

}
}

