use felipeum_primitives::transaction::{Transaction, TransactionId};
use felipeum_signature::keypair::new_keypair;
use felipeum_transaction_pool::pool::Pool;
use jsonrpsee::core::{async_trait, Error, RpcResult};
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
        let pool_transaction = Transaction {
            transaction_id: TransactionId::new(tx.value, 1),
            sender: tx.from,
            hash: "".to_string(),
            nonce: 1,
        };
        let txs = self.transaction_pool.get_all();
        println!("{:?}", txs);

        match self.transaction_pool.add_transaction(pool_transaction) {
            Ok(tx) => Ok(tx.hash),
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
    let server = ServerBuilder::default().build("127.0.0.1:4500").await?;

    let rpc_server = RpcServer::new(transaction_pool);
    let addr = server.local_addr()?;
    let handle = server.start(rpc_server.into_rpc())?;

    tokio::spawn(handle.stopped());

    Ok(addr)
}
