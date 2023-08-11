use std::path::PathBuf;

use spider_client::SpiderClientBuilder;

mod state;
use state::State;

#[tokio::main]
async fn main() {
    let client_path = PathBuf::from("client_state.dat");

    let mut builder = SpiderClientBuilder::load_or_set(&client_path, |builder| {
        builder.enable_fixed_addrs(true);
        builder.set_fixed_addrs(vec!["localhost:1930".into()]);
    });

    builder.try_use_keyfile("spider_keyfile.json").await;

    let client_channel = builder.start(true);

    let mut state = State::new(client_channel).await;
    state.run().await;
}
