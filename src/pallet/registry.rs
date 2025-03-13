//! # Kombucha Registry Pallet
//!
//! A pallet for tracking and verifying kombucha fermentation on the Elixir Chain.
//!
//! ## Overview
//!
//! The registry pallet provides functionality for:
//! - Registering kombucha fermentation operations
//! - Tracking fermentation conditions and metrics
//! - Verifying brew authenticity
//! - Certification of finished products
//!
//! All operations include comprehensive error correction at the classical,
//! bridge, and quantum levels per Matrix-Magiq architecture requirements.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::{Get, Randomness},
};
use frame_system::{self as system, ensure_signed};
use sp_runtime::traits::{Hash, Zero};
use sp_std::prelude::*;

// Import error correction modules
mod error_correction;
use error_correction::{apply_classical_correction, apply_bridge_correction, apply_quantum_correction};

#[cfg(test)]
mod tests;

/// The pallet's configuration trait.
pub trait Config: frame_system::Config {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    
    /// The quantum-resistant random number generator.
    type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
    
    /// Minimum fermentation period (in blocks)
    type MinFermentationPeriod: Get<Self::BlockNumber>;
}

// Fermentation batch information
#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct FermentationBatch<AccountId, BlockNumber> {
    // Brewer ID
    pub brewer: AccountId,
    // Batch unique identifier
    pub batch_id: Vec<u8>,
    // Start block number
    pub start_block: BlockNumber,
    // Expected completion block number
    pub expected_completion: BlockNumber,
    // Fermentation conditions hash
    pub conditions_hash: H256,
    // Certification status
    pub certified: bool,
    // Quantum verification proof
    pub quantum_proof: Option<Vec<u8>>,
}

// Fermentation conditions
#[derive(Encode, Decode, Clone, Default, RuntimeDebug, PartialEq, Eq)]
pub struct FermentationConditions {
    // Temperature (in Celsius * 10^2)
    pub temperature: u32,
    // pH level (in pH * 10^2)
    pub ph_level: u32,
    // SCOBY generation number
    pub scoby_generation: u32,
    // Tea type hash
    pub tea_hash: H256,
    // Sugar source identifier
    pub sugar_source: Vec<u8>,
    // Additional ingredient hash
    pub ingredients_hash: H256,
}

decl_storage! {
    trait Store for Module<T: Config> as KombuchaRegistry {
        // Storage for fermentation batches
        FermentationBatches get(fn fermentation_batch):
            map hasher(blake2_128_concat) Vec<u8> => FermentationBatch<T::AccountId, T::BlockNumber>;
        
        // Storage for fermentation conditions
        FermentationConditionsList get(fn fermentation_conditions):
            map hasher(blake2_128_concat) H256 => FermentationConditions;
        
        // Batches owned by a specific brewer
        BrewerBatches get(fn brewer_batches):
            map hasher(blake2_128_concat) T::AccountId => Vec<Vec<u8>>;
        
        // Total number of batches
        BatchCount get(fn batch_count): u64 = 0;
    }
}

