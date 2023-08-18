// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A collection of node-specific RPC methods.
//!
//! Since `substrate` core functionality makes no assumptions
//! about the modules used inside the runtime, so do
//! RPC methods defined in `sc-rpc` crate.
//! It means that `client/rpc` can't have any methods that
//! need some strong assumptions about the particular runtime.
//!
//! The RPCs available in this crate however can make some assumptions
//! about how the runtime is constructed and what FRAME pallets
//! are part of it. Therefore all node-runtime-specific RPCs can
//! be placed here or imported from corresponding FRAME RPC definitions.

#![warn(missing_docs)]
#![warn(unused_crate_dependencies)]

//use std::sync::Arc;
use std::{collections::BTreeMap, sync::Arc};

use sc_consensus_babe::BabeWorkerHandle;

use jsonrpsee::RpcModule;
use node_5ire_runtime::{opaque::Block};

use sp_core::H256;

use sc_network_sync::SyncingService;

use node_primitives::{AccountId, Balance, BlockNumber, Hash, Index};
use sc_client_api::AuxStore;
//use sc_consensus_babe::BabeWorkerHandle;
use sc_consensus_babe::{BabeConfiguration, Epoch};
use sc_consensus_epochs::SharedEpochChanges;

use grandpa::{
	FinalityProofProvider, GrandpaJustificationStream, SharedAuthoritySet, SharedVoterState,
};
use sc_rpc::SubscriptionTaskExecutor;
pub use sc_rpc_api::DenyUnsafe;
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_consensus::SelectChain;
use sp_consensus_babe::BabeApi;
use sp_keystore::KeystorePtr;
//use sp_keystore::SyncCryptoStorePtr;

use sp_runtime::traits::Block as BlockT;



use sc_client_api::{
	backend::{ Backend, StateBackend, StorageProvider},
	client::BlockchainEvents,
};
//=============================================
use sp_runtime::traits::BlakeTwo256;
use sc_transaction_pool::{ChainApi, Pool};
use sc_network::NetworkService;
use fc_rpc_core::types::{FeeHistoryCache, FeeHistoryCacheLimit, FilterPool};
// Frontier
use fc_rpc::{
	EthBlockDataCacheTask, OverrideHandle, RuntimeApiStorageOverride, SchemaV1Override,
	SchemaV2Override, SchemaV3Override, StorageOverride,TxPool, EthConfig
};
use fp_storage::EthereumStorageSchema;

/// Extra dependencies for BABE.
pub struct BabeDeps {

    /// BABE protocol config.
	//pub babe_config: BabeConfiguration,
	/// BABE pending epoch changes.
	/// pub shared_epoch_changes: SharedEpochChanges<Block, Epoch>,
	/// A handle to the BABE worker for issuing requests.
	pub babe_worker_handle: BabeWorkerHandle<Block>,
	/// The keystore that manages the keys of the node.
	pub keystore: KeystorePtr,
}

/// Extra dependencies for GRANDPA
pub struct GrandpaDeps<B> {
	/// Voting round info.
	pub shared_voter_state: SharedVoterState,
	/// Authority set info.
	pub shared_authority_set: SharedAuthoritySet<Hash, BlockNumber>,
	/// Receives notifications about justification events from Grandpa.
	pub justification_stream: GrandpaJustificationStream<Block>,
	/// Executor to drive the subscription manager in the Grandpa RPC handler.
	pub subscription_executor: SubscriptionTaskExecutor,
	/// Finality proof provider.
	pub finality_provider: Arc<FinalityProofProvider<B, Block>>,
}


/// Full client dependencies.
pub struct FullDeps<C, P, SC, B:BlockT, A:ChainApi> {
	/// The client instance to use.
	pub client: Arc<C>,
	/// Transaction pool instance.
	pub pool: Arc<P>,
	/// The SelectChain Strategy
	pub select_chain: SC,
	/// A copy of the chain spec.
	pub chain_spec: Box<dyn sc_chain_spec::ChainSpec>,
	/// Whether to deny unsafe calls
	pub deny_unsafe: DenyUnsafe,
	/// BABE specific dependencies.
	pub babe: BabeDeps,
	/// GRANDPA specific dependencies.
	pub grandpa: GrandpaDeps<B>,
	/// Shared statement store reference.
	pub statement_store: Arc<dyn sp_statement_store::StatementStore>,

