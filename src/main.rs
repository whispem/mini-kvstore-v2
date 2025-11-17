use std::io::{self, Write};

mod store;
use store::KVStore;

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
    let mut kv = KVStore::open("data")?;
    println!("mini-kvstore-v2 — segmented log, compaction, index, Rust");
    print_help();

    let stdin = io::stdin();
    loop {
        print!("> ");
        io::stdout().flush().ok();
        
        let mut line = String::new();
        if stdin.read_line(&mut line)? == 0 {
            break; // EOF
        }
        
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let mut parts = line.splitn(3, ' ');
        let cmd = parts.next().unwrap();
        
        match cmd {
            "set" => {
                let key = match parts.next() {
                    Some(k) => k,
                    None => {
                        println!("Usage: set <key> <value>");
                        continue;
                    }
                };
                let value = match parts.next() {
                    Some(v) => v,
                    None => {
                        println!("Usage: set <key> <value>");
                        continue;
                    }
                };
                
                match kv.set(key, value.as_bytes()) {
                    Ok(_) => println!("OK"),
                    Err(e) => println!("Error: {}", e),
                }
            }
            "get" => {
                let key = match parts.next() {
                    Some(k) => k,
                    None => {
                        println!("Usage: get <key>");
                        continue;
                    }
                };
                
                match kv.get(key) {
                    Ok(Some(v)) => println!("{}", String::from_utf8_lossy(&v)),
                    Ok(None) => println!("Key not found"),
                    Err(e) => println!("Error: {}", e),
                }
            }
            "delete" => {
                let key = match parts.next() {
                    Some(k) => k,
                    None => {
                        println!("Usage: delete <key>");
                        continue;
                    }
                };
                
                match kv.delete(key) {
                    Ok(_) => println!("Deleted"),
                    Err(e) => println!("Error: {}", e),
                }
            }
            "compact" => match kv.compact() {
                Ok(_) => println!("Compaction finished."),
                Err(e) => println!("Compaction error: {}", e),
            },
            "help" => print_help(),
            "quit" | "exit" => break,
            other => println!("Unknown command: {}. Type 'help' for available commands.", other),
        }
    }
    
    println!("Goodbye.");
    Ok(())
}
