use std::io::{self, Write};

mod store;
use store::KvStore;

fn print_help() {
    println!("Commands:");
    println!("  set <key> <value>        — set or update a key");
    println!("  get <key>                — get a value");
    println!("  delete <key>             — delete a key");
    println!("  compact                  — run manual compaction");
    println!("  help                     — show this help");
    println!("  quit / exit              — exit");
}

fn main() -> anyhow::Result<()> {
    let mut kv = KvStore::open("data")?;
    println!("mini-kvstore-v2 — segmented log, compaction, index, Rust");
    print_help();

    let stdin = io::stdin();
    loop {
        print!("> ");
        io::stdout().flush().ok();
        let mut line = String::new();
        if stdin.read_line(&mut line)? == 0 {
            break;
        }
        let line = line.trim();
        if line.is_empty() { continue; }
        let mut parts = line.splitn(3, ' ');
        let cmd = parts.next().unwrap();
        match cmd {
            "set" => {
                let key = match parts.next() { Some(k) => k, None => { println!("usage: set <key> <value>"); continue; }};
                let value = match parts.next() { Some(v) => v, None => { println!("usage: set <key> <value>"); continue; }};
                kv.set(key, value.as_bytes())?;
                println!("OK");
            }
            "get" => {
                let key = match parts.next() { Some(k) => k, None => { println!("usage: get <key>"); continue; }};
                match kv.get(key)? {
                    Some(v) => println!("{}", String::from_utf8_lossy(&v)),
                    None => println!("Key not found"),
                }
            }
            "delete" => {
                let key = match parts.next() { Some(k) => k, None => { println!("usage: delete <key>"); continue; }};
                kv.delete(key)?;
                println!("Deleted (if key existed)");
            }
            "compact" => {
                kv.compact()?;
                println!("Compaction finished.");
            }
            "help" => print_help(),
            "quit" | "exit" => break,
            other => println!("Unknown command: {}", other),
        }
    }
    println!("Goodbye.");
    Ok(())
}
