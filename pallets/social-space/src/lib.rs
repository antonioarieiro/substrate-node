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
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;
    use sp_runtime::{RuntimeDebug, traits::{AtLeast32BitUnsigned, CheckedAdd, One,Zero}};
    use codec::{Decode, Encode, MaxEncodedLen};
    use scale_info::TypeInfo;
    use frame_support::BoundedVec;
    use core::convert::TryInto;
    use scale_info::prelude::vec::Vec;
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        #[pallet::constant]
		type StringLimit: Get<u32>;

        type PostId: Member  + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;

        type CommentId: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaxEncodedLen;
    }

	#[pallet::pallet]
	#[pallet::without_storage_info]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);


    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen,TypeInfo)]
    pub struct Space<AccountId,BoundedString> {
        pub owner: AccountId,
        pub name:BoundedString,
        pub description:BoundedString,
        pub following : u32,
        pub followers : u32,
    }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen,TypeInfo)]
    pub struct Post<AccountId,PostId,BoundedString,BlockNumber> {
        pub owner: AccountId,
        pub id: PostId,
        pub content:BoundedString,
        pub image_ipfs: BoundedString,
        pub video_ipfs: BoundedString,
        pub likes : u32,
        pub dislikes : u32,
        pub date : BlockNumber
    }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen,TypeInfo)]
    pub struct Comment<AccountId,BoundedString,PostId,CommentId,BlockNumber> {
        pub owner:AccountId,
        pub id: CommentId,
        pub content: BoundedString,
        pub post_id : PostId,
        pub parent_comment_id: CommentId,
        pub likes: u32,
        pub dislikes:u32,
        pub date : BlockNumber

    }    
    #[pallet::storage]
	pub type Spaces<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Space<T::AccountId,BoundedVec<u8, T::StringLimit>>>;

    #[pallet::storage]
	#[pallet::getter(fn next_post_id)]
	pub(super) type NextPostId<T: Config> = StorageValue<_, T::PostId, ValueQuery>;


    
    #[pallet::storage]
	#[pallet::getter(fn next_comment_id)]
	pub(super) type NextCommentId<T: Config> = StorageValue<_, T::CommentId, ValueQuery>;


    #[pallet::storage]
	pub type Posts<T: Config> =
		StorageMap<_, Blake2_128Concat, T::PostId, Post<T::AccountId,T::PostId,BoundedVec<u8, T::StringLimit>,T::BlockNumber>>;

    
    #[pallet::storage]
	pub type Comments<T: Config> =
		StorageDoubleMap<
        _,
        Blake2_128Concat,T::PostId, 
        Blake2_128Concat,T::CommentId, 
        Comment<T::AccountId,BoundedVec<u8, T::StringLimit>,T::PostId,T::CommentId,T::BlockNumber>>;

    #[pallet::storage]
	pub type SpaceFollowers<T: Config> =
		StorageMap<_, Blake2_128Concat, T::AccountId, Vec<T::AccountId>,ValueQuery>;

    #[pallet::storage]
    pub type SpaceFollowing<T:Config> = 
        StorageMap<_, Blake2_128Concat, T::AccountId,Vec<T::AccountId>,ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SomethingStored(u32, T::AccountId),
        SpaceCreated(T::AccountId),
        SpaceFollowed(T::AccountId,T::AccountId),
        PostCreated(T::AccountId,T::PostId),
        PostCommented(T::AccountId,T::PostId),
        CommmentReplied(T::AccountId,T::CommentId,T::CommentId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
        ProfileAlreadyCreated,
        BadMetadata,
        Overflow,
        SpaceNotFound,
        AlreadyFollowing,
        PostNotFound,
		InexistentProfile,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T:Config> Pallet<T> {

        #[pallet::weight(10_000)]
		pub fn create_space(
             origin: OriginFor<T>,
             name:Vec<u8>,
             description:Vec<u8>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			Self::do_create_space(&who, name,description)


		}
        #[pallet::weight(10_000)]
        pub fn create_post(
            origin: OriginFor<T>,
            content: Vec<u8>,
            image_ipfs: Vec<u8>,
            video_ipfs:Vec<u8>,
            )-> DispatchResult{
            
            let who = ensure_signed(origin)?;

            Self::do_create_post(&who,content,image_ipfs,video_ipfs)


        }

        #[pallet::weight(10_000)]
        pub fn follow_space(
            origin: OriginFor<T>,
            target: T::AccountId,
            )-> DispatchResult {

            let who = ensure_signed(origin)?;

            Self::do_follow_space(&who,&target)?;

            Ok(())

        }

        #[pallet::weight(10_000)]
        pub fn comment_on_post(
            origin: OriginFor<T>,
            post_id: T::PostId,
            content: Vec<u8>,
            )-> DispatchResult {
            
            let who = ensure_signed(origin)?;

            Self::do_comment_on_post(&who,post_id,content)?;
            

            Ok(())
        }

        #[pallet::weight(10_000)]
        pub fn comment_on_comment(
            origin: OriginFor<T>,
            post_id: T::PostId,
            parent_comment_id: T::CommentId,
            content: Vec<u8>,
            )-> DispatchResult {

            let who = ensure_signed(origin)?;

            Self::do_comment_on_comment(&who,post_id,parent_comment_id,content)?;

            Ok(())
            
        }



	}

       impl<T: Config> Pallet<T> {

        pub fn do_create_space(
             who: &T::AccountId,
             name:Vec<u8>,
             description:Vec<u8>,
        ) -> DispatchResult {

            let bounded_name: BoundedVec<u8, T::StringLimit> =
                name.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
            let bounded_description: BoundedVec<u8, T::StringLimit> =
                description.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;

            let space = Space{
                owner:who.clone(),
                name:bounded_name,
                description:bounded_description,
                following:Zero::zero(),
                followers: Zero::zero(),

            };
            
            ensure!(!Spaces::<T>::contains_key(who.clone()),Error::<T>::ProfileAlreadyCreated);
            
            SpaceFollowers::<T>::try_mutate(who.clone(), |_followers|-> DispatchResult{

                Spaces::<T>::insert(who.clone(),space);
                Self::deposit_event(Event::SpaceCreated(who.clone()));
                Ok(())
            })

        }

        pub fn do_create_post(
             who: &T::AccountId,
             content:Vec<u8>,
             image_ipfs:Vec<u8>,
             movie_ipfs:Vec<u8>,
        ) -> DispatchResult {

			ensure!(Spaces::<T>::contains_key(who.clone()),Error::<T>::InexistentProfile);

            let post_id =
             NextPostId::<T>::try_mutate(|id| -> Result<T::PostId, DispatchError> {
                 let current_id = *id;
                 *id = id
                 .checked_add(&One::one())
                 .ok_or(Error::<T>::Overflow)?;
                  Ok(current_id)
                })?;

            let bounded_content: BoundedVec<u8, T::StringLimit> =
                content.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
      
            let bounded_image: BoundedVec<u8, T::StringLimit> =
                image_ipfs.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
         
            let bounded_movie: BoundedVec<u8, T::StringLimit> =
                movie_ipfs.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
            
            let post = Post {
                owner: who.clone(),
                id: post_id.clone(),
                content: bounded_content,
                image_ipfs:bounded_image,
                video_ipfs: bounded_movie,
                likes: Zero::zero(),
                dislikes: Zero::zero(),
                date : <frame_system::Pallet<T>>::block_number()

            };
            
            Posts::<T>::insert(post_id.clone(),post);
            Self::deposit_event(Event::PostCreated(who.clone(),post_id));

            Ok(())    
        
        }

        pub fn do_follow_space(
            who: &T::AccountId,
            target: &T::AccountId
            ) -> DispatchResult {

            SpaceFollowers::<T>::try_mutate_exists(target,|follow| -> DispatchResult {
                let followers = follow.as_mut().ok_or(Error::<T>::SpaceNotFound)?;

                ensure!(!followers.contains(&who.clone()),Error::<T>::AlreadyFollowing);

                Spaces::<T>::try_mutate_exists(target,|space| -> DispatchResult {

                    let space_struct = space.as_mut().ok_or(Error::<T>::SpaceNotFound)?;
                   
                    Spaces::<T>::try_mutate_exists(who.clone(),|space_origin| -> DispatchResult{

                        let space_origin_struct = space_origin.as_mut().ok_or(Error::<T>::SpaceNotFound)?;
                        
                        if target.clone() != who.clone() {

                             space_origin_struct.following = space_origin_struct.following
                                .checked_add(One::one())
                                .ok_or(Error::<T>::Overflow)?;
                        }else{
                            
                             space_struct.following = space_origin_struct.following
                                .checked_add(One::one())
                                .ok_or(Error::<T>::Overflow)?;

                        }
                    
                        space_struct.followers=space_struct.followers
                            .checked_add(One::one())
                            .ok_or(Error::<T>::Overflow)?;
                    
                       followers.push(who.clone());
                       Self::deposit_event(Event::SpaceFollowed(who.clone(),target.clone()));   
                     
                       Ok(())
                    })     
                })
           
            })
            

        }
        
        pub fn do_comment_on_post(
            who: &T::AccountId,
            post_id: T::PostId,
            content: Vec<u8>
            ) -> DispatchResult {

             let comment_id=NextCommentId::<T>::try_mutate(|id| -> Result<T::CommentId, DispatchError> {
                 
                 if *id==Zero::zero() {
                    
                    *id=One::one();
                 }

                 let current_id = *id;
                 *id = id
                 .checked_add(&One::one())
                 .ok_or(Error::<T>::Overflow)?;
                  Ok(current_id)
                })?;

                
            let bounded_content: BoundedVec<u8, T::StringLimit> =
                content.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
           
            
            let comment = Comment{
                owner : who.clone(),
                id : comment_id.clone(),
                content : bounded_content,
                post_id : post_id.clone(),
                parent_comment_id : Zero::zero(),
                likes: Zero::zero(),
                dislikes: Zero::zero(),
                date : <frame_system::Pallet<T>>::block_number()
    
            };
            
            //ensure such comment does no exists
            ensure!(!Comments::<T>::contains_key(post_id.clone(),comment_id.clone()),Error::<T>::NoneValue);
            
            //ensure post exists
            ensure!(Posts::<T>::contains_key(post_id.clone()),Error::<T>::PostNotFound);

            Comments::<T>::insert(post_id.clone(),comment_id,comment);
            Self::deposit_event(Event::PostCommented(who.clone(),post_id));

            Ok(())
        }


        pub fn do_comment_on_comment(
            who: &T::AccountId,
            post_id: T::PostId,
            parent_comment_id : T::CommentId,
            content: Vec<u8>
            ) -> DispatchResult {

            //ensure post exists and parent comment exists 
            ensure!(Posts::<T>::contains_key(post_id.clone()) && Comments::<T>::contains_key(post_id.clone(),parent_comment_id.clone()),Error::<T>::PostNotFound);

             let comment_id=NextCommentId::<T>::try_mutate(|id| -> Result<T::CommentId, DispatchError> {
                 
                 if *id==Zero::zero() {
                    
                    *id=One::one();
                 }

                 let current_id = *id;
                 *id = id
                 .checked_add(&One::one())
                 .ok_or(Error::<T>::Overflow)?;
                  Ok(current_id)
                })?;


            let bounded_content: BoundedVec<u8, T::StringLimit> =
                content.clone().try_into().map_err(|_| Error::<T>::BadMetadata)?;
           
            
            let comment = Comment{
                owner : who.clone(),
                id: comment_id.clone(),
                content : bounded_content,
                post_id : post_id.clone(),
                parent_comment_id : parent_comment_id.clone(),
                likes: Zero::zero(),
                dislikes: Zero::zero(),
                date : <frame_system::Pallet<T>>::block_number()
    
            };

             Comments::<T>::insert(post_id,comment_id.clone(),comment);
             Self::deposit_event(Event::CommmentReplied(who.clone(),parent_comment_id,comment_id));
             Ok(())   

        }
    }

}
