use libp2p::{
    core::upgrade,
    gossipsub::{self, Gossipsub, GossipsubEvent, IdentTopic, MessageAuthenticity, ValidationMode},
    identity,
    noise,
    swarm::SwarmBuilder,
    tcp::TokioTcpConfig,
    yamux, Multiaddr, PeerId, Swarm, Transport,
};
use tokio::sync::mpsc;
use crate::network::message::NetworkMessage;

use std::error::Error;

pub struct Network {
    pub swarm: Swarm<Gossipsub>,
    pub topic: IdentTopic,
}

impl Network {
    pub async fn new() -> Result<(Self, mpsc::Receiver<NetworkMessage>), Box<dyn Error>> {
        let local_key = identity::Keypair::generate_ed25519(); // You can replace with Dilithium in future
        let peer_id = PeerId::from(local_key.public());
        println!("Local peer id: {:?}", peer_id);

        let noise_keys = noise::Keypair::<noise::X25519Spec>::new().into_authentic(&local_key)?;

        let transport = TokioTcpConfig::new()
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::NoiseConfig::xx(noise_keys).into_authenticated())
            .multiplex(yamux::YamuxConfig::default())
            .boxed();

        let mut gossipsub_config = gossipsub::GossipsubConfigBuilder::default()
            .validation_mode(ValidationMode::Permissive)
            .build()
            .unwrap();

        let mut gossipsub = Gossipsub::new(MessageAuthenticity::Signed(local_key.clone()), gossipsub_config)?;
        let topic = IdentTopic::new("quantumcoin");

        gossipsub.subscribe(&topic)?;

        let (tx, rx) = mpsc::channel(32);

        let mut swarm = SwarmBuilder::new(transport, gossipsub, peer_id)
            .executor(Box::new(|fut| {
                tokio::spawn(fut);
            }))
            .build();

        tokio::spawn(Self::poll_events(swarm.clone(), tx.clone(), topic.clone()));

        Ok((
            Self {
                swarm,
                topic,
            },
            rx,
        ))
    }

    async fn poll_events(mut swarm: Swarm<Gossipsub>, tx: mpsc::Sender<NetworkMessage>, topic: IdentTopic) {
        loop {
            if let Some(event) = swarm.next().await {
                if let GossipsubEvent::Message { message, .. } = event {
                    if let Ok(msg) = serde_json::from_slice::<NetworkMessage>(&message.data) {
                        let _ = tx.send(msg).await;
                    }
                }
            }
        }
    }

    pub fn publish_message(&mut self, msg: NetworkMessage) {
        if let Ok(json) = serde_json::to_vec(&msg) {
            let _ = self.swarm.behaviour_mut().publish(self.topic.clone(), json);
        }
    }
}