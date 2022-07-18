mod block;
mod blockchain;
mod node;
mod p2p;
mod transaction;

use blockchain::Blockchain;
use dialoguer::{theme::ColorfulTheme, Select};
use tokio::sync::mpsc::{UnboundedSender};
use p2p::{AppBehaviour, ChainResponse};
use rand::seq::SliceRandom;
use std::time::{Duration, SystemTime};
use std::{
    thread,
};
use transaction::Transaction;

use libp2p::{
    core::upgrade,
    futures::StreamExt,
    identity, mplex,
    noise::{Keypair, NoiseConfig, X25519Spec},
    swarm::{Swarm, SwarmBuilder},
    tcp::TokioTcpConfig,
    PeerId, Transport,
};
use tokio::{
    select, spawn,
    sync::{mpsc},
};

pub fn handle_print_chain(chain: &Blockchain) {
    println!("{}", chain);
}

pub async fn swarm_factory(
    node: node::Node,
    rsp_sender: UnboundedSender<ChainResponse>,
    ) -> SwarmBuilder<AppBehaviour> {
    let id_keys = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(id_keys.public());

    let auth_keys = Keypair::<X25519Spec>::new()
        .into_authentic(&id_keys)
        .expect("can create auth keys");

    let transp = TokioTcpConfig::new()
        .upgrade(upgrade::Version::V1)
        .authenticate(NoiseConfig::xx(auth_keys).into_authenticated())
        .multiplex(mplex::MplexConfig::new())
        .boxed();

    let behaviour =
        p2p::AppBehaviour::new(peer_id, node, rsp_sender).await;

    let swarm = SwarmBuilder::new(transp, behaviour, peer_id).executor(Box::new(|fut| {
        spawn(fut);
    }));

    swarm
}

#[tokio::main]
async fn main() {
    let selections = &[
        "Create block",
        "View local blockchain",
        "Generate transaction",
        "View nodes",
        "View pending txs",
    ];

    let blockchain = Blockchain::new(0, 3, 256);
    let node = node::Node { 
        blockchain: blockchain,
        last_time_synced: 0.0,
    };
    let mut pending_txs: Vec<Transaction> = vec![];

    let (response_sender, mut response_rcv) = mpsc::unbounded_channel();
    let (init_sender, mut init_rcv) = mpsc::unbounded_channel();

    let (cli_sender, mut cli_rcv) = mpsc::unbounded_channel();

    let mut swarm = swarm_factory(node, response_sender)
        .await
        .build();

    Swarm::listen_on(
        &mut swarm,
        "/ip4/0.0.0.0/tcp/0"
            .parse()
            .expect("can get a local socket"),
    )
    .expect("swarm can be started");

 
    // Wallet num is peer id
    let wallen_num = swarm.behaviour().peer_id;
    thread::spawn(move || loop {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .clear(true)
            .with_prompt(format!("Your wallet num is {}\r\nPick option\r\n", wallen_num))
            .default(0)
            .items(&selections[..])
            .interact()
            .unwrap();

        cli_sender.send(selection).unwrap();

        // block sync only on interaction
        init_sender.send(true).expect("can send msg to init channel");

    });

    loop {
        let mut selection = 99;
        let evt = {
            select! {
                response = response_rcv.recv() => {
                    Some(p2p::EventType::LocalChainResponse(response.expect("response exists")))
                },
                _init = init_rcv.recv() => {
                    Some(p2p::EventType::Init)
                },
                _selection = cli_rcv.recv() => {
                    selection = _selection.unwrap();
                    Some(p2p::EventType::Cli)

                },
                _event = swarm.select_next_some() => {
                    None
                },
            }
        };

        if let Some(event) = evt {
            match event {
                p2p::EventType::Init => {
                    let topic = swarm.behaviour_mut().blockchain_topic.clone();
                    let peers = p2p::get_list_peers(&swarm);
                    if !peers.is_empty() {
                        let req = p2p::LocalChainRequest {
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
                            .publish(topic, json.as_bytes());
                    }
                }
                p2p::EventType::LocalChainResponse(resp) => {
                    let topic = swarm.behaviour_mut().blockchain_topic.clone();
                    let json = serde_json::to_string(&resp).expect("can jsonify response");
                    swarm
                        .behaviour_mut()
                        .floodsub
                        .publish(topic, json.as_bytes());
                }
                p2p::EventType::Cli => {
                    // let selection = cli_rcv.recv().await.unwrap();
                    if selection == 0 {
                        clearscreen::clear().expect("failed to clear screen");
                        thread::sleep(Duration::from_millis(100));

                        let suc = swarm
                            .behaviour_mut()
                            .node
                            .blockchain
                            .try_mine(pending_txs.clone());
                        if suc {
                            // IF successfull mining, then we broadcast the block to the network
                            // https://www.oreilly.com/library/view/mastering-bitcoin/9781491902639/ch08.html
                            // However, there is no complex logic like orphans blocks or mempool here yet.
                            pending_txs.clear();
                            let topic = swarm.behaviour_mut().blockchain_topic.clone();
                            let json = serde_json::to_string(
                                &swarm.behaviour_mut().node.blockchain.chain.last(),
                            )
                            .expect("can jsonify request");

                            swarm
                                .behaviour_mut()
                                .floodsub
                                .publish(topic, json.as_bytes());
                        }
                    }
                    if selection == 1 {
                        clearscreen::clear().expect("failed to clear screen");
                        thread::sleep(Duration::from_millis(100));
                        print!("Last time from syncing chains: {}. \r\n\r\n", &swarm.behaviour_mut().node.last_time_synced);
                        handle_print_chain(&swarm.behaviour_mut().node.blockchain);
                        print!("\n");
                    }
                    if selection == 2 {
                        clearscreen::clear().expect("failed to clear screen");

                        // We will send 100 coins to random peer
                        let peers = p2p::get_list_peers(&swarm);
                        let to = peers.choose(&mut rand::thread_rng());

                        let transaction = Transaction {
                            from: wallen_num.to_string().clone(),
                            to: to.unwrap().to_string(),
                            amount: 100,
                            time: SystemTime::now(),
                        };
                        thread::sleep(Duration::from_millis(100));
                        println!("Generated tx \n {}", transaction);

                        pending_txs.push(transaction);
                    }
                    if selection == 3 {
                        clearscreen::clear().expect("failed to clear screen");
                        let peers = p2p::get_list_peers(&swarm);
                        thread::sleep(Duration::from_millis(100));
                        print!("Peers len {}. Peers list: \r\n", peers.len());
                        for mut peer in peers {
                            peer = peer.split_whitespace().collect();
                            print!("{}\r\n", peer);
                        }
                        print!("\n");
                    }
                    if selection == 4 {
                        clearscreen::clear().expect("failed to clear screen");
                        thread::sleep(Duration::from_millis(100));
                        print!("Total txs {}. Tx list: \r\n", pending_txs.len());
                        for i in 0..pending_txs.len() {
                            print!("{}. {} \r\n", i + 1, &pending_txs[i]);
                        }
                        print!("\n");
                    }
                }
            }
        }
    }
}
