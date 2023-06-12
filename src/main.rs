use std::{io, path::PathBuf};

use spider_client::{
    AddressStrategy, Relation, Role, SpiderClient, SpiderId2048,
};


mod state;
use state::State;

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    println!("Hello, world!");

    let client_path = PathBuf::from("client_state.dat");
    let mut client = if client_path.exists() {
        SpiderClient::from_file(&client_path)
    } else {
        let mut client = SpiderClient::new();
        client.set_state_path(&client_path);
        client.add_strat(AddressStrategy::Addr(String::from("localhost:1930")));
        client.save();
        client
    };

    if !client.has_host_relation() {
        let path = PathBuf::from("spider_keyfile.json");

        let data = match std::fs::read_to_string(&path) {
            Ok(str) => str,
            Err(_) => String::from("[]"),
        };
        let id: SpiderId2048 = serde_json::from_str(&data).expect("Failed to deserialize spiderid");
        let host = Relation {
            id,
            role: Role::Peer,
        };
        client.set_host_relation(host);
        client.save();
    }

    client.connect().await;
    let mut state = State::new(client).await;
    state.run().await;

    Ok(())
}
