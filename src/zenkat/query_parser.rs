use std::collections::HashMap;

type QueryFn = fn() -> ();

pub struct QueryParser {
    ops: HashMap<String, QueryFn>,
}

impl QueryParser {
    pub fn new() -> QueryParser {
        let qp = QueryParser {
            ops: HashMap::new(),
        };
        return qp;
    }

    pub fn bind(&mut self, op_type: &str, to_call: QueryFn) -> () {
        self.ops.insert(op_type.to_string(), to_call);
    }

    pub fn unbind(&mut self, op_type: &str) {
        self.ops.remove(op_type);
    }
}
