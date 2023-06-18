use crate::{dht::RoutingTable, types::Id};

pub(crate) mod dht;
pub(crate) mod types;

fn main() {
    let mut rng = rand::thread_rng();
    let mut table = RoutingTable::new(Id::random(&mut rng), 5);

    for _ in 0..20 {
        table.insert(Id::random(&mut rng), ());
    }
    println!("({}) {table:?}", table.len());

    let id = Id::random(&mut rng);
    let closest = table.closest(id).take(10).collect::<Vec<_>>();
    println!("Closest to {id}:\n{closest:?}");
}
