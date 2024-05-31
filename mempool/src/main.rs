use std::collections::{HashMap, HashSet};
use tokio::sync::RwLock;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone, Debug)]
struct Transaction {
    id: Uuid,
    dependencies: Vec<Uuid>,
}

#[derive(Default)]
struct Mempool {
    transactions: RwLock<HashMap<Uuid, Transaction>>,
    dependency_graph: RwLock<HashMap<Uuid, HashSet<Uuid>>>,
}

impl Mempool {
    fn new() -> Self {
        Mempool {
            transactions: RwLock::new(HashMap::new()),
            dependency_graph: RwLock::new(HashMap::new()),
        }
    }
}

impl Mempool {
    async fn insert_transaction(&self, transaction: Transaction) {
        let mut transactions = self.transactions.write().await;
        let mut dependency_graph = self.dependency_graph.write().await;

        let tx_id = transaction.id;

        for dep in &transaction.dependencies {
            dependency_graph
                .entry(*dep)
                .or_insert_with(HashSet::new)
                .insert(tx_id);
        }

        transactions.insert(tx_id, transaction);
    }
}

impl Mempool {
    async fn get_executable_transactions(&self) -> Vec<Transaction> {
        let transactions = self.transactions.read().await;
        let dependency_graph = self.dependency_graph.read().await;

        let executable_txs: Vec<Transaction> = transactions
            .values()
            .filter(|tx| {
                tx.dependencies
                    .iter()
                    .all(|dep| !transactions.contains_key(dep))
            })
            .cloned()
            .collect();

        executable_txs
    }
}


#[tokio::main]
async fn main() {
    let mempool = Arc::new(Mempool::new());

    let tx1 = Transaction {
        id: Uuid::new_v4(),
        dependencies: vec![],
    };

    let tx2 = Transaction {
        id: Uuid::new_v4(),
        dependencies: vec![tx1.id],
    };

    let mempool_clone = Arc::clone(&mempool);
    tokio::spawn(async move {
        mempool_clone.insert_transaction(tx1).await;
    });

    let mempool_clone = Arc::clone(&mempool);
    tokio::spawn(async move {
        mempool_clone.insert_transaction(tx2).await;
    });

    // Give some time for transactions to be inserted
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let executable_txs = mempool.get_executable_transactions().await;
    println!("Executable Transactions: {:?}", executable_txs);
}