decl_event! {
    pub enum Event<T> where
        AccountId = <T as frame_system::Config>::AccountId,
    {
        /// A new fermentation batch was registered
        BatchRegistered(AccountId, Vec<u8>),
        /// Fermentation conditions were updated
        ConditionsUpdated(Vec<u8>, H256),
        /// Batch was certified
        BatchCertified(Vec<u8>),
        /// Error correction was applied
        ErrorCorrectionApplied(Vec<u8>, u8), // batch_id, correction_level
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// Batch ID already exists
        BatchIdExists,
        /// Batch does not exist
        BatchNotFound,
        /// Invalid fermentation period
        InvalidFermentationPeriod,
        /// Not authorized to perform this action
        NotAuthorized,
        /// Batch not ready for certification
        NotReadyForCertification,
        /// Error correction failed
        ErrorCorrectionFailed,
        /// Quantum verification failed
        QuantumVerificationFailed,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        // Initialize errors
        type Error = Error<T>;
        
        // Initialize events
        fn deposit_event() = default;
        
        /// Register a new kombucha fermentation batch
        #[weight = 10_000]
        pub fn register_batch(
            origin,
            batch_id: Vec<u8>,
            expected_completion: T::BlockNumber,
            conditions: FermentationConditions,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            
            // Ensure batch ID doesn't already exist
            ensure!(!FermentationBatches::<T>::contains_key(&batch_id), Error::<T>::BatchIdExists);
            
            // Ensure the fermentation period is valid
            let current_block = <frame_system::Module<T>>::block_number();
            ensure!(
                expected_completion > current_block && 
                expected_completion >= current_block + T::MinFermentationPeriod::get(),
                Error::<T>::InvalidFermentationPeriod
            );
            
            // Apply error correction at all levels
            apply_classical_correction(&batch_id).map_err(|_| Error::<T>::ErrorCorrectionFailed)?;
            apply_bridge_correction(&batch_id).map_err(|_| Error::<T>::ErrorCorrectionFailed)?;
            apply_quantum_correction(&batch_id).map_err(|_| Error::<T>::ErrorCorrectionFailed)?;
            
            // Calculate conditions hash
            let conditions_hash = T::Hashing::hash_of(&conditions);
            
            // Store conditions
            FermentationConditionsList::<T>::insert(conditions_hash, conditions);
            
            // Create and store the new batch
            let batch = FermentationBatch {
                brewer: sender.clone(),
                batch_id: batch_id.clone(),
                start_block: current_block,
                expected_completion,
                conditions_hash,
                certified: false,
                quantum_proof: None,
            };
            
            FermentationBatches::<T>::insert(&batch_id, batch);
            
            // Update brewer's batch list
            let mut brewer_batches = BrewerBatches::<T>::get(&sender);
            brewer_batches.push(batch_id.clone());
            BrewerBatches::<T>::insert(&sender, brewer_batches);
            
            // Increment batch count
            let new_count = BatchCount::get().checked_add(1).unwrap_or_default();
            BatchCount::put(new_count);
            
            // Emit event
            Self::deposit_event(RawEvent::BatchRegistered(sender, batch_id));
            
            Ok(())
        }
        
        /// Update fermentation conditions for a batch
        #[weight = 10_000]
        pub fn update_conditions(
            origin,
            batch_id: Vec<u8>,
            conditions: FermentationConditions,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            
            // Ensure batch exists
            ensure!(FermentationBatches::<T>::contains_key(&batch_id), Error::<T>::BatchNotFound);
            
            // Get the batch
            let mut batch = FermentationBatches::<T>::get(&batch_id);
            
            // Ensure sender is the brewer
            ensure!(batch.brewer == sender, Error::<T>::NotAuthorized);
            
            // Apply error correction at all levels
            apply_classical_correction(&batch_id).map_err(|_| Error::<T>::ErrorCorrectionFailed)?;
            apply_bridge_correction(&batch_id).map_err(|_| Error::<T>::ErrorCorrectionFailed)?;
            apply_quantum_correction(&batch_id).map_err(|_| Error::<T>::ErrorCorrectionFailed)?;
            
            // Calculate new conditions hash
            let conditions_hash = T::Hashing::hash_of(&conditions);
            
            // Store updated conditions
            FermentationConditionsList::<T>::insert(conditions_hash, conditions);
            
            // Update batch conditions hash
            batch.conditions_hash = conditions_hash;
            FermentationBatches::<T>::insert(&batch_id, batch);
            
            // Emit event
            Self::deposit_event(RawEvent::ConditionsUpdated(batch_id, conditions_hash));
            
            Ok(())
        }
        
        /// Certify a fermentation batch
        #[weight = 10_000]
        pub fn certify_batch(
            origin,
            batch_id: Vec<u8>,
            quantum_proof: Vec<u8>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            
            // Ensure batch exists
            ensure!(FermentationBatches::<T>::contains_key(&batch_id), Error::<T>::BatchNotFound);
            
            // Get the batch
            let mut batch = FermentationBatches::<T>::get(&batch_id);
            
            // Ensure sender is the brewer
            ensure!(batch.brewer == sender, Error::<T>::NotAuthorized);
            
            // Ensure batch is ready for certification
            let current_block = <frame_system::Module<T>>::block_number();
            ensure!(current_block >= batch.expected_completion, Error::<T>::NotReadyForCertification);
            
            // Apply error correction at all levels
            apply_classical_correction(&batch_id).map_err(|_| Error::<T>::ErrorCorrectionFailed)?;
            apply_bridge_correction(&batch_id).map_err(|_| Error::<T>::ErrorCorrectionFailed)?;
            apply_quantum_correction(&batch_id).map_err(|_| Error::<T>::ErrorCorrectionFailed)?;
            
            // Verify quantum proof
            // This would involve complex quantum verification logic in a real implementation
            // For now, we simply check that the proof is not empty
            ensure!(!quantum_proof.is_empty(), Error::<T>::QuantumVerificationFailed);
            
            // Update batch certification status
            batch.certified = true;
            batch.quantum_proof = Some(quantum_proof);
            FermentationBatches::<T>::insert(&batch_id, batch);
            
            // Emit event
            Self::deposit_event(RawEvent::BatchCertified(batch_id));
            
            Ok(())
        }
    }
}

// Helper functions for the pallet
impl<T: Config> Module<T> {
    /// Generate a unique batch ID
    pub fn generate_batch_id(sender: &T::AccountId) -> Vec<u8> {
        let current_block = <frame_system::Module<T>>::block_number();
        let random_seed = T::Randomness::random(&sender.encode());
        
        // Combine account, block number, and random seed to create a unique ID
        let mut combined = sender.encode();
        combined.extend_from_slice(&current_block.encode());
        combined.extend_from_slice(&random_seed.encode());
        
        combined
    }
    
    /// Verify fermentation conditions are within acceptable ranges
    pub fn verify_conditions(conditions: &FermentationConditions) -> bool {
        // Temperature should be between 20.00 and 30.00 degrees Celsius
        if conditions.temperature < 2000 || conditions.temperature > 3000 {
            return false;
        }
        
        // pH should be between 2.50 and 3.50
        if conditions.ph_level < 250 || conditions.ph_level > 350 {
            return false;
        }
        
        // SCOBY generation should be between 1 and 20
        if conditions.scoby_generation < 1 || conditions.scoby_generation > 20 {
            return false;
        }
        
        true
    }
}
