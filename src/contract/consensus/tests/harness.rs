/* This file is part of DarkFi (https://dark.fi)
 *
 * Copyright (C) 2020-2023 Dyne.org foundation
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use darkfi::{
    consensus::{
        SlotCheckpoint, ValidatorState, ValidatorStatePtr, TESTNET_BOOTSTRAP_TIMESTAMP,
        TESTNET_GENESIS_HASH_BYTES, TESTNET_GENESIS_TIMESTAMP, TESTNET_INITIAL_DISTRIBUTION,
    },
    runtime::vm_runtime::SMART_CONTRACT_ZKAS_DB_NAME,
    tx::Transaction,
    wallet::{WalletDb, WalletPtr},
    zk::{empty_witnesses, halo2::Field, ProvingKey, ZkCircuit},
    zkas::ZkBinary,
    Result,
};
use darkfi_sdk::{
    crypto::{
        merkle_prelude::*, poseidon_hash, Keypair, MerkleNode, MerkleTree, Nullifier, PublicKey,
        SecretKey, CONSENSUS_CONTRACT_ID, DARK_TOKEN_ID, MONEY_CONTRACT_ID,
    },
    pasta::pallas,
    ContractCall,
};
use darkfi_serial::{deserialize, serialize, Encodable};
use log::{info, warn};
use rand::rngs::OsRng;

use darkfi_consensus_contract::{
    client::{
        genesis_stake_v1::ConsensusGenesisStakeCallBuilder,
        proposal_v1::ConsensusProposalCallBuilder, stake_v1::ConsensusStakeCallBuilder,
        unstake_request_v1::ConsensusUnstakeRequestCallBuilder,
        unstake_v1::ConsensusUnstakeCallBuilder,
    },
    model::{ConsensusGenesisStakeParamsV1, ConsensusProposalParamsV1},
    ConsensusFunction,
};
use darkfi_money_contract::{
    client::{
        stake_v1::MoneyStakeCallBuilder, transfer_v1::TransferCallBuilder,
        unstake_v1::MoneyUnstakeCallBuilder, ConsensusNote, ConsensusOwnCoin, MoneyNote, OwnCoin,
    },
    model::{
        Coin, ConsensusOutput, ConsensusStakeParamsV1, ConsensusUnstakeReqParamsV1,
        MoneyTransferParamsV1, MoneyUnstakeParamsV1, Output,
    },
    MoneyFunction, CONSENSUS_CONTRACT_ZKAS_BURN_NS_V1, CONSENSUS_CONTRACT_ZKAS_MINT_NS_V1,
    CONSENSUS_CONTRACT_ZKAS_PROPOSAL_NS_V1, MONEY_CONTRACT_ZKAS_BURN_NS_V1,
    MONEY_CONTRACT_ZKAS_MINT_NS_V1,
};

pub fn init_logger() {
    let mut cfg = simplelog::ConfigBuilder::new();
    cfg.add_filter_ignore("sled".to_string());
    cfg.add_filter_ignore("blockchain::contractstore".to_string());
    // We check this error so we can execute same file tests in parallel,
    // otherwise second one fails to init logger here.
    if let Err(_) = simplelog::TermLogger::init(
        //simplelog::LevelFilter::Info,
        simplelog::LevelFilter::Debug,
        //simplelog::LevelFilter::Trace,
        cfg.build(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto,
    ) {
        warn!(target: "money_harness", "Logger already initialized");
    }
}

/// Enum representing configured wallet holders
#[derive(Debug, Eq, Hash, PartialEq)]
pub enum Holder {
    Faucet,
    Alice,
}

/// Enum representing transaction actions
#[derive(Debug, Eq, Hash, PartialEq)]
pub enum TxAction {
    Airdrop,
    GenesisStake,
    Stake,
    Proposal,
    UnstakeRequest,
    Unstake,
}

/// Auxiliary struct to calculate transaction actions benchmarks
pub struct TxActionBenchmarks {
    /// Vector holding each transaction size in Bytes
    pub sizes: Vec<usize>,
    /// Vector holding each transaction broadcasted size in Bytes
    pub broadcasted_sizes: Vec<usize>,
    /// Vector holding each transaction creation time
    pub creation_times: Vec<Duration>,
    /// Vector holding each transaction verify time
    pub verify_times: Vec<Duration>,
}

impl TxActionBenchmarks {
    pub fn new() -> Self {
        Self {
            sizes: vec![],
            broadcasted_sizes: vec![],
            creation_times: vec![],
            verify_times: vec![],
        }
    }

    pub fn statistics(&self, action: &TxAction) {
        if !self.sizes.is_empty() {
            let avg = self.sizes.iter().sum::<usize>();
            let avg = avg / self.sizes.len();
            info!(target: "consensus", "Average {:?} size: {:?} Bytes", action, avg);
        }
        if !self.broadcasted_sizes.is_empty() {
            let avg = self.broadcasted_sizes.iter().sum::<usize>();
            let avg = avg / self.broadcasted_sizes.len();
            info!(target: "consensus", "Average {:?} broadcasted size: {:?} Bytes", action, avg);
        }
        if !self.creation_times.is_empty() {
            let avg = self.creation_times.iter().sum::<Duration>();
            let avg = avg / self.creation_times.len() as u32;
            info!(target: "consensus", "Average {:?} creation time: {:?}", action, avg);
        }
        if !self.verify_times.is_empty() {
            let avg = self.verify_times.iter().sum::<Duration>();
            let avg = avg / self.verify_times.len() as u32;
            info!(target: "consensus", "Average {:?} verification time: {:?}", action, avg);
        }
    }
}

pub struct Wallet {
    pub keypair: Keypair,
    pub state: ValidatorStatePtr,
    pub money_merkle_tree: MerkleTree,
    pub consensus_staked_merkle_tree: MerkleTree,
    pub consensus_unstaked_merkle_tree: MerkleTree,
    pub wallet: WalletPtr,
    pub coins: Vec<OwnCoin>,
    pub spent_coins: Vec<OwnCoin>,
}

impl Wallet {
    async fn new(keypair: Keypair, faucet_pubkeys: &[PublicKey]) -> Result<Self> {
        let wallet = WalletDb::new("sqlite::memory:", "foo").await?;
        let sled_db = sled::Config::new().temporary(true).open()?;

        let state = ValidatorState::new(
            &sled_db,
            *TESTNET_BOOTSTRAP_TIMESTAMP,
            *TESTNET_GENESIS_TIMESTAMP,
            *TESTNET_GENESIS_HASH_BYTES,
            *TESTNET_INITIAL_DISTRIBUTION,
            wallet.clone(),
            faucet_pubkeys.to_vec(),
            false,
            false,
        )
        .await?;

        let money_merkle_tree = MerkleTree::new(100);
        let consensus_staked_merkle_tree = MerkleTree::new(100);
        let consensus_unstaked_merkle_tree = MerkleTree::new(100);

        let coins = vec![];
        let spent_coins = vec![];

        Ok(Self {
            keypair,
            state,
            money_merkle_tree,
            consensus_staked_merkle_tree,
            consensus_unstaked_merkle_tree,
            wallet,
            coins,
            spent_coins,
        })
    }
}

pub struct ConsensusTestHarness {
    pub holders: HashMap<Holder, Wallet>,
    pub proving_keys: HashMap<&'static str, (ProvingKey, ZkBinary)>,
    pub tx_action_benchmarks: HashMap<TxAction, TxActionBenchmarks>,
}

impl ConsensusTestHarness {
    pub async fn new() -> Result<Self> {
        let mut holders = HashMap::new();
        let faucet_kp = Keypair::random(&mut OsRng);
        let faucet_pubkeys = vec![faucet_kp.public];
        let faucet = Wallet::new(faucet_kp, &faucet_pubkeys).await?;
        holders.insert(Holder::Faucet, faucet);

        let alice_kp = Keypair::random(&mut OsRng);
        let alice = Wallet::new(alice_kp, &faucet_pubkeys).await?;

        // Get the zkas circuits and build proving keys
        let mut proving_keys = HashMap::new();
        let alice_sled = alice.state.read().await.blockchain.sled_db.clone();
        let mut db_handle = alice.state.read().await.blockchain.contracts.lookup(
            &alice_sled,
            &MONEY_CONTRACT_ID,
            SMART_CONTRACT_ZKAS_DB_NAME,
        )?;

        macro_rules! mkpk {
            ($ns:expr) => {
                let zkas_bytes = db_handle.get(&serialize(&$ns))?.unwrap();
                let (zkbin, _): (Vec<u8>, Vec<u8>) = deserialize(&zkas_bytes)?;
                let zkbin = ZkBinary::decode(&zkbin)?;
                let witnesses = empty_witnesses(&zkbin);
                let circuit = ZkCircuit::new(witnesses, zkbin.clone());
                let pk = ProvingKey::build(13, &circuit);
                proving_keys.insert($ns, (pk, zkbin));
            };
        }

        mkpk!(MONEY_CONTRACT_ZKAS_MINT_NS_V1);
        mkpk!(MONEY_CONTRACT_ZKAS_BURN_NS_V1);

        db_handle = alice.state.read().await.blockchain.contracts.lookup(
            &alice_sled,
            &CONSENSUS_CONTRACT_ID,
            SMART_CONTRACT_ZKAS_DB_NAME,
        )?;
        mkpk!(CONSENSUS_CONTRACT_ZKAS_MINT_NS_V1);
        mkpk!(CONSENSUS_CONTRACT_ZKAS_BURN_NS_V1);
        mkpk!(CONSENSUS_CONTRACT_ZKAS_PROPOSAL_NS_V1);

        holders.insert(Holder::Alice, alice);

        // Build benchmarks map
        let mut tx_action_benchmarks = HashMap::new();
        tx_action_benchmarks.insert(TxAction::Airdrop, TxActionBenchmarks::new());
        tx_action_benchmarks.insert(TxAction::GenesisStake, TxActionBenchmarks::new());
        tx_action_benchmarks.insert(TxAction::Stake, TxActionBenchmarks::new());
        tx_action_benchmarks.insert(TxAction::Proposal, TxActionBenchmarks::new());
        tx_action_benchmarks.insert(TxAction::UnstakeRequest, TxActionBenchmarks::new());
        tx_action_benchmarks.insert(TxAction::Unstake, TxActionBenchmarks::new());

        Ok(Self { holders, proving_keys, tx_action_benchmarks })
    }

    pub fn airdrop_native(
        &mut self,
        value: u64,
        holder: Holder,
    ) -> Result<(Transaction, MoneyTransferParamsV1)> {
        let recipient = self.holders.get_mut(&holder).unwrap().keypair.public;
        let faucet = self.holders.get_mut(&Holder::Faucet).unwrap();
        let (mint_pk, mint_zkbin) = self.proving_keys.get(&MONEY_CONTRACT_ZKAS_MINT_NS_V1).unwrap();
        let (burn_pk, burn_zkbin) = self.proving_keys.get(&MONEY_CONTRACT_ZKAS_BURN_NS_V1).unwrap();
        let tx_action_benchmark = self.tx_action_benchmarks.get_mut(&TxAction::Airdrop).unwrap();
        let timer = Instant::now();

        let builder = TransferCallBuilder {
            keypair: faucet.keypair,
            recipient,
            value,
            token_id: *DARK_TOKEN_ID,
            rcpt_spend_hook: pallas::Base::zero(),
            rcpt_user_data: pallas::Base::zero(),
            rcpt_user_data_blind: pallas::Base::random(&mut OsRng),
            change_spend_hook: pallas::Base::zero(),
            change_user_data: pallas::Base::zero(),
            change_user_data_blind: pallas::Base::random(&mut OsRng),
            coins: vec![],
            tree: faucet.money_merkle_tree.clone(),
            mint_zkbin: mint_zkbin.clone(),
            mint_pk: mint_pk.clone(),
            burn_zkbin: burn_zkbin.clone(),
            burn_pk: burn_pk.clone(),
            clear_input: true,
        };

        let debris = builder.build()?;

        let mut data = vec![MoneyFunction::TransferV1 as u8];
        debris.params.encode(&mut data)?;
        let calls = vec![ContractCall { contract_id: *MONEY_CONTRACT_ID, data }];
        let proofs = vec![debris.proofs];
        let mut tx = Transaction { calls, proofs, signatures: vec![] };
        let sigs = tx.create_sigs(&mut OsRng, &debris.signature_secrets)?;
        tx.signatures = vec![sigs];
        tx_action_benchmark.creation_times.push(timer.elapsed());

        // Calculate transaction sizes
        let encoded: Vec<u8> = serialize(&tx);
        let size = std::mem::size_of_val(&*encoded);
        tx_action_benchmark.sizes.push(size);
        let base58 = bs58::encode(&encoded).into_string();
        let size = std::mem::size_of_val(&*base58);
        tx_action_benchmark.broadcasted_sizes.push(size);

        Ok((tx, debris.params))
    }

    pub async fn execute_airdrop_native_tx(
        &mut self,
        holder: Holder,
        tx: Transaction,
        params: &MoneyTransferParamsV1,
        slot: u64,
    ) -> Result<()> {
        let wallet = self.holders.get_mut(&holder).unwrap();
        let tx_action_benchmark = self.tx_action_benchmarks.get_mut(&TxAction::Airdrop).unwrap();
        let timer = Instant::now();

        let erroneous_txs =
            wallet.state.read().await.verify_transactions(&[tx], slot, true).await?;
        assert!(erroneous_txs.is_empty());
        wallet.money_merkle_tree.append(&MerkleNode::from(params.outputs[0].coin.inner()));
        tx_action_benchmark.verify_times.push(timer.elapsed());

        Ok(())
    }

    pub fn genesis_stake(
        &mut self,
        holder: Holder,
        amount: u64,
    ) -> Result<(Transaction, ConsensusGenesisStakeParamsV1)> {
        let wallet = self.holders.get_mut(&holder).unwrap();
        let (mint_pk, mint_zkbin) =
            self.proving_keys.get(&CONSENSUS_CONTRACT_ZKAS_MINT_NS_V1).unwrap();
        let tx_action_benchmark =
            self.tx_action_benchmarks.get_mut(&TxAction::GenesisStake).unwrap();
        let timer = Instant::now();

        // Building Consensus::GenesisStake params
        let genesis_stake_call_debris = ConsensusGenesisStakeCallBuilder {
            keypair: wallet.keypair,
            recipient: wallet.keypair.public,
            amount,
            mint_zkbin: mint_zkbin.clone(),
            mint_pk: mint_pk.clone(),
        }
        .build()?;
        let (genesis_stake_params, genesis_stake_proofs) =
            (genesis_stake_call_debris.params, genesis_stake_call_debris.proofs);

        // Building genesis stake tx
        let mut data = vec![ConsensusFunction::GenesisStakeV1 as u8];
        genesis_stake_params.encode(&mut data)?;
        let contract_call = ContractCall { contract_id: *CONSENSUS_CONTRACT_ID, data };
        let calls = vec![contract_call];
        let proofs = vec![genesis_stake_proofs];
        let mut genesis_stake_tx = Transaction { calls, proofs, signatures: vec![] };
        let sigs = genesis_stake_tx.create_sigs(&mut OsRng, &[wallet.keypair.secret])?;
        genesis_stake_tx.signatures = vec![sigs];
        tx_action_benchmark.creation_times.push(timer.elapsed());

        // Calculate transaction sizes
        let encoded: Vec<u8> = serialize(&genesis_stake_tx);
        let size = std::mem::size_of_val(&*encoded);
        tx_action_benchmark.sizes.push(size);
        let base58 = bs58::encode(&encoded).into_string();
        let size = std::mem::size_of_val(&*base58);
        tx_action_benchmark.broadcasted_sizes.push(size);

        Ok((genesis_stake_tx, genesis_stake_params))
    }

    pub async fn execute_genesis_stake_tx(
        &mut self,
        holder: Holder,
        tx: Transaction,
        params: &ConsensusGenesisStakeParamsV1,
        slot: u64,
    ) -> Result<()> {
        let wallet = self.holders.get_mut(&holder).unwrap();
        let tx_action_benchmark =
            self.tx_action_benchmarks.get_mut(&TxAction::GenesisStake).unwrap();
        let timer = Instant::now();

        let erroneous_txs =
            wallet.state.read().await.verify_transactions(&[tx], slot, true).await?;
        assert!(erroneous_txs.is_empty());
        wallet.consensus_staked_merkle_tree.append(&MerkleNode::from(params.output.coin.inner()));
        tx_action_benchmark.verify_times.push(timer.elapsed());

        Ok(())
    }

    pub async fn execute_erroneous_genesis_stake_txs(
        &mut self,
        holder: Holder,
        txs: Vec<Transaction>,
        slot: u64,
        erroneous: usize,
    ) -> Result<()> {
        let wallet = self.holders.get_mut(&holder).unwrap();
        let tx_action_benchmark =
            self.tx_action_benchmarks.get_mut(&TxAction::GenesisStake).unwrap();
        let timer = Instant::now();

        let erroneous_txs =
            wallet.state.read().await.verify_transactions(&txs, slot, false).await?;
        assert_eq!(erroneous_txs.len(), erroneous);
        tx_action_benchmark.verify_times.push(timer.elapsed());

        Ok(())
    }

    pub async fn stake(
        &mut self,
        holder: Holder,
        slot: u64,
        owncoin: OwnCoin,
    ) -> Result<(Transaction, ConsensusStakeParamsV1, SecretKey)> {
        let wallet = self.holders.get_mut(&holder).unwrap();
        let (mint_pk, mint_zkbin) =
            self.proving_keys.get(&CONSENSUS_CONTRACT_ZKAS_MINT_NS_V1).unwrap();
        let (burn_pk, burn_zkbin) = self.proving_keys.get(&MONEY_CONTRACT_ZKAS_BURN_NS_V1).unwrap();
        let tx_action_benchmark = self.tx_action_benchmarks.get_mut(&TxAction::Stake).unwrap();
        let epoch = wallet.state.read().await.consensus.time_keeper.slot_epoch(slot);
        let timer = Instant::now();

        // Building Money::Stake params
        let money_stake_call_debris = MoneyStakeCallBuilder {
            coin: owncoin.clone(),
            tree: wallet.money_merkle_tree.clone(),
            burn_zkbin: burn_zkbin.clone(),
            burn_pk: burn_pk.clone(),
        }
        .build()?;

        let (
            money_stake_params,
            money_stake_proofs,
            money_stake_secret_key,
            money_stake_value_blind,
        ) = (
            money_stake_call_debris.params,
            money_stake_call_debris.proofs,
            money_stake_call_debris.signature_secret,
            money_stake_call_debris.value_blind,
        );

        // Building Consensus::Stake params
        let consensus_stake_call_debris = ConsensusStakeCallBuilder {
            coin: owncoin,
            epoch,
            value_blind: money_stake_value_blind,
            money_input: money_stake_params.input.clone(),
            mint_zkbin: mint_zkbin.clone(),
            mint_pk: mint_pk.clone(),
        }
        .build()?;

        let (consensus_stake_params, consensus_stake_proofs, consensus_stake_secret_key) = (
            consensus_stake_call_debris.params,
            consensus_stake_call_debris.proofs,
            consensus_stake_call_debris.signature_secret,
        );

        // Building stake tx
        let mut data = vec![MoneyFunction::StakeV1 as u8];
        money_stake_params.encode(&mut data)?;
        let money_call = ContractCall { contract_id: *MONEY_CONTRACT_ID, data };

        let mut data = vec![ConsensusFunction::StakeV1 as u8];
        consensus_stake_params.encode(&mut data)?;
        let consensus_call = ContractCall { contract_id: *CONSENSUS_CONTRACT_ID, data };

        let calls = vec![money_call, consensus_call];
        let proofs = vec![money_stake_proofs, consensus_stake_proofs];
        let mut stake_tx = Transaction { calls, proofs, signatures: vec![] };
        let money_sigs = stake_tx.create_sigs(&mut OsRng, &[money_stake_secret_key])?;
        let consensus_sigs = stake_tx.create_sigs(&mut OsRng, &[consensus_stake_secret_key])?;
        stake_tx.signatures = vec![money_sigs, consensus_sigs];
        tx_action_benchmark.creation_times.push(timer.elapsed());

        // Calculate transaction sizes
        let encoded: Vec<u8> = serialize(&stake_tx);
        let size = std::mem::size_of_val(&*encoded);
        tx_action_benchmark.sizes.push(size);
        let base58 = bs58::encode(&encoded).into_string();
        let size = std::mem::size_of_val(&*base58);
        tx_action_benchmark.broadcasted_sizes.push(size);

        Ok((stake_tx, consensus_stake_params, consensus_stake_secret_key))
    }

    pub async fn execute_stake_tx(
        &mut self,
        holder: Holder,
        tx: Transaction,
        params: &ConsensusStakeParamsV1,
        slot: u64,
    ) -> Result<()> {
        let wallet = self.holders.get_mut(&holder).unwrap();
        let tx_action_benchmark = self.tx_action_benchmarks.get_mut(&TxAction::Stake).unwrap();
        let timer = Instant::now();

        let erroneous_txs =
            wallet.state.read().await.verify_transactions(&[tx], slot, true).await?;
        assert!(erroneous_txs.is_empty());
        wallet.consensus_staked_merkle_tree.append(&MerkleNode::from(params.output.coin.inner()));
        tx_action_benchmark.verify_times.push(timer.elapsed());

        Ok(())
    }

    pub async fn proposal(
        &mut self,
        holder: Holder,
        slot_checkpoint: SlotCheckpoint,
        staked_oc: ConsensusOwnCoin,
    ) -> Result<(Transaction, ConsensusProposalParamsV1, SecretKey, SecretKey)> {
        let wallet = self.holders.get_mut(&holder).unwrap();
        let (proposal_pk, proposal_zkbin) =
            self.proving_keys.get(&CONSENSUS_CONTRACT_ZKAS_PROPOSAL_NS_V1).unwrap();
        let tx_action_benchmark = self.tx_action_benchmarks.get_mut(&TxAction::Proposal).unwrap();
        let timer = Instant::now();

        // Proposals always extend genesis block
        let fork_hash = wallet.state.read().await.consensus.genesis_block;

        // Building Consensus::Propose params
        let proposal_call_debris = ConsensusProposalCallBuilder {
            owncoin: staked_oc,
            slot_checkpoint,
            fork_hash,
            fork_previous_hash: fork_hash,
            merkle_tree: wallet.consensus_staked_merkle_tree.clone(),
            proposal_zkbin: proposal_zkbin.clone(),
            proposal_pk: proposal_pk.clone(),
        }
        .build()?;

        let (params, proofs, output_keypair, signature_secret_key) = (
            proposal_call_debris.params,
            proposal_call_debris.proofs,
            proposal_call_debris.keypair,
            proposal_call_debris.signature_secret,
        );

        let mut data = vec![ConsensusFunction::ProposalV1 as u8];
        params.encode(&mut data)?;
        let call = ContractCall { contract_id: *CONSENSUS_CONTRACT_ID, data };

        let calls = vec![call];
        let proofs = vec![proofs];
        let mut tx = Transaction { calls, proofs, signatures: vec![] };
        let sigs = tx.create_sigs(&mut OsRng, &[signature_secret_key])?;
        tx.signatures = vec![sigs];
        tx_action_benchmark.creation_times.push(timer.elapsed());

        // Calculate transaction sizes
        let encoded: Vec<u8> = serialize(&tx);
        let size = std::mem::size_of_val(&*encoded);
        tx_action_benchmark.sizes.push(size);
        let base58 = bs58::encode(&encoded).into_string();
        let size = std::mem::size_of_val(&*base58);
        tx_action_benchmark.broadcasted_sizes.push(size);

        Ok((tx, params, signature_secret_key, output_keypair.secret))
    }

    pub async fn execute_proposal_tx(
        &mut self,
        holder: Holder,
        tx: Transaction,
        params: &ConsensusProposalParamsV1,
        slot: u64,
    ) -> Result<()> {
        let wallet = self.holders.get_mut(&holder).unwrap();
        let tx_action_benchmark = self.tx_action_benchmarks.get_mut(&TxAction::Proposal).unwrap();
        let timer = Instant::now();

        let erroneous_txs =
            wallet.state.read().await.verify_transactions(&[tx], slot, true).await?;
        assert!(erroneous_txs.is_empty());
        wallet.consensus_staked_merkle_tree.append(&MerkleNode::from(params.output.coin.inner()));
        tx_action_benchmark.verify_times.push(timer.elapsed());

        Ok(())
    }

    pub async fn execute_erroneous_proposal_txs(
        &mut self,
        holder: Holder,
        txs: Vec<Transaction>,
        slot: u64,
        erroneous: usize,
    ) -> Result<()> {
        let wallet = self.holders.get_mut(&holder).unwrap();
        let tx_action_benchmark = self.tx_action_benchmarks.get_mut(&TxAction::Proposal).unwrap();
        let timer = Instant::now();

        let erroneous_txs =
            wallet.state.read().await.verify_transactions(&txs, slot, false).await?;
        assert_eq!(erroneous_txs.len(), erroneous);
        tx_action_benchmark.verify_times.push(timer.elapsed());

        Ok(())
    }

    pub async fn unstake_request(
        &mut self,
        holder: Holder,
        slot: u64,
        staked_oc: ConsensusOwnCoin,
    ) -> Result<(Transaction, ConsensusUnstakeReqParamsV1, SecretKey)> {
        let wallet = self.holders.get_mut(&holder).unwrap();
        let (burn_pk, burn_zkbin) =
            self.proving_keys.get(&CONSENSUS_CONTRACT_ZKAS_BURN_NS_V1).unwrap();
        let (mint_pk, mint_zkbin) =
            self.proving_keys.get(&CONSENSUS_CONTRACT_ZKAS_MINT_NS_V1).unwrap();
        let tx_action_benchmark =
            self.tx_action_benchmarks.get_mut(&TxAction::UnstakeRequest).unwrap();
        let epoch = wallet.state.read().await.consensus.time_keeper.slot_epoch(slot);
        let timer = Instant::now();

        // Building Consensus::Unstake params
        let unstake_request_call_debris = ConsensusUnstakeRequestCallBuilder {
            coin: staked_oc.clone(),
            epoch,
            tree: wallet.consensus_staked_merkle_tree.clone(),
            burn_zkbin: burn_zkbin.clone(),
            burn_pk: burn_pk.clone(),
            mint_zkbin: mint_zkbin.clone(),
            mint_pk: mint_pk.clone(),
        }
        .build()?;
        let (unstake_request_params, unstake_request_proofs, unstake_request_secret_key) = (
            unstake_request_call_debris.params,
            unstake_request_call_debris.proofs,
            unstake_request_call_debris.signature_secret,
        );

        // Building unstake request tx
        let mut data = vec![ConsensusFunction::UnstakeRequestV1 as u8];
        unstake_request_params.encode(&mut data)?;
        let call = ContractCall { contract_id: *CONSENSUS_CONTRACT_ID, data };
        let calls = vec![call];
        let proofs = vec![unstake_request_proofs];
        let mut unstake_request_tx = Transaction { calls, proofs, signatures: vec![] };
        let sigs = unstake_request_tx.create_sigs(&mut OsRng, &[unstake_request_secret_key])?;
        unstake_request_tx.signatures = vec![sigs];
        tx_action_benchmark.creation_times.push(timer.elapsed());

        // Calculate transaction sizes
        let encoded: Vec<u8> = serialize(&unstake_request_tx);
        let size = ::std::mem::size_of_val(&*encoded);
        tx_action_benchmark.sizes.push(size);
        let base58 = bs58::encode(&encoded).into_string();
        let size = ::std::mem::size_of_val(&*base58);
        tx_action_benchmark.broadcasted_sizes.push(size);

        Ok((unstake_request_tx, unstake_request_params, unstake_request_secret_key))
    }

    pub async fn execute_unstake_request_tx(
        &mut self,
        holder: Holder,
        tx: Transaction,
        params: &ConsensusUnstakeReqParamsV1,
        slot: u64,
    ) -> Result<()> {
        let wallet = self.holders.get_mut(&holder).unwrap();
        let tx_action_benchmark =
            self.tx_action_benchmarks.get_mut(&TxAction::UnstakeRequest).unwrap();
        let timer = Instant::now();

        let erroneous_txs =
            wallet.state.read().await.verify_transactions(&[tx], slot, true).await?;
        assert!(erroneous_txs.is_empty());
        wallet.consensus_unstaked_merkle_tree.append(&MerkleNode::from(params.output.coin.inner()));
        tx_action_benchmark.verify_times.push(timer.elapsed());

        Ok(())
    }

    pub async fn execute_erroneous_unstake_request_txs(
        &mut self,
        holder: Holder,
        txs: Vec<Transaction>,
        slot: u64,
        erroneous: usize,
    ) -> Result<()> {
        let wallet = self.holders.get_mut(&holder).unwrap();
        let tx_action_benchmark =
            self.tx_action_benchmarks.get_mut(&TxAction::UnstakeRequest).unwrap();
        let timer = Instant::now();

        let erroneous_txs =
            wallet.state.read().await.verify_transactions(&txs, slot, false).await?;
        assert_eq!(erroneous_txs.len(), erroneous);
        tx_action_benchmark.verify_times.push(timer.elapsed());

        Ok(())
    }

    pub fn unstake(
        &mut self,
        holder: Holder,
        staked_oc: ConsensusOwnCoin,
    ) -> Result<(Transaction, MoneyUnstakeParamsV1, SecretKey)> {
        let wallet = self.holders.get_mut(&holder).unwrap();
        let (burn_pk, burn_zkbin) =
            self.proving_keys.get(&CONSENSUS_CONTRACT_ZKAS_BURN_NS_V1).unwrap();
        let (mint_pk, mint_zkbin) = self.proving_keys.get(&MONEY_CONTRACT_ZKAS_MINT_NS_V1).unwrap();
        let tx_action_benchmark = self.tx_action_benchmarks.get_mut(&TxAction::Unstake).unwrap();
        let timer = Instant::now();

        // Building Consensus::Unstake params
        let consensus_unstake_call_debris = ConsensusUnstakeCallBuilder {
            coin: staked_oc.clone(),
            tree: wallet.consensus_unstaked_merkle_tree.clone(),
            burn_zkbin: burn_zkbin.clone(),
            burn_pk: burn_pk.clone(),
        }
        .build()?;
        let (
            consensus_unstake_params,
            consensus_unstake_proofs,
            consensus_unstake_secret_key,
            consensus_unstake_value_blind,
        ) = (
            consensus_unstake_call_debris.params,
            consensus_unstake_call_debris.proofs,
            consensus_unstake_call_debris.signature_secret,
            consensus_unstake_call_debris.value_blind,
        );

        // Building Money::Unstake params
        let money_unstake_call_debris = MoneyUnstakeCallBuilder {
            coin: staked_oc.into(),
            value_blind: consensus_unstake_value_blind,
            nullifier: consensus_unstake_params.input.nullifier,
            merkle_root: consensus_unstake_params.input.merkle_root,
            signature_public: consensus_unstake_params.input.signature_public,
            mint_zkbin: mint_zkbin.clone(),
            mint_pk: mint_pk.clone(),
        }
        .build()?;
        let (money_unstake_params, money_unstake_proofs) =
            (money_unstake_call_debris.params, money_unstake_call_debris.proofs);

        // Building unstake tx
        let mut data = vec![ConsensusFunction::UnstakeV1 as u8];
        consensus_unstake_params.encode(&mut data)?;
        let consensus_call = ContractCall { contract_id: *CONSENSUS_CONTRACT_ID, data };

        let mut data = vec![MoneyFunction::UnstakeV1 as u8];
        money_unstake_params.encode(&mut data)?;
        let money_call = ContractCall { contract_id: *MONEY_CONTRACT_ID, data };

        let calls = vec![consensus_call, money_call];
        let proofs = vec![consensus_unstake_proofs, money_unstake_proofs];
        let mut unstake_tx = Transaction { calls, proofs, signatures: vec![] };
        let consensus_sigs = unstake_tx.create_sigs(&mut OsRng, &[consensus_unstake_secret_key])?;
        let money_sigs = unstake_tx.create_sigs(&mut OsRng, &[consensus_unstake_secret_key])?;
        unstake_tx.signatures = vec![consensus_sigs, money_sigs];
        tx_action_benchmark.creation_times.push(timer.elapsed());

        // Calculate transaction sizes
        let encoded: Vec<u8> = serialize(&unstake_tx);
        let size = ::std::mem::size_of_val(&*encoded);
        tx_action_benchmark.sizes.push(size);
        let base58 = bs58::encode(&encoded).into_string();
        let size = ::std::mem::size_of_val(&*base58);
        tx_action_benchmark.broadcasted_sizes.push(size);

        Ok((unstake_tx, money_unstake_params, consensus_unstake_secret_key))
    }

    pub async fn execute_unstake_tx(
        &mut self,
        holder: Holder,
        tx: Transaction,
        params: &MoneyUnstakeParamsV1,
        slot: u64,
    ) -> Result<()> {
        let wallet = self.holders.get_mut(&holder).unwrap();
        let tx_action_benchmark = self.tx_action_benchmarks.get_mut(&TxAction::Unstake).unwrap();
        let timer = Instant::now();

        let erroneous_txs =
            wallet.state.read().await.verify_transactions(&[tx], slot, true).await?;
        assert!(erroneous_txs.is_empty());
        wallet.money_merkle_tree.append(&MerkleNode::from(params.output.coin.inner()));
        tx_action_benchmark.verify_times.push(timer.elapsed());

        Ok(())
    }

    pub fn gather_owncoin(
        &mut self,
        holder: Holder,
        output: Output,
        secret_key: Option<SecretKey>,
    ) -> Result<OwnCoin> {
        let wallet = self.holders.get_mut(&holder).unwrap();
        let leaf_position = wallet.money_merkle_tree.witness().unwrap();
        let secret_key = match secret_key {
            Some(key) => key,
            None => wallet.keypair.secret,
        };
        let note: MoneyNote = output.note.decrypt(&secret_key)?;
        let oc = OwnCoin {
            coin: Coin::from(output.coin),
            note: note.clone(),
            secret: secret_key,
            nullifier: Nullifier::from(poseidon_hash([wallet.keypair.secret.inner(), note.serial])),
            leaf_position,
        };

        Ok(oc)
    }

    pub fn gather_consensus_staked_owncoin(
        &mut self,
        holder: Holder,
        output: ConsensusOutput,
        secret_key: Option<SecretKey>,
    ) -> Result<ConsensusOwnCoin> {
        let wallet = self.holders.get_mut(&holder).unwrap();
        let leaf_position = wallet.consensus_staked_merkle_tree.witness().unwrap();
        let secret_key = match secret_key {
            Some(key) => key,
            None => wallet.keypair.secret,
        };
        let note: ConsensusNote = output.note.decrypt(&secret_key)?;
        let oc = ConsensusOwnCoin {
            coin: Coin::from(output.coin),
            note: note.clone(),
            secret: secret_key,
            nullifier: Nullifier::from(poseidon_hash([wallet.keypair.secret.inner(), note.serial])),
            leaf_position,
        };

        Ok(oc)
    }

    pub fn gather_consensus_unstaked_owncoin(
        &mut self,
        holder: Holder,
        output: ConsensusOutput,
        secret_key: Option<SecretKey>,
    ) -> Result<ConsensusOwnCoin> {
        let wallet = self.holders.get_mut(&holder).unwrap();
        let leaf_position = wallet.consensus_unstaked_merkle_tree.witness().unwrap();
        let secret_key = match secret_key {
            Some(key) => key,
            None => wallet.keypair.secret,
        };
        let note: ConsensusNote = output.note.decrypt(&secret_key)?;
        let oc = ConsensusOwnCoin {
            coin: Coin::from(output.coin),
            note: note.clone(),
            secret: secret_key,
            nullifier: Nullifier::from(poseidon_hash([wallet.keypair.secret.inner(), note.serial])),
            leaf_position,
        };

        Ok(oc)
    }

    pub async fn get_slot_checkpoint_by_slot(&self, slot: u64) -> Result<SlotCheckpoint> {
        let faucet = self.holders.get(&Holder::Faucet).unwrap();
        let slot_checkpoint =
            faucet.state.read().await.blockchain.get_slot_checkpoints_by_slot(&[slot])?[0]
                .clone()
                .unwrap();

        Ok(slot_checkpoint)
    }

    pub async fn generate_slot_checkpoint(&self, slot: u64) -> Result<SlotCheckpoint> {
        // We grab the genesis slot to generate slot checkpoint
        // using same consensus parameters
        let faucet = self.holders.get(&Holder::Faucet).unwrap();
        let genesis_block = faucet.state.read().await.consensus.genesis_block;
        let fork_hashes = vec![genesis_block];
        let fork_previous_hashes = vec![genesis_block];
        let genesis_slot = self.get_slot_checkpoint_by_slot(0).await?;
        let slot_checkpoint = SlotCheckpoint {
            slot,
            previous_eta: genesis_slot.previous_eta,
            fork_hashes,
            fork_previous_hashes,
            sigma1: genesis_slot.sigma1,
            sigma2: genesis_slot.sigma2,
        };

        // Store generated slot checkpoint
        for wallet in self.holders.values() {
            wallet.state.write().await.receive_slot_checkpoints(&[slot_checkpoint.clone()]).await?;
        }

        Ok(slot_checkpoint)
    }

    pub fn assert_trees(&self) {
        let faucet = self.holders.get(&Holder::Faucet).unwrap();
        let money_root = faucet.money_merkle_tree.root(0).unwrap();
        let consensus_stake_root = faucet.consensus_staked_merkle_tree.root(0).unwrap();
        let consensus_unstake_root = faucet.consensus_unstaked_merkle_tree.root(0).unwrap();
        for wallet in self.holders.values() {
            assert!(money_root == wallet.money_merkle_tree.root(0).unwrap());
            assert!(consensus_stake_root == wallet.consensus_staked_merkle_tree.root(0).unwrap());
            assert!(
                consensus_unstake_root == wallet.consensus_unstaked_merkle_tree.root(0).unwrap()
            );
        }
    }

    pub fn statistics(&self) {
        info!(target: "consensus", "==================== Statistics ====================");
        for (action, tx_action_benchmark) in &self.tx_action_benchmarks {
            tx_action_benchmark.statistics(action);
        }
        info!(target: "consensus", "====================================================");
    }
}
