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

impl KvModel {
    pub fn new() -> Self {
        KvModel{}
    }
}

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

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::{Result, BufReader, BufRead};
    use std::collections::HashMap;

    use regex::Regex;
    use model::{Value, Event, Model, EventKind};
    use super::{KvInput, KvModel, KvOutput, Op};
    use super::super::check_events;

    fn check_kv(log_name: String, correct: bool) {
        let model = KvModel::new();

        let file_name = format!("../linearizability/test_data/{}.txt", &log_name);
        let events = match parse_kv_log(&file_name) {
            Ok(events) => events,
            Err(e) => panic!("parse kv log {} failed: {}", &file_name, e),
        };
        assert_eq!(check_events(model, events), correct);
    }

    fn parse_kv_log(file_name: &String) -> Result<Vec<Event<Value<<KvModel as Model>::Input, <KvModel as Model>::Output>>>> {
        lazy_static! {
            static ref invoke_get : Regex = Regex::new(r#"\{:process (\d+), :type :invoke, :f :get, :key "(.*)", :value nil\}"#).unwrap();
            static ref invoke_put : Regex = Regex::new(r#"\{:process (\d+), :type :invoke, :f :put, :key "(.*)", :value "(.*)"\}"#).unwrap();
            static ref invoke_append : Regex = Regex::new(r#"\{:process (\d+), :type :invoke, :f :append, :key "(.*)", :value "(.*)"\}"#).unwrap();
            static ref return_get : Regex = Regex::new(r#"\{:process (\d+), :type :ok, :f :get, :key ".*", :value "(.*)"\}"#).unwrap();
            static ref return_put : Regex = Regex::new(r#"\{:process (\d+), :type :ok, :f :put, :key ".*", :value ".*"\}"#).unwrap();
            static ref return_append : Regex = Regex::new(r#"\{:process (\d+), :type :ok, :f :append, :key ".*", :value ".*"\}"#).unwrap();
        }

        let f = File::open(file_name)?;
        let buf_reader = BufReader::new(f);
        let mut events = vec![];
        let mut id = 0;
        let mut procid_map : HashMap<isize, usize> = HashMap::new();

        for line in buf_reader.lines() {
            let contents = line.unwrap();
            if let Some(args) = invoke_get.captures(&contents) {
                events.push(Event{
                    kind: EventKind::CallEvent,
                    value: Value::Input(KvInput{
                        op: Op::GET,
                        key: args[2].to_string(),
                        value: "".to_string(),
                    }),
                    id,
                });
                procid_map.insert(args[1].to_string().parse().unwrap(), id);
                id += 1;
            } else if let Some(args) = invoke_put.captures(&contents) {
                events.push(Event{
                    kind: EventKind::CallEvent,
                    value: Value::Input(KvInput{
                        op: Op::PUT,
                        key: args[2].to_string(),
                        value: args[3].to_string(),
                    }),
                    id,
                });
                procid_map.insert(args[1].to_string().parse().unwrap(),id);
                id += 1;
            } else if let Some(args) = invoke_append.captures(&contents) {
                events.push(Event{
                    kind: EventKind::CallEvent,
                    value: Value::Input(KvInput{
                        op: Op::APPEND,
                        key: args[2].to_string(),
                        value: args[3].to_string(),
                    }),
                    id,
                });
                procid_map.insert(args[1].to_string().parse().unwrap(),id);
                id += 1;
            } else if let Some(args) = return_get.captures(&contents) {
                let match_id = procid_map.remove(&args[1].to_string().parse().unwrap()).unwrap();
                events.push(Event{
                    kind: EventKind::ReturnEvent,
                    value: Value::Output(KvOutput{
                        value: args[2].to_string(),
                    }),
                    id: match_id,
                });
            } else if let Some(args) = return_put.captures(&contents) {
                let match_id = procid_map.remove(&args[1].to_string().parse().unwrap()).unwrap();
                events.push(Event{
                    kind: EventKind::ReturnEvent,
                    value: Value::Output(KvOutput{
                        value: "".to_string(),
                    }),
                    id: match_id,
                });
            } else if let Some(args) = return_append.captures(&contents) {
                let match_id = procid_map.remove(&args[1].to_string().parse().unwrap()).unwrap();
                events.push(Event{
                    kind: EventKind::ReturnEvent,
                    value: Value::Output(KvOutput{
                        value: "".to_string(),
                    }),
                    id: match_id,
                });
            } else {
                unreachable!();
            }
        }

        for (_, match_id) in procid_map {
            events.push(Event{
                kind: EventKind::ReturnEvent,
                value: Value::Output(KvOutput{
                    value: "".to_string(),
                }),
                id: match_id,
            })
        }
        Ok(events)
    }

    #[test]
    fn test_kv_1client_ok() {
        check_kv("c01-ok".to_string(), true)
    }

    #[test]
    fn test_kv_1client_bad() {
        check_kv("c01-bad".to_string(), false)
    }

    #[test]
    fn test_kv_10client_ok() {
        check_kv("c10-ok".to_string(), true)
    }

    #[test]
    fn test_kv_10client_bad() {
        check_kv("c10-bad".to_string(), false)
    }

    #[test]
    fn test_kv_50client_ok() {
        check_kv("c50-ok".to_string(), true)
    }

    #[test]
    fn test_kv_50client_bad() {
        check_kv("c50-bad".to_string(), false)
    }
}
