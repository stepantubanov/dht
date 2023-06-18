use std::{collections::BinaryHeap, mem::replace};

use crate::types::{Id, ReverseDistance};

const BITS: usize = Id::SIZE * 8;

#[derive(Debug, Clone)]
pub(crate) struct RoutingTable<C> {
    self_id: Id,
    buckets: Box<[Bucket<C>; BITS]>,
    nodes_per_bucket: usize,
}

#[derive(Debug, Clone)]
struct Bucket<C> {
    entries: Vec<Entry<C>>,
}

#[derive(Debug, Clone)]
struct Entry<C> {
    id: Id,
    contact: C,
}

impl<C> RoutingTable<C> {
    pub(crate) fn new(self_id: Id, nodes_per_bucket: usize) -> Self {
        Self {
            self_id,
            buckets: Box::new([(); Id::SIZE * 8].map(|_| Bucket {
                entries: Vec::new(),
            })),
            nodes_per_bucket,
        }
    }

    pub(crate) fn insert(&mut self, id: Id, contact: C) -> Option<C> {
        let distance = self.self_id ^ id;
        let matched_bits = distance.leading_zeros() as usize;
        if matched_bits == BITS {
            assert_eq!(self.self_id, id);
            // TODO: error?
            panic!("inserted id matches self_id");
        }

        let bucket = &mut self.buckets[BITS - matched_bits - 1];
        if let Some(existing) = bucket.entries.iter_mut().find(|e| e.id == id) {
            return Some(replace(&mut existing.contact, contact));
        }

        if bucket.entries.len() < self.nodes_per_bucket {
            bucket.entries.push(Entry { id, contact });
            return None;
        }

        // Can't add it. TODO: Check if we can evict anything.
        None
    }

    pub(crate) fn closest(&self, id: Id) -> impl Iterator<Item = (Id, &'_ C)> + '_ {
        // Could probaby try to apply a smarter algorithm by utilizing common bit prefix of
        // distance between search ID and self ID.

        let mut heap = self
            .buckets
            .iter()
            .flat_map(|b| b.to_heap_items(id))
            .collect::<BinaryHeap<_>>();
        std::iter::from_fn(move || {
            heap.pop().map(|item| {
                let restored_id = item.distance ^ id;
                (restored_id, item.contact)
            })
        })
    }

    #[must_use]
    pub(crate) fn len(&self) -> usize {
        self.buckets.iter().map(|b| b.entries.len()).sum()
    }
}

impl<C> Bucket<C> {
    fn to_heap_items(&self, id: Id) -> impl Iterator<Item = ReverseDistance<&C>> {
        self.entries.iter().map(move |e| ReverseDistance {
            distance: e.id ^ id,
            contact: &e.contact,
        })
    }
}

#[cfg(test)]
mod tests {
    use rand::{rngs::StdRng, Rng, SeedableRng};

    use super::*;

    #[test]
    fn returns_closest_items() {
        let nodes = [
            "8bb7c3513f3c547eb8782e775d3895972b22aaa3",
            "3a210072672c070de4029b361f8984b76d3d2fc5",
            "00000856ade0fad1aec472413691ed58698f68cb",
            "41eca3506242e5f033a25e3dd1382007fe350596",
            "3a21185e998b3fc0ebbaf29363c6186022441d45",
        ]
        .map(|hex| hex.parse::<Id>().unwrap());

        let mut table = RoutingTable::new(
            "949e1514bc61a4cda96b40879e5f0513865a2644".parse().unwrap(),
            5,
        );
        for node in &nodes {
            table.insert(*node, ());
        }

        let (closest, _): (Vec<_>, Vec<()>) = table
            .closest("3a21264604acddf0678e917c1d3440d059c4dfc4".parse().unwrap())
            .unzip();

        assert_eq!(
            closest,
            [
                "3a210072672c070de4029b361f8984b76d3d2fc5",
                "3a21185e998b3fc0ebbaf29363c6186022441d45",
                "00000856ade0fad1aec472413691ed58698f68cb",
                "41eca3506242e5f033a25e3dd1382007fe350596",
                "8bb7c3513f3c547eb8782e775d3895972b22aaa3",
            ]
            .map(|hex| hex.parse::<Id>().unwrap())
        );
    }

    // TODO: proptest
    #[test]
    fn returns_closest_randomized() {
        // To be able to debug in case of failure.
        let mut seed = [0; 32];
        rand::thread_rng().fill(&mut seed);
        println!("seed: {seed:?}");

        let mut rng = StdRng::from_seed(seed);

        let mut table = RoutingTable::new(Id::random(&mut rng), rng.gen_range(1..10));
        for _ in 0..rng.gen_range(100..1000) {
            table.insert(Id::random(&mut rng), ());
        }

        let search = Id::random(&mut rng);
        let ordered: Vec<_> = table.closest(search).collect();

        for pair in ordered.windows(2) {
            let [(a, _), (b, _)] = pair else {
                unreachable!("hardcoded window size");
            };
            let distance_a = *a ^ search;
            let distance_b = *b ^ search;
            assert!(distance_a <= distance_b);
        }
    }
}
