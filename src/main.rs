mod alert;
mod nas;
mod parse;
mod query;

extern crate sys_info;
extern crate daemonize;

use alert::{email, sms};
use parse::Behavior;
use query::Type;
use rusqlite::Connection;
use std::{collections::HashMap, fs::File, process, env, thread, time::Duration};
use daemonize::Daemonize;

macro_rules! nas_shutdown {
    () => {
        String::from(format!(
            "Alert NAS {} shutdown ! Because of too many suspicious activities !",
            sys_info::hostname().unwrap()
        ))
    };
}

const DB: &str = "/var/log/synolog/.SMBXFERDB";

/// Maximum of suspicious actions
const BAN_LIMIT: i32 = 50;

pub struct Cdtl {
    user: String,
    pwd: String,
    sys: String,
    folder: String,
}

/// Loop delay in milliseconds
const TIME: u64 = 2_000;

/// Get environment variable for lftp use
fn getenv(var: &str) -> String {
    match env::var(var) {
        Ok(val) => val,
        Err(e) => panic!("{} : {}", var, e),
    }
}

fn env_variables() -> Cdtl {
    let crdtl = getenv("CRDTL");
    Cdtl {
        user: crdtl[..10].to_string(),
        pwd: crdtl[10..18].to_string(),
        sys: getenv("TARGETSYS"),
        folder: getenv("FOLDER"),
    }
}

fn daemonize() {
    let stdout = File::create("/tmp/randetect.out").unwrap();
    let stderr = File::create("/tmp/randetect.err").unwrap();

    let daemonize = Daemonize::new()
        .pid_file("/run/randetect.pid")
        .chown_pid_file(true)
        .working_directory("/tmp")
        .user("nobody")
        .group("daemon")
        .group(2)
        .umask(0o027)
        .stdout(stdout)
        .stderr(stderr)
        .privileged_action(|| "Executed before drop privileges");

    match daemonize.start() {
        Ok(_) => println!("Success, daemonized"),
        Err(e) => { eprintln!("Error, {}", e); process::exit(1) },
    }
}

fn main() {
    daemonize();

    let var: Cdtl = env_variables();

    nas::enable_firewall();

    let duration = Duration::from_millis(TIME);

    let conn = match Connection::open(DB) {
        Err(conn) => panic!("Could not reach/open database {} {}", DB, conn),
        Ok(conn) => conn,
    };
    let mut id = query::updated_id(&conn);

    loop {
        let mut list: HashMap<String, parse::UserInfo> = HashMap::new();

        let mut query = query::select(&conn, Type::Move, &id);
        query.extend(query::select(&conn, Type::Delete, &id));
        query.extend(query::select(&conn, Type::SuspiciousCwd, &id));

        id = query::updated_id(&conn);

        parse::log(query, &mut list);

        let mut shutdown = 0;
        for user in list.iter() {
            let (name, info) = user;
            for beh in info.get_behaviors() {
                match beh {
                    Behavior::Delete(c) if *c >= BAN_LIMIT => {
                        nas::ban(info);
                        email::send(&name, info, "delete");
                        sms::send(&var, format!(
                                "Alert NAS {} user: {} banned because of deleting {} files from ip:{:?}"
                                , sys_info::hostname().unwrap(), name, *c, info.get_ips()));
                    }
                    Behavior::Suspicious(c) if *c >= BAN_LIMIT => {
                        nas::ban(info);
                        shutdown += 1;
                        email::send(&name, info, "Suspicious");
                        sms::send(&var, format!(
                                "Alert NAS {} user: {} banned because of suspicious activity {} times from ip:{:?}"
                                , sys_info::hostname().unwrap(), name, *c, info.get_ips()));
                    }
                    Behavior::Move(_s) => {
                        email::send(&name, info, "Move");
                    }
                    _ => (),
                }
            }
            if shutdown > 1 {
                sms::send(&var, nas_shutdown!());
                nas::poweroff();
            }
        }
        thread::sleep(duration);
    }
}
