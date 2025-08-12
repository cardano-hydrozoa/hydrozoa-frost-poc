// This file is auto-generated.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

pub use tx3_sdk::trp::{ClientOptions,ArgValue};
use tx3_sdk::trp::{ProtoTxRequest, TirInfo, TxEnvelope, SubmitParams, SubmitResponse, SubmitWitness};

pub const DEFAULT_TRP_ENDPOINT: &str = "http://localhost:8164";

pub const DEFAULT_HEADERS: &[(&str, &str)] = &[
];

pub const DEFAULT_ENV_ARGS: &[(&str, &str)] = &[
];

pub const FUND_IR: &str = "0d03000106736f757263650d0206736f757263650d0106666175636574050000000000010d010566726f737405000e0210010d0206736f757263650d010666617563657405000000000d0300000000000000";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundParams {
    pub faucet: ArgValue,
    pub frost: ArgValue,
}
impl FundParams {
    fn to_map(&self) -> HashMap<String, ArgValue> {
        let mut map = HashMap::new();

        map.insert("faucet".to_string(), self.faucet.clone());
        map.insert("frost".to_string(), self.frost.clone());

        map
    }
}

pub const SPEND_IR: &str = "0d030001066c6f636b65640d02066c6f636b65640f00091cbd3ae991b5aafccafe5ca70758bd36a9b2f872f57f6d3a1ffa0eb777000000000d010872656465656d657204010d010666617563657405000e0210010d02066c6f636b65640f00091cbd3ae991b5aafccafe5ca70758bd36a9b2f872f57f6d3a1ffa0eb777000000000d03000000010e706c757475735f7769746e657373020673637269707404125101010023259800a518a4d136564004ae690776657273696f6e0506010d020a636f6c6c61746572616c0d010566726f7374050d030000010000";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpendParams {
    pub faucet: ArgValue,
    pub frost: ArgValue,
    pub redeemer: ArgValue,
}
impl SpendParams {
    fn to_map(&self) -> HashMap<String, ArgValue> {
        let mut map = HashMap::new();

        map.insert("faucet".to_string(), self.faucet.clone());
        map.insert("frost".to_string(), self.frost.clone());
        map.insert("redeemer".to_string(), self.redeemer.clone());

        map
    }
}

pub struct Client {
    client: tx3_sdk::trp::Client,
}

impl Client {
    pub fn new(options: ClientOptions) -> Self {
        Self {
            client: tx3_sdk::trp::Client::new(options),
        }
    }

    pub fn with_default_options() -> Self {
        let mut headers = HashMap::new();
        for (key, value) in DEFAULT_HEADERS {
            headers.insert(key.to_string(), value.to_string());
        }

        let mut env_args: HashMap<String, ArgValue> = HashMap::new();
        for (key, value) in DEFAULT_ENV_ARGS {
            env_args.insert(key.to_string(), ArgValue::String(value.to_string()));
        }

        Self::new(ClientOptions {
            endpoint: DEFAULT_TRP_ENDPOINT.to_string(),
            headers: Some(headers),
            env_args: Some(env_args),
        })
    }

    pub async fn fund_tx(&self, args: FundParams) -> Result<TxEnvelope, tx3_sdk::trp::Error> {
        let tir_info = TirInfo {
            bytecode: FUND_IR.to_string(),
            encoding: "hex".to_string(),
            version: "v1alpha8".to_string(),
        };

        self.client.resolve(ProtoTxRequest {
            tir: tir_info,
            args: args.to_map(),
        }).await
    }

    pub async fn spend_tx(&self, args: SpendParams) -> Result<TxEnvelope, tx3_sdk::trp::Error> {
        let tir_info = TirInfo {
            bytecode: SPEND_IR.to_string(),
            encoding: "hex".to_string(),
            version: "v1alpha8".to_string(),
        };

        self.client.resolve(ProtoTxRequest {
            tir: tir_info,
            args: args.to_map(),
        }).await
    }

    pub async fn submit(&self, tx: TxEnvelope, witnesses: Vec<SubmitWitness>) -> Result<SubmitResponse, tx3_sdk::trp::Error> {
        self.client.submit(tx, witnesses).await
    }
}

// Create a default client instance
pub static PROTOCOL: once_cell::sync::Lazy<Client> = once_cell::sync::Lazy::new(|| Client::with_default_options());
