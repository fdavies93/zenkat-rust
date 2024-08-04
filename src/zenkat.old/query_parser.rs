use std::future::Future;
use std::sync::Arc;
use std::{collections::HashMap, pin::Pin};

use crate::{
    app_state::AppState,
    common::{
        zk_request::{ZkRequest, ZkRequestType},
        zk_response::ZkResponse,
    },
};

type QueryFn = fn(&ZkRequest, &Arc<AppState>) -> ZkResponseType;
type QueryFnBox = Box<QueryFn>;

pub type ZkResponseType = Pin<Box<dyn Future<Output = Result<ZkResponse, &'static str>>>>;

pub struct QueryParser {
    ops: HashMap<ZkRequestType, QueryFnBox>,
}

impl QueryParser {
    pub fn new() -> QueryParser {
        let qp = QueryParser {
            ops: HashMap::new(),
        };
        return qp;
    }

    pub async fn trigger(&self, request: &ZkRequest, state: &Arc<AppState>) -> ZkResponseType {
        let fn_ptr = self.ops.get(&request.request_type).unwrap();
        fn_ptr(request, state)
    }

    pub fn bind(&mut self, op_type: ZkRequestType, to_call: QueryFn) -> () {
        self.ops.insert(op_type, Box::new(to_call));
    }

    pub fn unbind(&mut self, op_type: &ZkRequestType) {
        self.ops.remove(op_type);
    }
}
