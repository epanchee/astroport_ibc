#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, IbcMsg, IbcTimeout, MessageInfo,
    Response, StdError, StdResult, Uint64,
};
use cw2::set_contract_version;
use serde::{Deserialize, Serialize};

use crate::error::ContractError;
use astro_ibc::astroport_governance::assembly::ProposalMessage;
use astro_ibc::astroport_governance::astroport::asset::addr_validate_to_lower;

use crate::ibc::{CHANNEL, IBC_APP_VERSION};
use crate::state::{Config, CONFIG, RESULTS};
use astro_ibc::controller::{ExecuteMsg, InstantiateMsg};

const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(
        deps.storage,
        &Config {
            owner: addr_validate_to_lower(deps.api, &msg.owner)?,
            timeout: msg.timeout,
        },
    )?;
    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if config.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    match msg {
        ExecuteMsg::IbcExecuteProposal { proposal_messages } => {
            let channel_id = CHANNEL
                .load(deps.storage)
                .map_err(|_| StdError::generic_err("The channel is closed"))?;
            let ibc_msg = CosmosMsg::Ibc(IbcMsg::SendPacket {
                channel_id: channel_id.clone(),
                data: to_binary(&proposal_messages)?,
                timeout: IbcTimeout::from(env.block.time.plus_seconds(config.timeout)),
            });

            Ok(Response::new()
                .add_message(ibc_msg)
                .add_attribute("action", "ibc_execute")
                .add_attribute("channel", channel_id))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(_deps: Deps, _env: Env, _msg: Empty) -> Result<Binary, ContractError> {
    unimplemented!()
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: Empty) -> Result<Response, ContractError> {
    Ok(Response::default())
}
