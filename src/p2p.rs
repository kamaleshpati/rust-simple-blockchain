use libp2p::{
    floodsub,
    floodsub::{Floodsub, FloodsubEvent},
    mdns::{Mdns, MdnsEvent},
    swarm::{NetworkBehaviourEventProcess, Swarm},
    NetworkBehaviour, PeerId,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tokio::sync::mpsc;

use crate::{block::Block, blockchain::Blockchain, node::Node};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChainResponse {
    pub blockchain: Blockchain,
    pub receiver: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LocalChainRequest {
    pub from_peer_id: String,
}

pub enum EventType {
    LocalChainResponse(ChainResponse),
    Init,
    Cli,
}

#[derive(NetworkBehaviour)]
pub struct AppBehaviour {
    pub floodsub: Floodsub,
    pub mdns: Mdns,
    #[behaviour(ignore)]
    pub response_sender: mpsc::UnboundedSender<ChainResponse>,
    #[behaviour(ignore)]
    pub node: Node,
    #[behaviour(ignore)]
    pub peer_id: PeerId,
    #[behaviour(ignore)]
    pub blockchain_topic: floodsub::Topic,
    #[behaviour(ignore)]
    pub transaction_topic: floodsub::Topic,
}

impl AppBehaviour {
    pub async fn new(
        peer_id: PeerId,
        node: Node,
        response_sender: mpsc::UnboundedSender<ChainResponse>,
    ) -> Self {
        let mut behaviour = Self {
            node: node,
            peer_id: peer_id,
            floodsub: Floodsub::new(peer_id),
            mdns: Mdns::new(Default::default())
                .await
                .expect("can create mdns"),

            blockchain_topic: floodsub::Topic::new("blockchain"),
            transaction_topic: floodsub::Topic::new("transactions"),
            response_sender,
        };
        
        behaviour
            .floodsub
            .subscribe(behaviour.blockchain_topic.clone());
        
        behaviour
    }
}

// incoming event handler
impl NetworkBehaviourEventProcess<FloodsubEvent> for AppBehaviour {
    fn inject_event(&mut self, event: FloodsubEvent) {
        if let FloodsubEvent::Message(msg) = event {
            if let Ok(resp) = serde_json::from_slice::<ChainResponse>(&msg.data) {
                if resp.receiver == self.peer_id.to_string() {

                    self.node.resolve_chain_conflict(&resp.blockchain);
                }
            } else if let Ok(resp) = serde_json::from_slice::<LocalChainRequest>(&msg.data) {
                let peer_id = resp.from_peer_id;
                if self.peer_id.to_string() == peer_id {
                    if let Err(e) = self.response_sender.send(ChainResponse {
                        blockchain: self.node.blockchain.clone(),
                        receiver: msg.source.to_string(),
                    }) {
                        println!("error sending response via channel, {} \r\n", e);
                    }
                }
            } else if let Ok(block) = serde_json::from_slice::<Block>(&msg.data) {
                self.node.blockchain.add_block(block);
            }
        }
    }
}

impl NetworkBehaviourEventProcess<MdnsEvent> for AppBehaviour {
    fn inject_event(&mut self, event: MdnsEvent) {
        match event {
            MdnsEvent::Discovered(discovered_list) => {
                for (peer, _addr) in discovered_list {
                    self.floodsub.add_node_to_partial_view(peer);
                }
            }
            MdnsEvent::Expired(expired_list) => {
                for (peer, _addr) in expired_list {
                    if !self.mdns.has_node(&peer) {
                        self.floodsub.remove_node_from_partial_view(&peer);
                    }
                }
            }
        }
    }
}

pub fn get_list_peers(swarm: &Swarm<AppBehaviour>) -> Vec<String> {

    let nodes = swarm.behaviour().mdns.discovered_nodes();
    let mut unique_peers = HashSet::new();
    for peer in nodes {
        unique_peers.insert(peer);
    }
    unique_peers.iter().map(|p| p.to_string()).collect()
}
