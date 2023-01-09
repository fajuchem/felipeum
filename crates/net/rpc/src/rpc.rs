use felipeum_transaction_pool::pool::{PoolTransaction, TransactionId, TransactionPool};
use jsonrpsee::core::{async_trait, RpcResult};
use std::net::SocketAddr;

use jsonrpsee::proc_macros::rpc;
use jsonrpsee::server::ServerBuilder;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRequest {
    pub from: String,
    pub to: String,
    pub value: u64,
}

#[rpc(server)]
pub trait RpcSpec {
    #[method(name = "sendTransaction")]
    async fn send_transaction(&self, tx: TransactionRequest) -> RpcResult<String>;
}

struct RpcServer {
    transaction_pool: TransactionPool,
}

#[async_trait]
impl RpcSpecServer for RpcServer {
    async fn send_transaction(&self, tx: TransactionRequest) -> RpcResult<String> {
        let pool_transaction = PoolTransaction {
            transaction_id: TransactionId::new(10, 1),
            sender: tx.from,
            hash: "".to_string(),
            nonce: 1,
        };
        {
            self.transaction_pool.add_transaction(pool_transaction);
        }
        Ok("hi".to_string())
    }
}

impl RpcServer {
    pub fn new(transaction_pool: TransactionPool) -> Self {
        RpcServer { transaction_pool }
    }
}

pub async fn run_server() -> anyhow::Result<SocketAddr> {
    let server = ServerBuilder::default().build("127.0.0.1:4500").await?;

    let transaction_pool = TransactionPool::new();
    let rpc_server = RpcServer::new(transaction_pool);
    let addr = server.local_addr()?;
    let handle = server.start(rpc_server.into_rpc())?;

    // In this example we don't care about doing shutdown so let's it run forever.
    // You may use the `ServerHandle` to shut it down or manage it yourself.
    tokio::spawn(handle.stopped());

    Ok(addr)
}
