use std::{collections::BTreeMap, sync::Arc};

use felipeum_primitives::transaction::{TransactionId, TransactionSigned};
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// Event fired when a new block was mined
#[derive(Debug, Clone)]
pub struct OnNewBlockEvent {
    pub hash: String,
    pub mined_transactions: Vec<PoolTransaction>,
}

/// Contains all state changes after a [`OnNewBlockEvent`] was processed
#[derive(Debug, Clone)]
pub struct OnNewBlockOutcome {
    pub block_hash: String,
    pub mined: Vec<PoolTransaction>,
}

#[derive(Debug, Clone)]
pub struct Pool {
    pool: Arc<PoolInner>,
}

impl Pool {
    pub fn new() -> Self {
        Self {
            pool: Arc::new(PoolInner::new()),
        }
    }

    // TODO: add on_new_block to call when new tx needs to be removed from pool
    // also define a better struct to blocks in the chain etc..
    pub fn on_new_block(&self, event: OnNewBlockEvent) {
        self.pool.on_new_block(event);
    }

    pub fn add_transaction(&self, tx: PoolTransaction) -> Result<PoolTransaction, PoolError> {
        self.pool.add_transaction(tx)
    }

    pub fn add_transaction_listener(&self) -> mpsc::Receiver<NewTransactionEvent> {
        self.pool.add_transaction_listener()
    }

    pub fn get_all(&self) -> Vec<Arc<PoolTransaction>> {
        self.pool.get_all()
    }

    pub fn get(&self, key: TransactionId) -> Option<PoolTransaction> {
        self.pool.get(key)
    }
}

#[derive(Debug)]
pub struct PoolInner {
    pool: RwLock<TxPool>,
    event_listener: Mutex<Vec<mpsc::Sender<OnNewBlockOutcome>>>,
    transaction_listener: Mutex<Vec<mpsc::Sender<NewTransactionEvent>>>,
}
impl PoolInner {
    pub fn new() -> Self {
        Self {
            pool: RwLock::new(TxPool::new()),
            event_listener: Default::default(),
            transaction_listener: Default::default(),
        }
    }

    // TODO: do I even need at this point to notify then on_new_block was process by the pool?
    // this will probably be done by the block provider
    pub fn add_event_listener(&self) -> mpsc::Receiver<OnNewBlockOutcome> {
        const EVENT_LISTENER_BUFFER_SIZE: usize = 1024;
        let (tx, rx) = mpsc::channel(EVENT_LISTENER_BUFFER_SIZE);
        self.event_listener.lock().push(tx);
        rx
    }

    pub fn on_new_block(&self, event: OnNewBlockEvent) {
        let outcome = self.pool.write().on_new_block(event);
        self.notify_on_new_block(outcome);
    }

    // TODO: do I even need at this point to notify then on_new_block was process by the pool?
    // this will probably be done by the block provider
    pub fn notify_on_new_block(&self, event: OnNewBlockOutcome) {
        let mut event_listeners = self.event_listener.lock();

        event_listeners.retain_mut(|listener| match listener.try_send(event.clone()) {
            Ok(()) => true,
            Err(err) => {
                if matches!(err, mpsc::error::TrySendError::Full(_)) {
                    true
                } else {
                    false
                }
            }
        });
    }

    pub fn add_transaction_listener(&self) -> mpsc::Receiver<NewTransactionEvent> {
        const TX_LISTENER_BUFFER_SIZE: usize = 1024;
        let (tx, rx) = mpsc::channel(TX_LISTENER_BUFFER_SIZE);
        self.transaction_listener.lock().push(tx);
        rx
    }

    pub fn add_transaction(&self, tx: PoolTransaction) -> Result<PoolTransaction, PoolError> {
        let added = self.pool.write().add_transaction(tx);
        match added {
            Ok(transaction) => {
                let pool_transaction = PoolTransaction::from(transaction.clone());
                let event = NewTransactionEvent {
                    transaction: pool_transaction,
                };
                self.on_new_transaction(event);
                Ok(transaction)
            }
            Err(err) => Err(err),
        }
    }

    pub fn get(&self, key: TransactionId) -> Option<PoolTransaction> {
        self.pool.read().get(key)
    }

    pub fn get_all(&self) -> Vec<Arc<PoolTransaction>> {
        self.pool.read().get_all()
    }

    pub fn on_new_transaction(&self, event: NewTransactionEvent) {
        let mut transaction_listeners = self.transaction_listener.lock();

        transaction_listeners.retain_mut(|listener| match listener.try_send(event.clone()) {
            Ok(()) => true,
            Err(err) => {
                if matches!(err, mpsc::error::TrySendError::Full(_)) {
                    true
                } else {
                    false
                }
            }
        });
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PoolTransaction {
    pub transaction: TransactionSigned,
    pub transaction_id: TransactionId,
    // todo: add origin, cost, etc..
}

impl From<TransactionSigned> for PoolTransaction {
    fn from(transaction: TransactionSigned) -> Self {
        let transaction_id = TransactionId::new(
            transaction.transaction.from.clone(),
            transaction.transaction.nonce,
        );
        Self {
            transaction,
            transaction_id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct NewTransactionEvent {
    pub transaction: PoolTransaction,
}

#[derive(Debug)]
pub enum PoolError {
    DiscardedOnInsert(String),
}

impl PoolError {
    pub fn hash(&self) -> String {
        match self {
            PoolError::DiscardedOnInsert(hash) => hash.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TxPool {
    txs: BTreeMap<TransactionId, PoolTransaction>,
}

impl TxPool {
    pub fn new() -> Self {
        TxPool {
            txs: BTreeMap::new(),
        }
    }

    fn remove_transaction(&mut self, tx: &PoolTransaction) -> Option<PoolTransaction> {
        let internal = self.txs.remove(&tx.transaction_id)?;

        Some(internal)
    }

    pub fn on_new_block(&mut self, event: OnNewBlockEvent) -> OnNewBlockOutcome {
        for tx in &event.mined_transactions {
            self.remove_transaction(tx);
        }

        OnNewBlockOutcome {
            block_hash: event.hash,
            mined: event.mined_transactions,
        }
    }

    pub fn get_all(&self) -> Vec<Arc<PoolTransaction>> {
        self.txs.iter().map(|(_, v)| Arc::new(v.clone())).collect()
    }

    pub fn get(&self, key: TransactionId) -> Option<PoolTransaction> {
        self.txs.get(&key).map(|tx| tx.clone())
    }

    pub fn add_transaction(
        &mut self,
        transaction: PoolTransaction,
    ) -> Result<PoolTransaction, PoolError> {
        match self
            .txs
            .insert(transaction.transaction_id.clone(), transaction.clone())
        {
            Some(transaction) => Ok(transaction),
            None => return Err(PoolError::DiscardedOnInsert(transaction.transaction.hash)),
        }
    }
}
