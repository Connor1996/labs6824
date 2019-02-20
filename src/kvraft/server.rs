use std::sync::Arc;

use futures::sync::mpsc::unbounded;

use super::service::*;
use crate::raft;

pub struct KvServer {
    pub rf: raft::Node,
    me: usize,
    // snapshot if log grows this big
    maxraftstate: u64,
    // Your definitions here.
}

impl KvServer {
    pub fn new(
        servers: Vec<raft::service::RaftClient>,
        me: usize,
        persister: Box<dyn raft::persister::Persister>,
        maxraftstate: u64,
    ) -> KvServer {
        // You may need initialization code here.

        let (tx, apply_ch) = unbounded();
        let rf = raft::Raft::new(servers, me, persister, tx);

        KvServer {
            me,
            maxraftstate,
            rf: raft::Node::new(rf),
        }
    }
}

// Choose concurrency paradigm.
//
// You can either drive the kv server by the rpc framework,
//
// ```rust
// struct Node { server: Arc<Mutex<KvServer>> }
// ```
//
// or spawn a new thread runs the kv server and communicate via
// a channel.
//
// ```rust
// struct Node { sender: Sender<Msg> }
// ```
#[derive(Clone)]
pub struct Node {
    state: Arc<raft::State>,
    // Your definitions here.
}

impl Node {
    pub fn new(kv: KvServer) -> Node {
        let state = kv.rf.state.clone();
        // Your code here.
        Node { state }
    }

    /// the tester calls Kill() when a KVServer instance won't
    /// be needed again. you are not required to do anything
    /// in Kill(), but it might be convenient to (for example)
    /// turn off debug output from this instance.
    pub fn kill(&self) {
        // Your code here, if desired.
    }

    /// The current term of this peer.
    pub fn term(&self) -> u64 {
        self.state.term()
    }

    /// Whether this peer believes it is the leader.
    pub fn is_leader(&self) -> bool {
        self.state.is_leader()
    }
}

impl KvService for Node {
    fn get(&self, arg: GetRequest) -> GetReply {
        // Your code here.
        unimplemented!()
    }

    fn put_append(&self, arg: PutAppendRequest) -> PutAppendReply {
        // Your code here.
        unimplemented!()
    }
}