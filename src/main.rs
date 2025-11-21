use mini_kvstore_v2::KVStore;
use std::io::{self, Write};

fn main() {
    let mut kv = KVStore::open("db").expect("failed to open db");

    println!("mini-kvstore-v2 (type help for instructions)");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            continue;
        }

        let mut parts = input.trim().splitn(3, ' ');
        let cmd = match parts.next() {
            Some(c) => c,
            None => continue,
        };

        match cmd {
            "set" => {
                let key = parts.next().unwrap_or("");
                let val = parts.next().unwrap_or("");

                match kv.set(key, val.as_bytes()) {
                    Ok(()) => println!("OK"),
                    Err(e) => println!("Error: {}", e),
                }
            },

            "get" => {
                let key = parts.next().unwrap_or("");

                match kv.get(key) {
                    Ok(Some(v)) => println!("{}", String::from_utf8_lossy(&v)),
                    Ok(None) => println!("Key not found"),
                    Err(e) => println!("Error: {}", e),
                }
            },

            "delete" => {
                let key = parts.next().unwrap_or("");
                match kv.delete(key) {
                    Ok(()) => println!("Deleted"),
                    Err(e) => println!("Error: {}", e),
                }
            },

            "list" => {
                for key in kv.list_keys() {
                    println!("  {}", key);
                }
            },

            "compact" => match kv.compact() {
                Ok(()) => println!("Compaction finished"),
                Err(e) => println!("Compaction error: {}", e),
            },

            "stats" => println!("{:?}", kv.stats()),
            "help" => print_help(),
            "quit" | "exit" => break,
            other => println!("Unknown command: {}", other),
        }
    }
}

fn print_help() {
    println!("Available commands:");
    println!("  set <key> <value>");
    println!("  get <key>");
    println!("  delete <key>");
    println!("  list");
    println!("  compact");
    println!("  stats");
    println!("  help");
    println!("  quit / exit");
}
