use std::collections::HashMap;

use super::model::Model;
use super::model::Operation;

#[derive(Clone)]
pub enum Op {
    GET,
    PUT,
    APPEND,
}

#[derive(Clone)]
pub struct KvInput {
    op: Op,
    key: String,
    value: String,
}

#[derive(Clone)]
pub struct KvOutput {
    value: String,
}

#[derive(Clone)]
pub struct KvModel {}

impl Model for KvModel {
    type State = String;
    type Input = KvInput;
    type Output = KvOutput;

    fn partition(
        &self,
        history: Vec<Operation<Self::Input, Self::Output>>,
    ) -> Vec<Vec<Operation<Self::Input, Self::Output>>> {
        let mut map = HashMap::new();
        for op in history {
            let v = map.entry(op.input.key.clone()).or_insert(vec![]);
            (*v).push(op);
        }
        let mut ret = vec![];
        for (_, ops) in map {
            ret.push(ops);
        }
        ret
    }

    fn init(&self) -> Self::State {
        // note: we are modeling a single key's value here;
        // we're partitioning by key, so this is okay
        "".to_string()
    }

    fn step(
        &self,
        state: &Self::State,
        input: &Self::Input,
        output: &Self::Output,
    ) -> (bool, Self::State) {
        match input.op {
            Op::GET => (&output.value == state, state.clone()),
            Op::PUT => (true, input.value.clone()),
            Op::APPEND => (true, state.clone() + &input.value),
        }
    }
}
