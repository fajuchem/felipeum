use felipeum_primitives::{
    signature::Signature,
    transaction::{Transaction, TransactionSigned},
};
use felipeum_signature::keypair::new_keypair;
use felipeum_transaction_pool::pool::{Pool, PoolTransaction};
use jsonrpsee::proc_macros::rpc;
use jsonrpsee::server::ServerBuilder;
use jsonrpsee::{
    core::{async_trait, Error, RpcResult},
    tracing::info,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InnerTransactionRequest {
    pub from: String,
    pub to: String,
    pub value: u64,
    pub nonce: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRequest {
    pub transaction: InnerTransactionRequest,
    pub signature: String,
}

#[rpc(server)]
pub trait RpcSpec {
    #[method(name = "sendTransaction")]
    async fn send_transaction(&self, tx: TransactionRequest) -> RpcResult<String>;

    #[method(name = "newAccount")]
    async fn new_account(&self) -> RpcResult<NewAccount>;
}

struct RpcServer {
    transaction_pool: Pool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NewAccount {
    pub public_key: String,
    pub private_key: String,
}

#[async_trait]
impl RpcSpecServer for RpcServer {
    async fn send_transaction(&self, tx: TransactionRequest) -> RpcResult<String> {
        let transaction = Transaction {
            from: tx.transaction.from,
            to: tx.transaction.to,
            nonce: tx.transaction.nonce,
        };
        info!("transaction: {:?}", transaction);

        // let encoded = transaction.signature_hash();
        // info!("encoded: {:?}", encoded);

        // let hash = std::str::from_utf8(&encoded).unwrap().to_string();
        // info!("hash: {:?}", hash);

        // let signature = Signature::new(tx.signature.as_bytes());
        // info!("signature: {:?}", signature);
        let signature = Signature::new("asdf".as_bytes());
        let hash = "asdf".to_string();

        let transaction_signed = TransactionSigned {
            transaction,
            hash,
            signature,
        };
        info!("transaction_signed: {:?}", transaction_signed);

        let pool_transaction = PoolTransaction::from(transaction_signed);
        info!("pool_transaction: {:?}", pool_transaction);

        match self.transaction_pool.add_transaction(pool_transaction) {
            Ok(tx) => Ok(tx.transaction.hash),
            Err(msg) => Err(Error::Custom(msg.hash().to_string())),
        }
    }

    async fn new_account(&self) -> RpcResult<NewAccount> {
        match new_keypair() {
            Ok(k) => Ok(NewAccount {
                public_key: hex::encode(k.public_key()),
                private_key: hex::encode(k.secret()),
            }),

            Err(msg) => Err(Error::Custom(msg.to_string())),
        }
    }
}

impl RpcServer {
    pub fn new(transaction_pool: Pool) -> Self {
        RpcServer { transaction_pool }
    }
}

pub async fn run_server(transaction_pool: Pool) -> anyhow::Result<SocketAddr> {
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_origin(Any)
        .allow_headers(Any);
    let middleware = tower::ServiceBuilder::new().layer(cors);

    let server = ServerBuilder::default()
        .set_middleware(middleware)
        .build("127.0.0.1:4500")
        .await?;

    let rpc_server = RpcServer::new(transaction_pool);
    let addr = server.local_addr()?;
    let handle = server.start(rpc_server.into_rpc())?;

    tokio::spawn(handle.stopped());

    Ok(addr)
}
