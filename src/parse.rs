//use std::fs::File;
//use std::io;
use crate::query::Log;
use std::collections::HashMap;

/// Maximum of suspicious actions
const BAN_LIMIT: u16 = 50;

#[derive(Debug)]
enum Behavior {
    Suspicious(i32),     // Containing nb of files manipulated.
    Misbehaving(String), // Contaning name of directory been moved.
    Normal,              // Normal user activity.
}

#[derive(Debug)]
pub struct UserInfo {
    ip: Vec<String>,
    //    kind: ActivityType,
    //  kind: Vec<ActivityType>,
    count: i32,
}

impl UserInfo {
    fn new(ip: String) -> UserInfo {
        UserInfo {
            ip: {
                let mut n = Vec::new();
                n.push(ip);
                n
            },
            count: 0,
        }
    }

    fn update(&mut self, newip: String) {
        self.count += 1;
        if !self.ip.contains(&newip) {
            self.ip.push(newip);
        }
    }
}

/// Accounting of action in order to determine user behavior(Normal, Suspicious, Misbehaving)
pub fn log(entry: Vec<Log>, users: &mut HashMap<String, UserInfo>) {
    for el in entry {
        let uname = el.get_username();
        if !users.contains_key(&uname) {
            users.insert(uname, UserInfo::new(el.get_ip()));
        } else {
            users.get_mut(&uname).unwrap().update(el.get_ip());
        }
    }
}
