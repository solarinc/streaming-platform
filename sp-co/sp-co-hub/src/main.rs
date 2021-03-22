use std::collections::HashMap;
use streaming_platform::{sp_cfg::ServerConfig, server, sp_dto::{Key, Subscribes}};

fn main() {
    env_logger::init();
    
    let config = ServerConfig {
        host: "127.0.0.1:11001".to_owned()
    };

    let event_subscribes = HashMap::new();
    let mut rpc_subscribes = HashMap::new();
    let mut rpc_response_subscribes = HashMap::new();

    rpc_subscribes.insert("Auth".to_owned(), vec![
        Key::new("Auth", "Auth", "Auth")
    ]);

    rpc_subscribes.insert("Build".to_owned(), vec![
        Key::new("Deploy", "Build", "Build")
    ]);

    rpc_subscribes.insert("Pod".to_owned(), vec![
        Key::new("DeployPack", "Pod", "Pod")
    ]);

    rpc_response_subscribes.insert("Build".to_owned(), vec![
        Key::new("DeployPack", "Pod", "Pod")
    ]);

    rpc_response_subscribes.insert("Web".to_owned(), vec![
        Key::new("Auth", "Auth", "Auth"),
        Key::new("Deploy", "Build", "Build")
    ]);

    server::start(config, Subscribes::ByAddr(event_subscribes, rpc_subscribes, rpc_response_subscribes));
}