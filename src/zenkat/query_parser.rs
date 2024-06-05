use std::collections::HashMap;
use std::sync::Arc;

use crate::{
    app_state::AppState,
    common::{
        zk_request::{ZkRequest, ZkRequestType},
        zk_response::ZkResponse,
    },
};

type QueryFn = fn(&ZkRequest, &Arc<AppState>) -> Result<ZkResponse, &'static str>;

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

    pub fn trigger(
        &self,
        request: &ZkRequest,
        state: &Arc<AppState>,
    ) -> Result<ZkResponse, &'static str> {
        let fn_ptr = self.ops.get(&request.request_type).unwrap();
        fn_ptr(request, state)
    }

    pub fn bind(&mut self, op_type: ZkRequestType, to_call: QueryFn) -> () {
        self.ops.insert(op_type, to_call);
    }

    pub fn unbind(&mut self, op_type: &ZkRequestType) {
        self.ops.remove(op_type);
    }
}
