use std::collections::HashSet;
use std::hash::Hash;

use crate::RNG;
use priority_queue::PriorityQueue;
use rand::Rng;

#[derive(Debug, Clone)]
pub struct SetSampler<T>
where
    T: Hash + Eq + Copy + Clone,
{
    rng: RNG,
    contents: HashSet<T>,
    pub queue: PriorityQueue<T, u32>,
}

impl<T> SetSampler<T>
where
    T: Hash + Eq + Copy,
{
    pub fn new() -> Self {
        Self {
            rng: RNG::default(),
            contents: HashSet::new(),
            queue: PriorityQueue::new(),
        }
    }
    // pub fn iter(&self) -> std::collections::hash_set::Iter<T> {
    //     self.queue.iter()
    // }
    pub fn insert(&mut self, item: T) {
        // let range = SampleRange::new(0, 100);
        let priority = self.rng.gen();
        self.contents.insert(item);
        self.queue.push(item, priority);
    }

    pub fn remove(&mut self, item: T) {
        self.contents.remove(&item);
        self.queue.remove(&item);
    }

    pub fn get_random(&self) -> Option<T> {
        let item = self.queue.peek();
        match item {
            Some((item, _priority)) => Some(*item),
            None => None,
        }
    }

    pub fn pop_random(&mut self) -> Option<T> {
        let item = self.queue.pop();
        match item {
            Some((item, _priority)) => {
                self.contents.remove(&item);
                Some(item)
            }
            None => None,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.contents.is_empty()
    }
}
