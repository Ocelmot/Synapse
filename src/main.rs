use std::path::PathBuf;

use spider_client::{
    link::{link_set::links::Address, transports::tcp::TCP_SCHEME},
    SpiderClientBuilder,
};

mod state;
use state::State;

#[tokio::main]
async fn main() {
    let client_path = PathBuf::from("client_state.dat");

    let mut builder = SpiderClientBuilder::load_or_set(&client_path, |builder| {
        builder.enable_fixed_addrs(true);
        builder.set_fixed_addrs(vec![Address::new(TCP_SCHEME, "localhost:1930")]);
        builder.enable_transport(TCP_SCHEME.to_owned());
    })
    .await
    .expect("Failed to load config");

    builder.try_use_keyfile("spider_keyfile.json").await;

    let client_channel = builder.start(true).await.expect("failed to start");

    let mut state = State::new(client_channel).await;
    state.run().await;
}
