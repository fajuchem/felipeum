use felipeum_p2p::{
    chain::Chain,
    p2p::{
        get_list_peers, handle_create_block, handle_print_chain, handle_print_peers, AppBehaviour,
        EventType, LocalChainRequest, CHAIN_TOPIC, KEYS, PEER_ID, POOL_TX_TOPIC,
    },
};
use felipeum_rpc::rpc::run_server;
use felipeum_transaction_pool::pool::Pool;
use libp2p::{
    core::upgrade,
    futures::StreamExt,
    mplex,
    noise::{Keypair, NoiseConfig, X25519Spec},
    swarm::{Swarm, SwarmBuilder},
    tcp::TokioTcpConfig,
    Transport,
};
use log::{error, info};
use std::time::Duration;
use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    select, spawn,
    sync::mpsc,
    time::sleep,
};

// TODO: kind of replace the MockEthProvider from reth
pub async fn run_executor(pool: Pool, storage: Storage) {
    loop {
        sleep(Duration::from_secs(3)).await;
        info!("executor");
        let all = pool.get_all();
        println!("{:?}", all);
    }
}

#[derive(Clone)]
pub struct Storage {
    blocks: Vec<String>,
}

impl Storage {
    fn new() -> Self {
        Self { blocks: vec![] }
    }

    fn add_block(&mut self, block: String) {
        self.blocks.push(block);
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    // initialize pool
    let pool = Pool::new();

    let mut recv_trans = pool.add_transaction_listener();

    match run_server(pool.clone()).await {
        Ok(server) => format!("http://{}", server),
        Err(msg) => format!("{}", msg),
    };

    // initialize storage
    let storage = Storage::new();

    // itiliaze p2p
    info!("Peer Id: {}", PEER_ID.clone());
    let (response_sender, mut response_rcv) = mpsc::unbounded_channel();
    let (init_sender, mut init_rcv) = mpsc::unbounded_channel();

    let auth_keys = Keypair::<X25519Spec>::new()
        .into_authentic(&KEYS)
        .expect("can create auth keys");

    let transp = TokioTcpConfig::new()
        .upgrade(upgrade::Version::V1)
        .authenticate(NoiseConfig::xx(auth_keys).into_authenticated())
        .multiplex(mplex::MplexConfig::new())
        .boxed();

    let behaviour = AppBehaviour::new(
        Chain::new(pool.clone()),
        response_sender,
        init_sender.clone(),
    )
    .await;
    let mut swarm = SwarmBuilder::new(transp, behaviour, *PEER_ID)
        .executor(Box::new(|fut| {
            spawn(fut);
        }))
        .build();

    let mut stdin = BufReader::new(stdin()).lines();

    Swarm::listen_on(
        &mut swarm,
        "/ip4/0.0.0.0/tcp/0"
            .parse()
            .expect("can get a local socket"),
    )
    .expect("swarm can be started");

    spawn(run_executor(pool.clone(), storage.clone()));

    spawn(async move {
        sleep(Duration::from_secs(1)).await;
        info!("Sending init event");
        init_sender.send(true).expect("can send init event");
    });

    loop {
        let evt = {
            select! {
                tx = recv_trans.recv() => {
                    println!("new tx added in the local pool");
                    match tx {
                        Some(tx) => Some(EventType::NewTx(tx.transaction.transaction)),
                        None => None,
                    }
                },
                line = stdin.next_line() => Some(EventType::Input(line.expect("can get line").expect("can read line from stdin"))),
                response = response_rcv.recv() => {
                    Some(EventType::LocalChainResponse(response.expect("response exists")))
                },
                _init = init_rcv.recv() => {
                    Some(EventType::Init)
                }
                _ = swarm.select_next_some() => {
                    None
                },
            }
        };

        if let Some(event) = evt {
            match event {
                EventType::Init => {
                    let peers = get_list_peers(&swarm);
                    swarm.behaviour_mut().app.genesis();

                    info!("connected nodes: {}", peers.len());
                    if !peers.is_empty() {
                        let req = LocalChainRequest {
                            from_peer_id: peers
                                .iter()
                                .last()
                                .expect("at least one peer")
                                .to_string(),
                        };

                        let json = serde_json::to_string(&req).expect("can jsonify request");
                        swarm
                            .behaviour_mut()
                            .floodsub
                            .publish(CHAIN_TOPIC.clone(), json.as_bytes());
                    }
                }
                EventType::LocalChainResponse(resp) => {
                    let json = serde_json::to_string(&resp).expect("can jsonify response");
                    swarm
                        .behaviour_mut()
                        .floodsub
                        .publish(CHAIN_TOPIC.clone(), json.as_bytes());
                }
                EventType::Input(line) => match line.as_str() {
                    "ls p" => handle_print_peers(&swarm),
                    "ls pool" => {
                        let all = pool.get_all();
                        println!("{:?}", all);
                    }
                    cmd if cmd.starts_with("ls c") => handle_print_chain(&swarm),
                    cmd if cmd.starts_with("create b") => handle_create_block(cmd, &mut swarm),
                    _ => error!("unknown command"),
                },
                EventType::NewTx(new_tx) => {
                    let json = serde_json::to_string(&new_tx).expect("can jsonify response");
                    swarm
                        .behaviour_mut()
                        .floodsub
                        .publish(POOL_TX_TOPIC.clone(), json.as_bytes());
                    println!("send p2p: {:?}", new_tx);
                }
            }
        }
    }
}
