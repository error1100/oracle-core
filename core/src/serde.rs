//! Types to allow oracle configuration to convert to and from Serde.

use std::convert::{TryFrom, TryInto};

use derive_more::From;
use ergo_lib::ergotree_ir::chain::{
    address::{AddressEncoder, AddressEncoderError, NetworkPrefix},
    token::TokenId,
};
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    cli_commands::{
        bootstrap::{Addresses, BootstrapConfig, TokensToMint},
        prepare_update::{UpdateBootstrapConfig, UpdateTokensToMint},
    },
    contracts::{
        ballot::BallotContractParameters, oracle::OracleContractParameters,
        pool::PoolContractParameters, refresh::RefreshContractParameters,
        update::UpdateContractParameters,
    },
    datapoint_source::PredefinedDataPointSource,
    oracle_config::{
        BallotBoxWrapperParameters, CastBallotBoxVoteParameters, OracleConfig, TokenIds,
    },
};

/// Used to (de)serialize `OracleConfig` instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct OracleConfigSerde {
    node_ip: String,
    node_port: u16,
    node_api_key: String,
    base_fee: u64,
    log_level: Option<LevelFilter>,
    core_api_port: u16,
    oracle_address: String,
    data_point_source: Option<PredefinedDataPointSource>,
    data_point_source_custom_script: Option<String>,
    oracle_contract_parameters: OracleContractParametersSerde,
    pool_contract_parameters: PoolContractParametersSerde,
    refresh_contract_parameters: RefreshContractParametersSerde,
    update_contract_parameters: UpdateContractParametersSerde,
    ballot_parameters: BallotBoxWrapperParametersSerde,
    token_ids: TokenIds,
    addresses: AddressesSerde,
}

#[derive(Debug, Error, From)]
pub enum SerdeConversionError {
    #[error("Serde conversion error: AddressEncoder {0}")]
    AddressEncoder(AddressEncoderError),
    #[error("Serde conversion error: Network prefixes of addresses differ")]
    NetworkPrefixesDiffer,
}

impl From<(OracleConfig, NetworkPrefix)> for OracleConfigSerde {
    fn from(t: (OracleConfig, NetworkPrefix)) -> Self {
        let c = t.0;
        let network_prefix = t.1;
        let oracle_contract_parameters =
            OracleContractParametersSerde::from(c.oracle_contract_parameters);
        let pool_contract_parameters =
            PoolContractParametersSerde::from(c.pool_contract_parameters);
        let refresh_contract_parameters =
            RefreshContractParametersSerde::from(c.refresh_contract_parameters);
        let ballot_parameters = BallotBoxWrapperParametersSerde {
            contract_parameters: BallotContractParametersSerde::from(
                c.ballot_parameters.contract_parameters,
            ),
            vote_parameters: c.ballot_parameters.vote_parameters,
            ballot_token_owner_address: AddressEncoder::new(network_prefix)
                .address_to_str(&c.ballot_parameters.ballot_token_owner_address),
        };
        let update_contract_parameters =
            UpdateContractParametersSerde::from(c.update_contract_parameters);

        OracleConfigSerde {
            node_ip: c.node_ip,
            node_port: c.node_port,
            node_api_key: c.node_api_key,
            base_fee: c.base_fee,
            log_level: c.log_level,
            core_api_port: c.core_api_port,
            oracle_address: c.oracle_address,
            data_point_source: c.data_point_source,
            data_point_source_custom_script: c.data_point_source_custom_script,
            oracle_contract_parameters,
            pool_contract_parameters,
            refresh_contract_parameters,
            update_contract_parameters,
            ballot_parameters,
            token_ids: c.token_ids,
            addresses: AddressesSerde::from((c.addresses, network_prefix)),
        }
    }
}

