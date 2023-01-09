use std::collections::BTreeMap;

use parking_lot::Mutex;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct PoolTransaction {
    pub transaction_id: TransactionId,
    pub sender: String,
    pub hash: String,
    pub nonce: u64,
}

#[derive(Clone)]
pub struct ValidPoolTransaction {
    pub transaction: PoolTransaction,
}

#[derive(Clone)]
pub struct NewTransactionEvent {
    pub transaction: ValidPoolTransaction,
}

pub struct TransactionPool {
    transaction_listener: Mutex<Vec<mpsc::Sender<NewTransactionEvent>>>,
    txs: BTreeMap<TransactionId, PoolTransaction>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TransactionId {
    pub sender: u64,
    pub nonce: u64,
}

impl TransactionId {
    pub fn new(sender: u64, nonce: u64) -> Self {
        Self { sender, nonce }
    }
}

#[derive(Debug)]
pub enum PoolError {
    SomeError(String),
}

impl TransactionPool {
    pub fn new() -> Self {
        TransactionPool {
            transaction_listener: Default::default(),
            txs: BTreeMap::new(),
        }
    }
    pub fn add_transaction_listener(&self) -> mpsc::Receiver<NewTransactionEvent> {
        const TX_LISTENER_BUFFER_SIZE: usize = 1024;
        let (tx, rx) = mpsc::channel(TX_LISTENER_BUFFER_SIZE);
        self.transaction_listener.lock().push(tx);
        rx
    }

    pub fn add_transaction(&mut self, transaction: PoolTransaction) -> Result<(), PoolError> {
        match self.txs.insert(transaction.transaction_id, transaction) {
            Some(transaction) => {
                let event = NewTransactionEvent {
                    transaction: ValidPoolTransaction { transaction },
                };
                self.on_new_transaction(event);
            }
            None => todo!(),
        }
        todo!()
    }

    fn on_new_transaction(&self, event: NewTransactionEvent) {
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
