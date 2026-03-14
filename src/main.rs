use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::Argon2;
use clap::{Parser, Subcommand};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
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
    Get { name: String },
    List,
    Delete { name: String },
}


#[derive(Serialize, Deserialize)]
struct Entry {
    login: String,
    password: String,
}


fn storage_dir() -> PathBuf {
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
    let exe_dir = exe.parent().unwrap_or_else(|| std::path::Path::new("."));
    let portable = exe_dir.join("hiro_data");
    if portable.exists() || exe_dir.join("hiro_data.portable").exists() {
        portable
    } else {
        dirs_next::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".hiro")
    }
}


fn entry_path(name: &str) -> PathBuf {
    storage_dir().join(format!("{}.enc", name))
}


fn derive_key(master: &str, salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    Argon2::default()
        .hash_password_into(master.as_bytes(), salt, &mut key)
        .unwrap();
    key
}

fn encrypt(data: &[u8], key: &[u8; 32]) -> Vec<u8> {
    let cipher = Aes256Gcm::new(key.into());
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let mut result = nonce_bytes.to_vec();
    result.extend(cipher.encrypt(nonce, data).unwrap());
    result
}

fn decrypt(data: &[u8], key: &[u8; 32]) -> Option<Vec<u8>> {
    if data.len() < 12 { return None; }
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher.decrypt(nonce, ciphertext).ok()
}


fn main() {
    let cli = Cli::parse();
    fs::create_dir_all(storage_dir()).unwrap();

    let master = rpassword::prompt_password("master password: ").unwrap();
    let salt = b"hiro-static-salt";
    let key = derive_key(&master, salt);

    match cli.command {
        Commands::Add => {
            print!("name (e.g. github.com): ");
            io::stdout().flush().unwrap();
            let mut name = String::new();
            io::stdin().read_line(&mut name).unwrap();
            let name = name.trim();

            print!("login: ");
            io::stdout().flush().unwrap();
            let mut login = String::new();
            io::stdin().read_line(&mut login).unwrap();
            let login = login.trim().to_string();

            let password = rpassword::prompt_password("password: ").unwrap();

            let entry = Entry { login, password };
            let json = serde_json::to_vec(&entry).unwrap();
            let encrypted = encrypt(&json, &key);
            fs::write(entry_path(name), encrypted).unwrap();
            println!("saved: {}", name);
        }
        Commands::Get { name } => {
            let path = entry_path(&name);
            if !path.exists() {
                println!("not found: {}", name);
                return;
            }
            let data = fs::read(path).unwrap();
            match decrypt(&data, &key) {
                Some(json) => {
                    let entry: Entry = serde_json::from_slice(&json).unwrap();
                    println!("login:    {}", entry.login);
                    println!("password: {}", entry.password);
                }
                None => println!("wrong master password"),
            }
        }
        Commands::List => {
            let dir = storage_dir();
            let mut found = false;
            for e in fs::read_dir(&dir).unwrap() {
                let e = e.unwrap();
                let name = e.file_name();
                let name = name.to_string_lossy();
                if name.ends_with(".enc") {
                    println!("{}", name.trim_end_matches(".enc"));
                    found = true;
                }
            }
            if !found {
                println!("empty");
            }
        }
        Commands::Delete { name } => {
            let path = entry_path(&name);
            if path.exists() {
                fs::remove_file(path).unwrap();
                println!("deleted: {}", name);
            } else {
                println!("not found: {}", name);
            }
        }
    }
}