impl TryFrom<OracleConfigSerde> for OracleConfig {
    type Error = SerdeConversionError;
    fn try_from(c: OracleConfigSerde) -> Result<Self, Self::Error> {
        let (oracle_contract_parameters, oracle_contract_prefix) =
            <(OracleContractParameters, NetworkPrefix)>::try_from(c.oracle_contract_parameters)?;

        let (pool_contract_parameters, pool_contract_prefix) =
            <(PoolContractParameters, NetworkPrefix)>::try_from(c.pool_contract_parameters)?;

        let (refresh_contract_parameters, refresh_contract_prefix) =
            <(RefreshContractParameters, NetworkPrefix)>::try_from(c.refresh_contract_parameters)?;

        let (update_contract_parameters, update_contract_prefix) =
            <(UpdateContractParameters, NetworkPrefix)>::try_from(c.update_contract_parameters)?;

        let (ballot_token_owner_address, network_prefix) = {
            let a = AddressEncoder::unchecked_parse_network_address_from_str(
                &c.ballot_parameters.ballot_token_owner_address,
            )?;
            (a.address(), a.network())
        };
        let (ballot_contract_parameters, ballot_contract_prefix) =
            <(BallotContractParameters, NetworkPrefix)>::try_from(
                c.ballot_parameters.contract_parameters,
            )?;
        let ballot_parameters = BallotBoxWrapperParameters {
            contract_parameters: ballot_contract_parameters,
            vote_parameters: c.ballot_parameters.vote_parameters,
            ballot_token_owner_address,
        };

        let addresses_with_prefix = AddressesWithPrefix::try_from(c.addresses)?;

        if addresses_with_prefix.prefix == network_prefix
            && ballot_contract_prefix == network_prefix
            && update_contract_prefix == network_prefix
            && refresh_contract_prefix == network_prefix
            && oracle_contract_prefix == network_prefix
            && pool_contract_prefix == network_prefix
        {
            Ok(OracleConfig {
                node_ip: c.node_ip,
                node_port: c.node_port,
                node_api_key: c.node_api_key,
                base_fee: c.base_fee,
                log_level: c.log_level,
                core_api_port: c.core_api_port,
                oracle_address: c.oracle_address,
                data_point_source: c.data_point_source,
                data_point_source_custom_script: c.data_point_source_custom_script,
                oracle_contract_parameters,
                pool_contract_parameters,
                refresh_contract_parameters,
                update_contract_parameters,
                ballot_parameters,
                token_ids: c.token_ids,
                addresses: addresses_with_prefix.addresses,
            })
        } else {
            Err(SerdeConversionError::NetworkPrefixesDiffer)
        }
    }
}

/// Used to (de)serialize `BootstrapConfig` instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapConfigSerde {
    refresh_contract_parameters: RefreshContractParametersSerde,
    pool_contract_parameters: PoolContractParametersSerde,
    update_contract_parameters: UpdateContractParametersSerde,
    ballot_contract_parameters: BallotContractParametersSerde,
    tokens_to_mint: TokensToMint,
    node_ip: String,
    node_port: String,
    node_api_key: String,
    addresses: AddressesSerde,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AddressesSerde {
    address_for_oracle_tokens: String,
    wallet_address_for_chain_transaction: String,
}

struct AddressesWithPrefix {
    addresses: Addresses,
    prefix: NetworkPrefix,
}

impl From<(Addresses, NetworkPrefix)> for AddressesSerde {
    fn from(t: (Addresses, NetworkPrefix)) -> Self {
        let addresses = t.0;
        let prefix = t.1;
        let encoder = AddressEncoder::new(prefix);
        AddressesSerde {
            address_for_oracle_tokens: encoder.address_to_str(&addresses.address_for_oracle_tokens),
            wallet_address_for_chain_transaction: encoder
                .address_to_str(&addresses.wallet_address_for_chain_transaction),
        }
    }
}

impl TryFrom<AddressesSerde> for AddressesWithPrefix {
    type Error = AddressEncoderError;
    fn try_from(addresses: AddressesSerde) -> Result<Self, Self::Error> {
        let oracle_token_network_addr = AddressEncoder::unchecked_parse_network_address_from_str(
            &addresses.address_for_oracle_tokens,
        )?;
        let prefix = oracle_token_network_addr.network();
        let wallet_address_for_chain_transaction = AddressEncoder::new(prefix)
            .parse_address_from_str(&addresses.wallet_address_for_chain_transaction)?;
        Ok(AddressesWithPrefix {
            addresses: Addresses {
                address_for_oracle_tokens: oracle_token_network_addr.address(),
                wallet_address_for_chain_transaction,
            },
            prefix,
        })
    }
}