    /// Graph pool instance.
	pub graph: Arc<Pool<A>>,
	/// The Node authority flag
	pub is_authority: bool,
	/// Whether to enable dev signer
	pub enable_dev_signer: bool,
	/// Network service
	pub network: Arc<NetworkService<Block, Hash>>,


   pub sync: Arc<SyncingService<B>>,

	/// EthFilterApi pool.
	pub filter_pool: Option<FilterPool>,
	/// Backend.
	pub backend: Arc<fc_db::Backend<Block>>,
	/// Maximum number of logs in a query.
	pub max_past_logs: u32,
	/// Fee history cache.
	pub fee_history_cache: FeeHistoryCache,
	/// Maximum fee history cache size.
	pub fee_history_cache_limit: FeeHistoryCacheLimit,
	/// Ethereum data access overrides.
	pub overrides: Arc<OverrideHandle<Block>>,
	/// Cache for Ethereum block data.
	pub block_data_cache: Arc<EthBlockDataCacheTask<Block>>,
	pub execute_gas_limit_multiplier: u64,


    /// Mandated parent hashes for a given block hash.
	pub forced_parent_hashes: Option<BTreeMap<H256, H256>>,

}



pub fn overrides_handle<C, BE>(client: Arc<C>) -> Arc<OverrideHandle<Block>>
where
	C: ProvideRuntimeApi<Block> + StorageProvider<Block, BE> + AuxStore,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError>,
	C: Send + Sync + 'static,
	C::Api: sp_api::ApiExt<Block>
		+ fp_rpc::EthereumRuntimeRPCApi<Block>
		+ fp_rpc::ConvertTransactionRuntimeApi<Block>,
	BE: Backend<Block> + 'static,
	BE::State: StateBackend<BlakeTwo256>,
{
	// let mut overrides_map = BTreeMap::new();
	// overrides_map.insert(
	// 	EthereumStorageSchema::V1,
	// 	Box::new(SchemaV1Override::new(client.clone()))
	// 		as Box<dyn StorageOverride<_> + Send + Sync>,
	// );
	// overrides_map.insert(
	// 	EthereumStorageSchema::V2,
	// 	Box::new(SchemaV2Override::new(client.clone()))
	// 		as Box<dyn StorageOverride<_> + Send + Sync>,
	// );
	// overrides_map.insert(
	// 	EthereumStorageSchema::V3,
	// 	Box::new(SchemaV3Override::new(client.clone()))
	// 		as Box<dyn StorageOverride<_> + Send + Sync>,
	// );

	// Arc::new(OverrideHandle {
	// 	schemas: overrides_map,
	// 	fallback: Box::new(RuntimeApiStorageOverride::new(client)),
	// })

	let mut overrides_map = BTreeMap::new();
	overrides_map.insert(
		EthereumStorageSchema::V1,
		Box::new(SchemaV1Override::new(client.clone())) as Box<dyn StorageOverride<_>>,
	);
	overrides_map.insert(
		EthereumStorageSchema::V2,
		Box::new(SchemaV2Override::new(client.clone())) as Box<dyn StorageOverride<_>>,
	);
	overrides_map.insert(
		EthereumStorageSchema::V3,
		Box::new(SchemaV3Override::new(client.clone())) as Box<dyn StorageOverride<_>>,
	);

	Arc::new(OverrideHandle {
		schemas: overrides_map,
		fallback: Box::new(RuntimeApiStorageOverride::new(client)),
	})
}



