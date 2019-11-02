use serde_json::{Value, json};
use streaming_platform::{magic_ball, Mode, MagicBall, sp_dto::MsgMeta};

fn main() {
    let host = "127.0.0.1:60000";
    let addr = "SuperService2";
    let access_key = "";
    let mode = Mode::FullMessageSimple(process_event, process_rpc_request);

    magic_ball(host, addr, access_key, mode);
}

fn process_event(mb: &mut MagicBall, msg_meta: &MsgMeta, payload: Value, attachments: Vec<u8>) {

}
fn process_rpc_request(mb: &mut MagicBall, msg_meta: &MsgMeta, payload: Value, attachments: Vec<u8>) -> Value {
    json!({})
}
