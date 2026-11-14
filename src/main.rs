mod protocol;
mod receiver;
mod sender;

use iroh::{Endpoint, endpoint::presets};
use iroh_mdns_address_lookup::{DiscoveryEvent, MdnsAddressLookup};
use n0_future::StreamExt;
use std::time::Duration;
use std::collections::HashSet;
use tokio::time::timeout;

const ALPN: &[u8] = b"iroh-file/0";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let peer = std::env::args().nth(1);
    let path = std::env::args().nth(2);

    let endpoint = Endpoint::bind(presets::N0).await?;
    let mdns = MdnsAddressLookup::builder().build(endpoint.id())?;
    endpoint.address_lookup()?.add(mdns.clone());
    
    wait_until_online(&endpoint).await;

    match peer.as_deref() {
        None => receiver::listen(endpoint).await?,
        Some("scan") => scan(&mdns).await,
        Some(id) => {
            let path = path.expect("use like this: cargo run -- <id> <path>");
            sender::send_file(endpoint, id.to_string(), path).await?
        }
    }
    
    Ok(())
}

async fn wait_until_online(endpoint: &Endpoint) {
    match timeout(Duration::from_secs(5), endpoint.online()).await {
        Ok(()) => println!("online"),
        Err(_) => println!("offline"),
        // Jetzt kann mann online und offline.
    }
}

async fn scan(mdns: &MdnsAddressLookup) {
    println!("scanning LAN for 5 seconds...");
    let mut events = mdns.subscribe().await;
    let mut seen = HashSet::new();

    let _ = timeout(Duration::from_secs(5), async {
        while let Some(event) = events.next().await {
            match event {
                DiscoveryEvent::Discovered { endpoint_info, .. } => {
                    if seen.insert(endpoint_info.endpoint_id) {
                        let ips: Vec<_> = endpoint_info.ip_addrs().collect();
                        println!("found {} at {:?}", endpoint_info.endpoint_id, ips)
                    }
                }
                DiscoveryEvent::Expired { endpoint_id } => {
                    seen.remove(&endpoint_id);
                    println!("gone: {endpoint_id}");
                }
                _ => {}
            }
        }
    })
    .await;

    println!("scan done");
}
