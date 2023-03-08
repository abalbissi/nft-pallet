#![cfg_attr(not(feature = "std"), no_std)]


pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::inherent::Vec;
    use frame_support::pallet;
    use frame_support::pallet_prelude::*;
    use frame_support::pallet_prelude::{DispatchResult, *};
    use frame_support::traits::{Currency, ExistenceRequirement};
    use frame_system::pallet_prelude::*;
    use scale_info::TypeInfo;

    #[pallet::pallet]
    #[pallet::without_storage_info]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        type Currency: Currency<Self::AccountId>;
    }

    #[pallet::storage]
    #[pallet::getter(fn nfts)]
    pub type Nfts<T: Config> = StorageMap<_, Twox64Concat, u128, NftOf<T>>;
    #[pallet::storage]
    #[pallet::getter(fn marketplace)]
    pub type Marketplace<T: Config> = StorageMap<_, Twox64Concat, u128, OfferOf<T>>;
    #[pallet::storage]
    #[pallet::getter(fn nft_id)]
    pub type NftId<T: Config> = StorageValue<_, u128>;

    type BalanceOf<T> =
        <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    type NftOf<T> = Nft<<T as frame_system::Config>::AccountId, Vec<u8>>;
    type OfferOf<T> = Offer<<T as frame_system::Config>::AccountId, BalanceOf<T>>;

    #[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
    pub struct Nft<AccountId, Body> {
        name: Body,
        description: Body,
        img_url: Body,
        owner: AccountId,
    }
    #[derive(Encode, Decode, Clone, Eq, PartialEq, MaxEncodedLen, RuntimeDebug, TypeInfo)]
    pub struct Offer<AccountId, Value> {
        owner: AccountId,
        price: Value,
    }


    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        NewNftMinted {
            id: u128,
            name: Vec<u8>,
            description: Vec<u8>,
            img_url: Vec<u8>,
            owner: T::AccountId,
        },
        ListNft {
            id: u128,
            owner: T::AccountId,
            value: BalanceOf<T>,
        },
        Transfer {
            id: u128,
            from: T::AccountId,
            to: T::AccountId,
        },
        Burned {
            owner: T::AccountId,
            id: u128,
        },
    }

   
    #[pallet::error]
    pub enum Error<T> {
        NotTheOwner,
        BalanceToLow,
        NftNotExist,
        AlreadyListedForSale,
        NftNotForSale,
        NftOwnerSoldNft,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(10_000+ T::DbWeight::get().writes(1).ref_time())]
        pub fn create_nft(
            origin: OriginFor<T>,
            name: Vec<u8>,
            description: Vec<u8>,
            img_url: Vec<u8>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let current_id = match <NftId<T>>::get() {
                None => 0,
                Some(id) => id,
            };
            let new_nft = Nft {
                name,
                description,
                img_url,
                owner: who.clone(),
            };
            <Nfts<T>>::insert(current_id, new_nft.clone());
            <NftId<T>>::put(current_id + 1);
            Self::deposit_event(Event::NewNftMinted {
                name: new_nft.name,
                description: new_nft.description,
                img_url: new_nft.img_url,
                owner: who,
                id: current_id,
            });

            Ok(())
        }
        #[pallet::call_index(1)]
        #[pallet::weight(10_000+ T::DbWeight::get().writes(1).ref_time())]
        pub fn list_nft(origin: OriginFor<T>, nft_id: u128, value: BalanceOf<T>) -> DispatchResult {
            let nft = <Nfts<T>>::get(nft_id).ok_or(Error::<T>::NftNotExist)?;
            let who = ensure_signed(origin)?;
            ensure!(who.clone() == nft.owner.clone(), Error::<T>::NotTheOwner);
            ensure!(
                <Marketplace<T>>::contains_key(nft_id),
                Error::<T>::AlreadyListedForSale
            );
            let new_offer = Offer {
                owner: who.clone(),
                price: value.clone(),
            };
            <Marketplace<T>>::insert(nft_id, new_offer);
            Self::deposit_event(Event::ListNft {
                value,
                owner: who.clone(),
                id: nft_id,
            });

            Ok(())
        }
        #[pallet::call_index(2)]
        #[pallet::weight(10_000+ T::DbWeight::get().writes(1).ref_time())]
        pub fn transfer_nft(
            origin: OriginFor<T>,
            nft_id: u128,
            to: T::AccountId,
        ) -> DispatchResult {
            let nft = <Nfts<T>>::get(nft_id).ok_or(Error::<T>::NftNotExist)?;
            let who = ensure_signed(origin)?;
            ensure!(who == nft.owner, Error::<T>::NotTheOwner);
            <Nfts<T>>::insert(
                nft_id,
                Nft {
                    name: nft.name,
                    description: nft.description,
                    img_url: nft.img_url,
                    owner: to.clone(),
                },
            );

            Self::deposit_event(Event::Transfer {
                id: nft_id,
                from: who,
                to,
               
            });
            Ok(())
        }
        #[pallet::call_index(3)]
        #[pallet::weight(10_000+ T::DbWeight::get().writes(1).ref_time())]
        pub fn burn_nft(
            origin: OriginFor<T>,
            id: u128,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            let nft = <Nfts<T>>::get(id).ok_or(Error::<T>::NftNotExist)?;
            ensure!(who.clone() == nft.owner.clone(), Error::<T>::NotTheOwner);
            <Nfts<T>>::remove(id);
            Self::deposit_event(Event::Burned{
               owner:who.clone(),
               id,
            });
            Ok(())
        }

    }
}
