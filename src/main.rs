use std::collections::HashMap;
use std::{thread, time};

mod alert;
mod parse;
mod query;

/// Loop delay in milliseconds
const TIME: u64 = 2_000;

fn main() {
    let duration = time::Duration::from_millis(TIME);

    let mut list: HashMap<String, parse::UserInfo> = HashMap::new();

    //    loop {
    let mut query = query::select(query::Type::Move);
    query.extend(query::select(query::Type::Delete));
    query.extend(query::select(query::Type::SuspiciousCwd));

    parse::log(query, &mut list);
    for user in list.iter() {
        println!("List= {:?}", user);
    }

    // thread::sleep(duration);
    //    }
}