impl From<(BootstrapConfig, NetworkPrefix)> for BootstrapConfigSerde {
    fn from(t: (BootstrapConfig, NetworkPrefix)) -> Self {
        let c = t.0;
        let prefix = t.1;
        BootstrapConfigSerde {
            refresh_contract_parameters: RefreshContractParametersSerde::from(
                c.refresh_contract_parameters,
            ),
            pool_contract_parameters: PoolContractParametersSerde::from(c.pool_contract_parameters),
            update_contract_parameters: UpdateContractParametersSerde::from(
                c.update_contract_parameters,
            ),
            ballot_contract_parameters: BallotContractParametersSerde::from(
                c.ballot_contract_parameters,
            ),
            tokens_to_mint: c.tokens_to_mint,
            node_ip: c.node_ip,
            node_port: c.node_port,
            node_api_key: c.node_api_key,
            addresses: AddressesSerde::from((c.addresses, prefix)),
        }
    }
}

impl TryFrom<BootstrapConfigSerde> for BootstrapConfig {
    type Error = SerdeConversionError;

    fn try_from(c: BootstrapConfigSerde) -> Result<Self, Self::Error> {
        let (pool_contract_parameters, pool_contract_prefix) =
            <(PoolContractParameters, NetworkPrefix)>::try_from(c.pool_contract_parameters)?;
        let (refresh_contract_parameters, refresh_contract_prefix) =
            <(RefreshContractParameters, NetworkPrefix)>::try_from(c.refresh_contract_parameters)?;
        let (update_contract_parameters, update_contract_prefix) =
            <(UpdateContractParameters, NetworkPrefix)>::try_from(c.update_contract_parameters)?;
        let (ballot_contract_parameters, ballot_contract_prefix) =
            <(BallotContractParameters, NetworkPrefix)>::try_from(c.ballot_contract_parameters)?;
        let AddressesWithPrefix { addresses, prefix } = AddressesWithPrefix::try_from(c.addresses)?;

        if pool_contract_prefix == prefix
            && refresh_contract_prefix == prefix
            && update_contract_prefix == prefix
            && ballot_contract_prefix == prefix
        {
            Ok(BootstrapConfig {
                pool_contract_parameters,
                refresh_contract_parameters,
                update_contract_parameters,
                ballot_contract_parameters,
                tokens_to_mint: c.tokens_to_mint,
                node_ip: c.node_ip,
                node_port: c.node_port,
                node_api_key: c.node_api_key,
                addresses,
            })
        } else {
            Err(SerdeConversionError::NetworkPrefixesDiffer)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleContractParametersSerde {
    p2s: String,
    pool_nft_index: usize,
}

impl From<OracleContractParameters> for OracleContractParametersSerde {
    fn from(p: OracleContractParameters) -> Self {
        OracleContractParametersSerde {
            p2s: AddressEncoder::new(NetworkPrefix::Mainnet).address_to_str(&p.p2s),
            pool_nft_index: p.pool_nft_index,
        }
    }
}

impl TryFrom<OracleContractParametersSerde> for (OracleContractParameters, NetworkPrefix) {
    type Error = AddressEncoderError;
    fn try_from(contract: OracleContractParametersSerde) -> Result<Self, Self::Error> {
        let a = AddressEncoder::unchecked_parse_network_address_from_str(&contract.p2s)?;

        Ok((
            OracleContractParameters {
                p2s: a.address(),
                pool_nft_index: contract.pool_nft_index,
            },
            a.network(),
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PoolContractParametersSerde {
    p2s: String,
    refresh_nft_index: usize,
    update_nft_index: usize,
}

impl From<PoolContractParameters> for PoolContractParametersSerde {
    fn from(p: PoolContractParameters) -> Self {
        PoolContractParametersSerde {
            p2s: AddressEncoder::new(NetworkPrefix::Mainnet).address_to_str(&p.p2s),
            refresh_nft_index: p.refresh_nft_index,
            update_nft_index: p.update_nft_index,
        }
    }
}

impl TryFrom<PoolContractParametersSerde> for (PoolContractParameters, NetworkPrefix) {
    type Error = AddressEncoderError;
    fn try_from(contract: PoolContractParametersSerde) -> Result<Self, Self::Error> {
        let a = AddressEncoder::unchecked_parse_network_address_from_str(&contract.p2s)?;
        Ok((
            PoolContractParameters {
                p2s: a.address(),
                refresh_nft_index: contract.refresh_nft_index,
                update_nft_index: contract.update_nft_index,
            },
            a.network(),
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RefreshContractParametersSerde {
    p2s: String,
    pool_nft_index: usize,
    oracle_token_id_index: usize,
    min_data_points_index: usize,
    min_data_points: u64,
    buffer_index: usize,
    buffer_length: u64,
    max_deviation_percent_index: usize,
    max_deviation_percent: u64,
    epoch_length_index: usize,
    epoch_length: u64,
}

impl From<RefreshContractParameters> for RefreshContractParametersSerde {
    fn from(p: RefreshContractParameters) -> Self {
        RefreshContractParametersSerde {
            p2s: AddressEncoder::new(NetworkPrefix::Mainnet).address_to_str(&p.p2s),
            pool_nft_index: p.pool_nft_index,
            oracle_token_id_index: p.oracle_token_id_index,
            min_data_points_index: p.min_data_points_index,
            min_data_points: p.min_data_points,
            buffer_index: p.buffer_index,
            buffer_length: p.buffer_length,
            max_deviation_percent_index: p.max_deviation_percent_index,
            max_deviation_percent: p.max_deviation_percent,
            epoch_length_index: p.epoch_length_index,
            epoch_length: p.epoch_length,
        }
    }
}

impl TryFrom<RefreshContractParametersSerde> for (RefreshContractParameters, NetworkPrefix) {
    type Error = AddressEncoderError;
    fn try_from(contract: RefreshContractParametersSerde) -> Result<Self, Self::Error> {
        let a = AddressEncoder::unchecked_parse_network_address_from_str(&contract.p2s)?;
        Ok((
            RefreshContractParameters {
                p2s: a.address(),
                pool_nft_index: contract.pool_nft_index,
                oracle_token_id_index: contract.oracle_token_id_index,
                min_data_points_index: contract.min_data_points_index,
                min_data_points: contract.min_data_points,
                buffer_index: contract.buffer_index,
                buffer_length: contract.buffer_length,
                max_deviation_percent_index: contract.max_deviation_percent_index,
                max_deviation_percent: contract.max_deviation_percent,
                epoch_length_index: contract.epoch_length_index,
                epoch_length: contract.epoch_length,
            },
            a.network(),
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BallotContractParametersSerde {
    p2s: String,
    min_storage_rent_index: usize,
    min_storage_rent: u64,
    update_nft_index: usize,
}

impl From<BallotContractParameters> for BallotContractParametersSerde {
    fn from(c: BallotContractParameters) -> Self {
        BallotContractParametersSerde {
            p2s: AddressEncoder::new(NetworkPrefix::Mainnet).address_to_str(&c.p2s),
            min_storage_rent_index: c.min_storage_rent_index,
            min_storage_rent: c.min_storage_rent,
            update_nft_index: c.update_nft_index,
        }
    }
}

impl TryFrom<BallotContractParametersSerde> for (BallotContractParameters, NetworkPrefix) {
    type Error = AddressEncoderError;
    fn try_from(contract: BallotContractParametersSerde) -> Result<Self, Self::Error> {
        let a = AddressEncoder::unchecked_parse_network_address_from_str(&contract.p2s)?;
        Ok((
            BallotContractParameters {
                p2s: a.address(),
                min_storage_rent_index: contract.min_storage_rent_index,
                min_storage_rent: contract.min_storage_rent,
                update_nft_index: contract.update_nft_index,
            },
            a.network(),
        ))
    }
}

/// Used to (de)serialize `OracleContractParameters` instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UpdateContractParametersSerde {
    p2s: String,
    pool_nft_index: usize,
    ballot_token_index: usize,
    min_votes_index: usize,
    min_votes: u64,
}

impl TryFrom<UpdateContractParametersSerde> for (UpdateContractParameters, NetworkPrefix) {
    type Error = AddressEncoderError;

    fn try_from(contract: UpdateContractParametersSerde) -> Result<Self, Self::Error> {
        let a = AddressEncoder::unchecked_parse_network_address_from_str(&contract.p2s)?;
        Ok((
            UpdateContractParameters {
                p2s: a.address(),
                pool_nft_index: contract.pool_nft_index,
                ballot_token_index: contract.ballot_token_index,
                min_votes_index: contract.min_votes_index,
                min_votes: contract.min_votes,
            },
            a.network(),
        ))
    }
}

impl From<UpdateContractParameters> for UpdateContractParametersSerde {
    fn from(p: UpdateContractParameters) -> Self {
        UpdateContractParametersSerde {
            p2s: AddressEncoder::new(NetworkPrefix::Mainnet).address_to_str(&p.p2s),
            pool_nft_index: p.pool_nft_index,
            ballot_token_index: p.ballot_token_index,
            min_votes_index: p.min_votes_index,
            min_votes: p.min_votes,
        }
    }
}

#[derive(Clone, Deserialize)]
pub struct UpdateBootstrapConfigSerde {
    pool_contract_parameters: Option<PoolContractParametersSerde>,
    refresh_contract_parameters: Option<RefreshContractParametersSerde>,
    update_contract_parameters: Option<UpdateContractParametersSerde>,
    tokens_to_mint: UpdateTokensToMint,
    addresses: AddressesSerde,
}

/// The network prefix of the 2nd element is the one in use by the existing oracle pool.
impl TryFrom<(UpdateBootstrapConfigSerde, NetworkPrefix)> for UpdateBootstrapConfig {
    type Error = SerdeConversionError;
    fn try_from(
        t: (UpdateBootstrapConfigSerde, NetworkPrefix),
    ) -> Result<UpdateBootstrapConfig, Self::Error> {
        let c = t.0;

        let existing_network_prefix = t.1;

        // Here we collect the network prefixes of any contract updates, to check for equality with
        // existing_network_prefix.
        let mut prefixes = vec![];

        let pool_contract_parameters = c
            .pool_contract_parameters
            .map(|r| r.try_into())
            .transpose()?;
        if let Some((_, prefix)) = pool_contract_parameters {
            prefixes.push(prefix);
        }

        let refresh_contract_parameters = c
            .refresh_contract_parameters
            .map(|r| r.try_into())
            .transpose()?;
        if let Some((_, prefix)) = refresh_contract_parameters {
            prefixes.push(prefix);
        }

        let update_contract_parameters = c
            .update_contract_parameters
            .map(|r| r.try_into())
            .transpose()?;
        if let Some((_, prefix)) = update_contract_parameters {
            prefixes.push(prefix);
        }

        let AddressesWithPrefix { addresses, prefix } = c.addresses.try_into()?;
        prefixes.push(prefix);

        for p in prefixes {
            if p != existing_network_prefix {
                return Err(SerdeConversionError::NetworkPrefixesDiffer);
            }
        }
        Ok(UpdateBootstrapConfig {
            pool_contract_parameters: pool_contract_parameters
                .map(|p: (PoolContractParameters, NetworkPrefix)| p.0),
            refresh_contract_parameters: refresh_contract_parameters
                .map(|p: (RefreshContractParameters, NetworkPrefix)| p.0),
            update_contract_parameters: update_contract_parameters
                .map(|p: (UpdateContractParameters, NetworkPrefix)| p.0),
            tokens_to_mint: c.tokens_to_mint,
            addresses,
        })
    }
}

pub(crate) fn token_id_as_base64_string<S>(
    value: &TokenId,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let bytes: Vec<u8> = value.clone().into();
    serializer.serialize_str(&base64::encode(bytes))
}

pub(crate) fn token_id_from_base64<'de, D>(deserializer: D) -> Result<TokenId, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    // Interesting fact: `s` can't be of type `&str` otherwise we get the following error at
    // runtime:
    //   "invalid type: string ..., expected a borrowed string"
    let s: String = serde::de::Deserialize::deserialize(deserializer)?;
    TokenId::from_base64(&s).map_err(serde::de::Error::custom)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BallotBoxWrapperParametersSerde {
    contract_parameters: BallotContractParametersSerde,
    vote_parameters: Option<CastBallotBoxVoteParameters>,
    ballot_token_owner_address: String,
}
