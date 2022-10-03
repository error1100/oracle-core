use crate::oracle_config::ORACLE_CONFIG;
use crate::oracle_state::LiveEpochState;
use crate::oracle_state::StageError;
use crate::pool_commands::PoolCommand;
use anyhow::Result;

pub struct EpochState {
    epoch_start_height: u64,
}

/// Enum for the state that the oracle pool is currently in
#[derive(Debug, Clone)]
pub enum PoolState {
    NeedsBootstrap,
    LiveEpoch(LiveEpochState),
}

pub fn process(pool_state: PoolState, height: u32) -> Result<Option<PoolCommand>, StageError> {
    match pool_state {
        PoolState::NeedsBootstrap => {
            log::warn!(
                "No oracle pool found, needs bootstrap or wait for bootstrap txs to be on-chain"
            );
            Ok(None)
        }
        PoolState::LiveEpoch(live_epoch) => {
            let epoch_length = ORACLE_CONFIG
                .refresh_box_wrapper_inputs
                .contract_inputs
                .contract_parameters()
                .epoch_length() as u32;
            if let Some(local_datapoint_box_state) = live_epoch.local_datapoint_box_state {
                if local_datapoint_box_state.epoch_id != live_epoch.epoch_id {
                    log::info!("Height {height}. Publishing datapoint. Last datapoint was published at {}, current epoch id is {})...", local_datapoint_box_state.epoch_id, live_epoch.epoch_id);
                    Ok(Some(PoolCommand::PublishSubsequentDataPoint {
                        republish: false,
                    }))
                } else if local_datapoint_box_state.height < height - epoch_length {
                    log::info!(
                        "Height {height}. Re-publishing datapoint (last one is too old, at {})...",
                        local_datapoint_box_state.height
                    );
                    Ok(Some(PoolCommand::PublishSubsequentDataPoint {
                        republish: true,
                    }))
                } else if height >= live_epoch.latest_pool_box_height + epoch_length {
                    log::info!("Height {height}. Refresh action. Height {height}. Last epoch id {}, previous epoch started (pool box) at {}", live_epoch.epoch_id, live_epoch.latest_pool_box_height,);
                    Ok(Some(PoolCommand::Refresh))
                } else {
                    Ok(None)
                }
            } else {
                // no last local datapoint posted
                log::info!("Height {height}. Publishing datapoint (first)...");
                Ok(Some(PoolCommand::PublishFirstDataPoint))
            }
        }
    }
}

// TODO: add tests
