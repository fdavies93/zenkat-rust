use std::collections::HashMap;

use crate::common::zk_request::{ZkRequest, ZkRequestType};

type QueryFn = fn() -> ();

pub struct QueryParser {
    ops: HashMap<ZkRequestType, QueryFn>,
}

impl QueryParser {
    pub fn new() -> QueryParser {
        let qp = QueryParser {
            ops: HashMap::new(),
        };
        return qp;
    }

    pub fn trigger(&self, request: &ZkRequest) {
        let fn_ptr = self.ops.get(&request.request_type).unwrap();
        fn_ptr();
    }

    pub fn bind(&mut self, op_type: ZkRequestType, to_call: QueryFn) -> () {
        self.ops.insert(op_type, to_call);
    }

    pub fn unbind(&mut self, op_type: &ZkRequestType) {
        self.ops.remove(op_type);
    }
}
