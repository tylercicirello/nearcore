#[cfg(feature = "test_features")]
use actix::Actor;
use actix::Addr;

use near_chain_configs::GenesisConfig;
use near_client::test_utils::setup_no_network_with_validity_period_and_no_epoch_sync;
use near_client::ViewClientActor;
use near_jsonrpc::{start_http, RpcConfig};
use near_network::test_utils::open_port;
#[cfg(feature = "test_features")]
use near_network::test_utils::{make_peer_manager, make_routing_table_actor};
#[cfg(feature = "test_features")]
use near_network::NetworkConfig;
#[cfg(feature = "test_features")]
use near_primitives::types::NumBlocks;
#[cfg(feature = "test_features")]
use near_store::test_utils::create_test_store;

lazy_static::lazy_static! {
    pub static ref TEST_GENESIS_CONFIG: GenesisConfig =
        GenesisConfig::from_json(include_str!("../../../../nearcore/res/genesis_config.json"));
}

pub enum NodeType {
    Validator,
    NonValidator,
}

pub fn start_all(node_type: NodeType) -> (Addr<ViewClientActor>, String) {
    start_all_with_validity_period_and_no_epoch_sync(node_type, 100, false)
}

pub fn start_all_with_validity_period_and_no_epoch_sync(
    node_type: NodeType,
    transaction_validity_period: NumBlocks,
    enable_doomslug: bool,
) -> (Addr<ViewClientActor>, String) {
    let (client_addr, view_client_addr) = setup_no_network_with_validity_period_and_no_epoch_sync(
        vec!["test1".parse().unwrap(), "test2".parse().unwrap()],
        if let NodeType::Validator = node_type {
            "test1".parse().unwrap()
        } else {
            "other".parse().unwrap()
        },
        true,
        transaction_validity_period,
        enable_doomslug,
    );

    let port = open_port();
    let addr = format!("127.0.0.1:{}", port);
    let config = RpcConfig::new(&addr);
    #[cfg(feature = "test_features")]
    let seed = "test2";
    #[cfg(feature = "test_features")]
    let net_config = NetworkConfig::from_seed(seed, port);
    #[cfg(feature = "test_features")]
    let store = create_test_store();
    #[cfg(feature = "test_features")]
    let routing_table_addr =
        make_routing_table_actor(net_config.public_key.clone().into(), store.clone());
    #[cfg(feature = "test_features")]
    let peer_manager_addr = make_peer_manager(
        store,
        net_config,
        vec![("test1", open_port())],
        10,
        routing_table_addr.clone(),
    )
    .0
    .start();
    start_http(
        config,
        TEST_GENESIS_CONFIG.clone(),
        client_addr.clone(),
        view_client_addr.clone(),
        #[cfg(feature = "test_features")]
        peer_manager_addr,
        #[cfg(feature = "test_features")]
        routing_table_addr,
    );
    (view_client_addr, addr)
}

#[allow(unused_macros)] // Suppress Rustc warnings even though this macro is used.
macro_rules! test_with_client {
    ($node_type:expr, $client:ident, $block:expr) => {
        init_test_logger();

        run_actix(|| {
            let (_view_client_addr, addr) = test_utils::start_all($node_type);

            let $client = new_client(&format!("http://{}", addr));

            actix::spawn(async move {
                $block.await;
                System::current().stop();
            });
        })
        .unwrap();
    };
}