/// Instantiate all Full RPC extensions.
pub fn create_full<C, P, SC, B,BE,A,EC: EthConfig<B, C>>(   
    	deps: FullDeps<C, P, SC, B, A>,
    subscription_task_executor: SubscriptionTaskExecutor,
    pubsub_notification_sinks: Arc<
    fc_mapping_sync::EthereumBlockNotificationSinks<
        fc_mapping_sync::EthereumBlockNotification<B>,
        >,
    >,
	_backend: Arc<B>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
// B: BlockT<Hash = sp_core::H256>,
BE: Backend<Block> + 'static,
BE::State: StateBackend<BlakeTwo256>,
	C: ProvideRuntimeApi<Block>
		+ sc_client_api::BlockBackend<Block>
		+ HeaderBackend<Block>
		+ AuxStore
		+ HeaderMetadata<Block, Error = BlockChainError>
		+ Sync
		+ Send
        + StorageProvider<Block, BE>
        + 'static,

        C: BlockchainEvents<Block>,
	C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError>,
	C: Send + Sync + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,


	C::Api: BlockBuilder<Block>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
	C::Api: BabeApi<Block>,
	C::Api: BlockBuilder<Block>,
	//P: TransactionPool + 'static,
    P: TransactionPool<Block=Block> + 'static,
	SC: SelectChain<Block> + 'static,
    C::Api: fp_rpc::ConvertTransactionRuntimeApi<Block>,
	C::Api: fp_rpc::EthereumRuntimeRPCApi<Block>,
	B: sc_client_api::Backend<Block> + Send + Sync + 'static ,  //+ sp_api::BlockT,
	B::State: sc_client_api::backend::StateBackend<sp_runtime::traits::HashFor<Block>>,
    A: ChainApi<Block = Block> + 'static,
    B: BlockT<Hash = sp_core::H256>,


{
	//use mmr_rpc::{Mmr, MmrApiServer};
	use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
	use sc_consensus_babe_rpc::{Babe, BabeApiServer};
	use sc_consensus_grandpa_rpc::{Grandpa, GrandpaApiServer};
	//use sc_rpc::{
	//	dev::{Dev, DevApiServer},
	//	statement::StatementApiServer,
	//};
    //use sc_finality_grandpa_rpc::{Grandpa, GrandpaApiServer};

	use sc_rpc_spec_v2::chain_spec::{ChainSpec, ChainSpecApiServer};
	use sc_sync_state_rpc::{SyncState, SyncStateApiServer};
	use substrate_frame_rpc_system::{System, SystemApiServer};
	use substrate_state_trie_migration_rpc::{StateMigration, StateMigrationApiServer};

    use fc_rpc::{
		Eth, EthApiServer, EthDevSigner, EthFilter, EthFilterApiServer, EthPubSub,
		EthPubSubApiServer, EthSigner, Net, NetApiServer, TxPoolApiServer, Web3, Web3ApiServer,
	};

	let mut io = RpcModule::new(());
	let FullDeps { client, pool, select_chain, chain_spec, deny_unsafe, babe, grandpa,statement_store,graph,
		is_authority,
		enable_dev_signer,
		network,
        sync,
		filter_pool,
		backend,
		max_past_logs,
		fee_history_cache,
		fee_history_cache_limit,
		overrides,
		block_data_cache,
		execute_gas_limit_multiplier ,
        forced_parent_hashes,
	} = deps;

	let BabeDeps { keystore, babe_worker_handle } = babe;
	
    //let BabeDeps { keystore, babe_config} = babe;

    let GrandpaDeps {
		shared_voter_state,
		shared_authority_set,
		justification_stream,
		subscription_executor,
		finality_provider,
	} = grandpa;


    let  pp=pool.clone();
	let  pbp=pool.clone();

	let chain_name = chain_spec.name().to_string();
	let genesis_hash = client.block_hash(0).ok().flatten().expect("Genesis block exists; qed");
	let properties = chain_spec.properties();
	io.merge(ChainSpec::new(chain_name, genesis_hash, properties).into_rpc())?;

	io.merge(System::new(client.clone(), pool, deny_unsafe).into_rpc())?;
	// Making synchronous calls in light client freezes the browser currently,
	// more context: https://github.com/paritytech/substrate/pull/3480
	// These RPCs should use an asynchronous caller instead.
	//io.merge(Mmr::new(client.clone()).into_rpc())?;
	//io.merge(TransactionPayment::new(client.clone()).into_rpc())?;
	//io.merge(
	//	Babe::new(client.clone(), babe_worker_handle.clone(), keystore, select_chain, deny_unsafe)
	//		.into_rpc(),
	//)?;

    // io.merge(
	// 	Babe::new(
	// 		client.clone(),
    //         babe_worker_handle.clone(),
    //         keystore,
			
	// 		select_chain,
	// 		deny_unsafe,
	// 	)
	// 	.into_rpc(),
	// )?;
	// io.merge(
	// 	Grandpa::new(
	// 		subscription_executor,
	// 		shared_authority_set.clone(),
	// 		shared_voter_state,
	// 		justification_stream,
	// 		finality_provider,
	// 	)
	// 	.into_rpc(),
	// )?;

	io.merge(
		SyncState::new(chain_spec, client.clone(), shared_authority_set, babe_worker_handle)?
			.into_rpc(),
	)?;

	//io.merge(StateMigration::new(client.clone(), backend, deny_unsafe).into_rpc())?;
    			// Making synchronous calls in light client freezes the browser currently,
	// more context: https://github.com/paritytech/substrate/pull/3480
	// These RPCs should use an asynchronous caller instead.
	// io.merge(Contracts::new(client.clone()).into_rpc())?;
	// io.merge(Mmr::new(client.clone()).into_rpc())?;
	io.merge(TransactionPayment::new(client.clone()).into_rpc())?;
	let mut signers = Vec::new();
	if enable_dev_signer {
		signers.push(Box::new(EthDevSigner::new()) as Box<dyn EthSigner>);
	}
	io.merge(
		Babe::new(
			client.clone(),
            babe_worker_handle.clone(),
            			keystore,
			select_chain,
			deny_unsafe,
		)
		.into_rpc(),
	)?;
	io.merge(
		Grandpa::new(
			subscription_executor,
			shared_authority_set.clone(),
			shared_voter_state,
			justification_stream,
			finality_provider,
		)
		.into_rpc(),
	)?;
	io.merge(
		SyncState::new(chain_spec, client.clone(), shared_authority_set,  babe_worker_handle)?
			.into_rpc(),
	)?;
	// io.merge(StateMigration::new(client.clone(), backend, deny_unsafe).into_rpc())?;
    io.merge(
		       Eth::new(
			client.clone(),
			pp,
			graph,
			Some(node_5ire_runtime::TransactionConverter),
            sync,
            signers,
            overrides.clone(),
            backend.clone(),
			
			// Is authority.
			is_authority,
			block_data_cache.clone(),
			fee_history_cache,
			fee_history_cache_limit,
			execute_gas_limit_multiplier,
            forced_parent_hashes,
		)
		.replace_config::<EC>()
		.into_rpc(),
	)?;

    let tx_pool = TxPool::new(client.clone(), graph);

    if let Some(filter_pool) = filter_pool {
		
		io.merge(
			EthFilter::new(
				client.clone(),
				backend,
                tx_pool.clone(),
				filter_pool,
				500_usize, // max stored filters
				max_past_logs,
				block_data_cache,
			)
			.into_rpc(),
		)?;
	}
	io.merge(
		EthPubSub::new(
			pbp,
			client.clone(),
            sync, 
			subscription_task_executor,
			overrides,
            pubsub_notification_sinks,
		)
		.into_rpc(),
	)?;
	// io.merge(Contracts::new(client.clone()).into_rpc())?;
	io.merge(
		Net::new(
			client.clone(),
			network,
			// Whether to format the `peer_count` response as Hex (default) or not.
			true,
		)
		.into_rpc(),
	)?;
	io.merge(Web3::new(client).into_rpc())?;
	// io.merge(Dev::new(client, deny_unsafe).into_rpc())?;

//	io.merge(Dev::new(client, deny_unsafe).into_rpc())?;
//	let statement_store =
//		sc_rpc::statement::StatementStore::new(statement_store, deny_unsafe).into_rpc();
	//io.merge(statement_store)?;

	Ok(io)
}
