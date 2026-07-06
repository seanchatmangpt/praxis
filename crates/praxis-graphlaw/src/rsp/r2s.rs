use std::collections::HashSet;
use std::hash::Hash;
use std::mem;

pub enum StreamOperator {
    RSTREAM,
    ISTREAM,
    DSTREAM,
}

impl Default for StreamOperator {
    fn default() -> Self {
        StreamOperator::RSTREAM
    }
}
pub struct Relation2StreamOperator<O> {
    stream_operator: StreamOperator,
    old_result: HashSet<O>,
    new_result: HashSet<O>,
    ts: usize,
}

impl<O> Relation2StreamOperator<O>
where
    O: Clone + Hash + Eq,
{
    pub fn new(stream_operator: StreamOperator, start_time: usize) -> Relation2StreamOperator<O> {
        match stream_operator {
            StreamOperator::RSTREAM => Relation2StreamOperator {
                stream_operator,
                old_result: HashSet::with_capacity(0),
                new_result: HashSet::with_capacity(0),
                ts: start_time,
            },
            _ => Relation2StreamOperator {
                stream_operator,
                old_result: HashSet::new(),
                new_result: HashSet::new(),
                ts: start_time,
            },
        }
    }
    pub fn eval(&mut self, new_response: Vec<O>, ts: usize) -> Vec<O> {
        match self.stream_operator {
            StreamOperator::RSTREAM => new_response,
            StreamOperator::ISTREAM => {
                let to_compare = new_response.clone();
                self.prepare_compare(new_response, ts);
                to_compare
                    .into_iter()
                    .filter(|b| !self.old_result.contains(b))
                    .collect()
            }
            StreamOperator::DSTREAM => {
                self.prepare_compare(new_response, ts);
                let to_compare = self.old_result.clone();
                to_compare
                    .into_iter()
                    .filter(|b| !self.new_result.contains(b))
                    .collect()
            }
        }
    }

    fn prepare_compare(&mut self, new_repsonse: Vec<O>, ts: usize) {
        if self.ts < ts {
            mem::swap(&mut self.new_result, &mut self.old_result);
            self.new_result.clear();
            self.ts = ts;
        }
        new_repsonse.into_iter().for_each(|v| {
            self.new_result.insert(v);
            ()
        });
    }
}
#[cfg(test)]
#[path = "r2s_test.rs"]
mod r2s_test;
