use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "hiro")]
#[command(about = "hiro - менеджер паролей")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add,
    Get { url: String },
    List,
    Delete { url: String },
    Generate,
}

#[derive(Serialize, Deserialize, Clone)]
struct Entry {
    url: String,
    login: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct Database {
    entries: HashMap<String, Entry>,
}



fn db_path() -> PathBuf {
    let mut path = dirs_next::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(".hiro.json");
    path
}

fn load_db() -> Database {
    let path = db_path();
    if path.exists() {
        let data = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or(Database {
            entries: HashMap::new(),
        })
    } else {
        Database {
            entries: HashMap::new(),
        }
    }
}

fn save_db(db: &Database) {
    let path = db_path();
    let data = serde_json::to_string_pretty(db).unwrap();
    fs::write(path, data).unwrap();
}





fn main() {
    let cli = Cli::parse();
    let mut db = load_db();

    match cli.command {
        Commands::Add => {
            let url = rpassword::prompt_password("URL сайта: ").unwrap();
            let login = rpassword::prompt_password("Логин: ").unwrap();
            let password = rpassword::prompt_password("Пароль: ").unwrap();
            db.entries.insert(url.clone(), Entry { url: url.clone(), login, password });
            save_db(&db);
            println!("сохранено: {}", url);
        }
        Commands::Get { url } => {
            match db.entries.get(&url) {
                Some(e) => {
                    println!("url:    {}", e.url);
                    println!("логин:  {}", e.login);
                    println!("пароль: {}", e.password);
                }
                None => println!("не найдено: {}", url),
            }
        }
        Commands::List => {
            if db.entries.is_empty() {
                println!("пусто");
            } else {
                for (url, e) in &db.entries {
                    println!("{} ({})", url, e.login);
                }
            }
        }
        Commands::Delete { url } => {
            if db.entries.remove(&url).is_some() {
                save_db(&db);
                println!("удалено: {}", url);
            } else {
                println!("не найдено: {}", url);
            }
        }
        Commands::Generate => {
            use rand::Rng;
            let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*".chars().collect();
            let mut rng = rand::thread_rng();
            let pwd: String = (0..20).map(|_| chars[rng.gen_range(0..chars.len())]).collect();
            println!("пароль: {}", pwd);
        }
    }
}
