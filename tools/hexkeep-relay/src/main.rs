use futures::StreamExt;
use libp2p::{
    identify, noise, ping, relay,
    swarm::{NetworkBehaviour, SwarmEvent},
    tcp, yamux,
};
#[derive(NetworkBehaviour)]
struct Behaviour {
    relay: relay::Behaviour,
    identify: identify::Behaviour,
    ping: ping::Behaviour,
}
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::args()
        .nth(1)
        .unwrap_or("4001".into())
        .parse::<u16>()?;
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default().nodelay(true),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_quic()
        .with_behaviour(|key| {
            let peer = key.public().to_peer_id();
            let config = relay::Config {
                max_reservations: 128,
                max_circuits: 256,
                max_circuit_duration: std::time::Duration::from_secs(3600),
                max_circuit_bytes: 64 * 1024 * 1024,
                ..Default::default()
            };
            Behaviour {
                relay: relay::Behaviour::new(peer, config),
                identify: identify::Behaviour::new(identify::Config::new(
                    "/hexkeep/blind-relay/2".into(),
                    key.public(),
                )),
                ping: ping::Behaviour::new(Default::default()),
            }
        })?
        .build();
    swarm.listen_on(format!("/ip4/0.0.0.0/tcp/{port}").parse()?)?;
    swarm.listen_on(format!("/ip4/0.0.0.0/udp/{port}/quic-v1").parse()?)?;
    loop {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                // Reservations may arrive before Identify reports our address.
                // Include our listening addresses in the very first response.
                swarm.add_external_address(address.clone());
                println!("RELAY {}/p2p/{}", address, swarm.local_peer_id());
            }
            SwarmEvent::Behaviour(BehaviourEvent::Relay(event)) => {
                println!("CIRCUIT {event:?}");
            }
            SwarmEvent::Behaviour(BehaviourEvent::Identify(identify::Event::Received {
                info,
                ..
            })) => {
                swarm.add_external_address(info.observed_addr);
            }
            _ => {}
        }
    }
}
