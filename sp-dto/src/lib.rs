use std::fmt::Debug;
use bytes::{Buf, BufMut};
use serde_derive::{Serialize, Deserialize};
use serde_json::Error;
use uuid::Uuid;
pub use bytes;
pub use uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CmpSpec {
    pub tx: String,
    pub rx: String,
    pub app_addr: String,
    pub client_addr: String
}

impl CmpSpec {
    // these methods work for child components, note this: tx: self.rx.clone(),
    pub fn new_rx(&self, rx: &str) -> CmpSpec {
        CmpSpec {
            tx: self.rx.clone(),
            rx: rx.to_owned(),
            app_addr: self.app_addr.clone(),
            client_addr: self.client_addr.clone()
        }
    }
    pub fn add_to_rx(&self, delta: &str) -> CmpSpec {
        CmpSpec {
            tx: self.rx.clone(),
            rx: self.rx.clone() + "." + delta,
            app_addr: self.app_addr.clone(),
            client_addr: self.client_addr.clone()
        }
    }
}

impl Default for CmpSpec {
    fn default() -> Self {
        CmpSpec {
            tx: String::new(),
            rx: String::new(),
            app_addr: String::new(),
            client_addr: String::new()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Participator {
    /// UI component
    Component(CmpSpec),
    /// Service process running in background somewhere
    Service(String)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RouteSpec {
    /// Message path ends after it was transfered to rx.
    Simple,
    /// Defines final receiver after multiple message transfers.
    Client(Participator)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Route {
    pub source: Participator,
    pub spec: RouteSpec,
    pub points: Vec<Participator>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MsgMeta {
    /// Addr of message sender
    pub tx: String,
    /// Addr of message receiver
    pub rx: String,
    /// Logic key for message processing
    pub key: String,
    /// Defines what kind of message is is
    pub kind: MsgKind,
    /// Correlation id is needed for rpc and for message chains
    pub correlation_id: Uuid,
    /// Logical message route, receiver are responsible for moving message on.
    pub route: Route,
    /// Size of payload, used for deserialization. Also useful for monitoring.
    pub payload_size: u32,
    /// Attachments to message
	pub attachments: Vec<Attachment>
}

impl MsgMeta {
    /// Zero based index for key part, . is used as a separator.
    pub fn view(&self) -> String {
        format!("{} -> {} {} {:?}", self.tx, self.rx, self.key, self.kind)
    }
    pub fn key_part(&self, index: usize) -> Result<&str, String> {
        let split: Vec<&str> = self.key.split(".").collect();

        if index >= split.len() {
            return Err("index equals or superior to key parts length".to_owned());
        }

        return Ok(split[index])
    }
    /// Compares key part with passed value, index is zero based, . is used as a separator.
    pub fn match_key_part(&self, index: usize, value: &str) -> Result<bool, String> {
        let split: Vec<&str> = self.key.split(".").collect();

        if index >= split.len() {
            return Err("index equals or superior to key parts length".to_owned());
        }

        return Ok(split[index] == value)
    }
    pub fn source_cmp_addr(&self) -> Option<String> {
        match &self.route.source {
            Participator::Component(spec) => Some(spec.rx.clone()),
            _ => None
        }
    }
    pub fn source_svc_addr(&self) -> Option<String> {
        match &self.route.source {
            Participator::Service(addr) => Some(addr.clone()),
            _ => None
        }
    }
    pub fn client_cmp_addr(&self) -> Option<String> {
        match &self.route.spec {
            RouteSpec::Client(participator) => {
                match participator {
                    Participator::Component(spec) => Some(spec.rx.clone()),
                    _ => None
                }
            }
            _ => None
        }
    }
    pub fn client_svc_addr(&self) -> Option<String> {
        match &self.route.spec {
            RouteSpec::Client(participator) => {
                match participator {
                    Participator::Service(addr) => Some(addr.clone()),
                    _ => None
                }
            }
            _ => None
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MsgKind {
    Event,
    RpcRequest,
    RpcResponse
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Attachment {
	pub name: String,
    pub size: u32
}

pub fn event_dto<T>(tx: String, rx: String, key: String, payload: T, route: Route) -> Result<Vec<u8>, Error> where T: Debug, T: serde::Serialize, for<'de> T: serde::Deserialize<'de> {
    let mut payload = serde_json::to_vec(&payload)?;
    let correlation_id = Uuid::new_v4();

    let mut msg_meta = MsgMeta {
        tx,
        rx,
        key,
        kind: MsgKind::Event,
        correlation_id,
        route,
        payload_size: payload.len() as u32,
		attachments: vec![]
    };

    let mut msg_meta = serde_json::to_vec(&msg_meta)?;    

    let mut buf = vec![];

    buf.put_u32_be(msg_meta.len() as u32);

    buf.append(&mut msg_meta);
    buf.append(&mut payload);
    
    Ok(buf)
}

pub fn reply_to_rpc_dto<T>(tx: String, rx: String, key: String, correlation_id: Uuid, payload: T, route: Route) -> Result<Vec<u8>, Error> where T: Debug, T: serde::Serialize, for<'de> T: serde::Deserialize<'de> {
    let mut payload = serde_json::to_vec(&payload)?;

    let msg_meta = MsgMeta {
        tx,
        rx,
        key,
        kind: MsgKind::RpcResponse,
        correlation_id,
        route,
        payload_size: payload.len() as u32,
		attachments: vec![]
    };        

    let mut msg_meta = serde_json::to_vec(&msg_meta)?;    

    let mut buf = vec![];

    buf.put_u32_be(msg_meta.len() as u32);

    buf.append(&mut msg_meta);
    buf.append(&mut payload);
  
    Ok(buf)
}

/*
    pub fn recv_event(&self) -> Result<(MsgMeta, R), Error> {
        let (msg_meta, len, data) = self.rx.recv()?;            

        let payload = serde_json::from_slice::<R>(&data[len + 4..])?;        

        Ok((msg_meta, payload))
    }
    pub fn recv_rpc_request(&self) -> Result<(MsgMeta, R), Error> {
        let (msg_meta, len, data) = self.rpc_request_rx.recv()?;            

        let payload = serde_json::from_slice::<R>(&data[len + 4..])?;        

        Ok((msg_meta, payload))
    }
    */

pub fn rpc_dto<T>(tx: String, rx: String, key: String, payload: T, route: Route) -> Result<Vec<u8>, Error> where T: Debug, T: serde::Serialize, for<'de> T: serde::Deserialize<'de> {
    let mut payload = serde_json::to_vec(&payload)?;
    let correlation_id = Uuid::new_v4();

    let msg_meta = MsgMeta {
        tx,
        rx,
        key,
        kind: MsgKind::RpcRequest,
        correlation_id,
        route,
        payload_size: payload.len() as u32,
		attachments: vec![]
    };
    
    let mut msg_meta = serde_json::to_vec(&msg_meta)?;

    let mut buf = vec![];

    buf.put_u32_be(msg_meta.len() as u32);

    buf.append(&mut msg_meta);
    buf.append(&mut payload);

    Ok(buf)
}

pub fn rpc_dto_with_correlation_id<T>(tx: String, rx: String, key: String, mut payload: T, route: Route) -> Result<(Uuid, Vec<u8>), Error> where T: Debug, T: serde::Serialize, for<'de> T: serde::Deserialize<'de> {
    let mut payload = serde_json::to_vec(&payload)?;
    let correlation_id = Uuid::new_v4();

    let msg_meta = MsgMeta {
        tx,
        rx,
        key,
        kind: MsgKind::RpcRequest,
        correlation_id,
        route,
        payload_size: payload.len() as u32,
		attachments: vec![]
    };

    let mut msg_meta = serde_json::to_vec(&msg_meta)?;        

    let mut buf = vec![];

    buf.put_u32_be(msg_meta.len() as u32);

    buf.append(&mut msg_meta);
    buf.append(&mut payload);

    Ok((correlation_id, buf))
}

pub fn rpc_dto_with_attachments<T>(tx: String, rx: String, key: String, payload: T, attachments: Vec<(String, Vec<u8>)>, route: Route) -> Result<Vec<u8>, Error> where T: Debug, T: serde::Serialize, for<'de> T: serde::Deserialize<'de> {
    let mut payload = serde_json::to_vec(&payload)?;
    let correlation_id = Uuid::new_v4();
    let mut attachments_meta = vec![];
    let mut attachments_payload = vec![];

    for (attachment_name, mut attachment_payload) in attachments {
        attachments_meta.push(Attachment {
            name: attachment_name,
            size: attachment_payload.len() as u32
        });        
        attachments_payload.append(&mut attachment_payload);
    }

    let msg_meta = MsgMeta {
        tx,
        rx,
        key,
        kind: MsgKind::RpcRequest,
        correlation_id,
        route,
        payload_size: payload.len() as u32,
		attachments: attachments_meta
    };

    let mut msg_meta = serde_json::to_vec(&msg_meta)?;    

    let mut buf = vec![];

    buf.put_u32_be(msg_meta.len() as u32);

    buf.append(&mut msg_meta);
    buf.append(&mut payload);
    buf.append(&mut attachments_payload);

    Ok(buf)
}

pub fn rpc_dto_with_later_attachments<T>(tx: String, rx: String, key: String, payload: T, attachments: Vec<(String, u32)>, route: Route) -> Result<Vec<u8>, Error> where T: Debug, T: serde::Serialize, for<'de> T: serde::Deserialize<'de> {
    let mut payload = serde_json::to_vec(&payload)?;
    let correlation_id = Uuid::new_v4();
    let mut attachments_meta = vec![];    

    for (attachment_name,attachment_size) in attachments {
        attachments_meta.push(Attachment {
            name: attachment_name,
            size: attachment_size
        });                
    }

    let msg_meta = MsgMeta {
        tx,
        rx,
        key,
        kind: MsgKind::RpcRequest,
        correlation_id,
        route,
        payload_size: payload.len() as u32,
		attachments: attachments_meta
    };

    let mut msg_meta = serde_json::to_vec(&msg_meta)?;    

    let mut buf = vec![];

    buf.put_u32_be(msg_meta.len() as u32);

    buf.append(&mut msg_meta);
    buf.append(&mut payload);    

    Ok(buf)
}

/*
impl MagicBall2 {
    pub fn new(addr: String, sender: Sender, rx: crossbeam::channel::Receiver<(MsgMeta, usize, Vec<u8>)>, rpc_request_rx: crossbeam::channel::Receiver<(MsgMeta, usize, Vec<u8>)>, rpc_tx: crossbeam::channel::Sender<ClientMsg>) -> MagicBall2 {
        MagicBall2 {
            addr,
            sender,
            rx,
            rpc_request_rx,
            rpc_tx
        }
    }
    */

pub fn event_dto2(tx: String, rx: String, key: String, mut payload: Vec<u8>, route: Route) -> Result<Vec<u8>, Error> {        
    let correlation_id = Uuid::new_v4();
    
    let msg_meta = MsgMeta {
        tx,
        rx,
        key,
        kind: MsgKind::Event,
        correlation_id,
        route,
        payload_size: payload.len() as u32,
		attachments: vec![]
    };

    let mut msg_meta = serde_json::to_vec(&msg_meta)?;        

    let mut buf = vec![];

    buf.put_u32_be(msg_meta.len() as u32);

    buf.append(&mut msg_meta);
    buf.append(&mut payload);
    
    Ok(buf)
}

pub fn reply_to_rpc_dto2(tx: String, rx: String, key: String, correlation_id: Uuid, mut payload: Vec<u8>, route: Route) -> Result<Vec<u8>, Error> {
    let msg_meta = MsgMeta {
        tx,
        rx,
        key,
        kind: MsgKind::RpcResponse,
        correlation_id,
        route,
        payload_size: payload.len() as u32,
		attachments: vec![]
    };

    let mut msg_meta = serde_json::to_vec(&msg_meta)?;        

    let mut buf = vec![];

    buf.put_u32_be(msg_meta.len() as u32);

    buf.append(&mut msg_meta);
    buf.append(&mut payload);
    
    Ok(buf)
}

    /*
    pub fn recv_event(&self) -> Result<(MsgMeta, Vec<u8>), Error> {
        let (msg_meta, len, data) = self.rx.recv()?;            
        let payload = &data[len + 4..];        

        Ok((msg_meta, payload.to_vec()))
    }
    pub fn recv_rpc_request(&self) -> Result<(MsgMeta, Vec<u8>), Error> {
        let (msg_meta, len, data) = self.rpc_request_rx.recv()?;                
        let payload = &data[len + 4..];        

        Ok((msg_meta, payload.to_vec()))
    }
    */

pub fn rpc_dto2(tx: String, rx: String, key: String, mut payload: Vec<u8>, route: Route) -> Result<Vec<u8>, Error> {
    let correlation_id = Uuid::new_v4();

    let msg_meta = MsgMeta {
        tx,
        rx,
        key,
        kind: MsgKind::RpcRequest,
        correlation_id,
        route,
        payload_size: payload.len() as u32,
		attachments: vec![]
    };

    let mut msg_meta = serde_json::to_vec(&msg_meta)?;    

    let mut buf = vec![];

    buf.put_u32_be(msg_meta.len() as u32);

    buf.append(&mut msg_meta);
    buf.append(&mut payload);

    Ok(buf)
}

pub fn rpc_dto_with_attachments2(tx: String, rx: String, key: String, mut payload: Vec<u8>, attachments: Vec<(String, Vec<u8>)>, route: Route) -> Result<Vec<u8>, Error> {
    let correlation_id = Uuid::new_v4();
    let mut attachments_meta = vec![];
    let mut attachments_payload = vec![];

    for (attachment_name, mut attachment_payload) in attachments {
        attachments_meta.push(Attachment {
            name: attachment_name,
            size: attachment_payload.len() as u32
        });        
        attachments_payload.append(&mut attachment_payload);
    }

    let msg_meta = MsgMeta {
        tx,
        rx,
        key,
        kind: MsgKind::RpcRequest,
        correlation_id,
        route,
        payload_size: payload.len() as u32,
		attachments: attachments_meta
    };

    let mut msg_meta = serde_json::to_vec(&msg_meta)?;    

    let mut buf = vec![];

    buf.put_u32_be(msg_meta.len() as u32);

    buf.append(&mut msg_meta);
    buf.append(&mut payload);
    buf.append(&mut attachments_payload);

    Ok(buf)
}

pub fn rpc_dto_with_later_attachments2(tx: String, rx: String, key: String, mut payload: Vec<u8>, attachments: Vec<(String, u32)>, route: Route) -> Result<Vec<u8>, Error> {
    let correlation_id = Uuid::new_v4();
    let mut attachments_meta = vec![];    

    for (attachment_name,attachment_size) in attachments {
        attachments_meta.push(Attachment {
            name: attachment_name,
            size: attachment_size
        });                
    }

    let msg_meta = MsgMeta {
        tx,
        rx,
        key,
        kind: MsgKind::RpcRequest,
        correlation_id,
        route,
        payload_size: payload.len() as u32,
		attachments: attachments_meta
    };

    let mut msg_meta = serde_json::to_vec(&msg_meta)?;    

    let mut buf = vec![];

    buf.put_u32_be(msg_meta.len() as u32);

    buf.append(&mut msg_meta);
    buf.append(&mut payload);    

    Ok(buf)
}

pub fn rpc_dto_with_correlation_id_2(tx: String, rx: String, key: String, mut payload: Vec<u8>, route: Route) -> Result<(Uuid, Vec<u8>), Error> {
    let correlation_id = Uuid::new_v4();

    let msg_meta = MsgMeta {
        tx,
        rx,
        key,
        kind: MsgKind::RpcRequest,
        correlation_id,
        route,
        payload_size: payload.len() as u32,
		attachments: vec![]
    };

    let mut msg_meta = serde_json::to_vec(&msg_meta)?;        

    let mut buf = vec![];

    buf.put_u32_be(msg_meta.len() as u32);

    buf.append(&mut msg_meta);
    buf.append(&mut payload);

    Ok((correlation_id, buf))
}

pub fn get_msg_meta(data: &[u8]) -> Result<MsgMeta, Error> {
    let mut buf = std::io::Cursor::new(data);
    let len = buf.get_u32_be() as usize;

    serde_json::from_slice::<MsgMeta>(&data[4..len + 4])
}

pub fn get_msg<T>(data: &[u8]) -> Result<(MsgMeta, T, Vec<(String, Vec<u8>)>), Error> where T: Debug, T: serde::Serialize, for<'de> T: serde::Deserialize<'de> {
    let mut buf = std::io::Cursor::new(data);    
    let len = buf.get_u32_be();
    let msg_meta_offset = (len + 4) as usize;

    let msg_meta = serde_json::from_slice::<MsgMeta>(&data[4..msg_meta_offset as usize])?;

    let payload_offset = msg_meta_offset + msg_meta.payload_size as usize;

    let payload = serde_json::from_slice::<T>(&data[msg_meta_offset..payload_offset])?;

    let mut attachments = vec![];
    let mut attachment_offset = payload_offset;

    for attachment in msg_meta.attachments.iter() {
        let attachment_start = attachment_offset;
        attachment_offset = attachment_offset + attachment.size as usize;
        attachments.push((attachment.name.clone(), (&data[attachment_start..attachment_offset]).to_owned()))
    }

    Ok((msg_meta, payload, attachments))
}
