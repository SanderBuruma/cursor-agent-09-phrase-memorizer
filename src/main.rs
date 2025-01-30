use std::error::Error;
use std::fs::File;
use std::io::{self, Write, Read};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::process::Command;
use std::path::Path;
use std::fs;

fn clear_screen() {
    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/c", "cls"])
            .status()
            .unwrap();
    } else {
        Command::new("clear")
            .status()
            .unwrap();
    }
}

fn pause() -> io::Result<()> {
    println!("\nPress Enter to continue...");
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(())
}

fn read_words() -> Result<Vec<String>, Box<dyn Error>> {
    let mut words = Vec::new();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path("words.csv")?;
    
    for result in rdr.records() {
        let record = result?;
        if let Some(word) = record.get(0) {
            words.push(word.to_string());
        }
    }
    Ok(words)
}

fn get_next_backup_number() -> io::Result<String> {
    let mut highest = 0;
    for entry in fs::read_dir(".")? {
        let entry = entry?;
        let name = entry.file_name().into_string().unwrap_or_default();
        if name.starts_with("phrase.") && name.ends_with(".txt") {
            if let Some(num_str) = name.strip_prefix("phrase.").and_then(|s| s.strip_suffix(".txt")) {
                if let Ok(num) = num_str.parse::<u32>() {
                    highest = highest.max(num);
                }
            }
        }
    }
    Ok(format!("{:03}", highest + 1))
}

fn backup_existing_phrase() -> io::Result<()> {
    if Path::new("phrase.txt").exists() {
        let backup_num = get_next_backup_number()?;
        let backup_name = format!("phrase.{}.txt", backup_num);
        fs::rename("phrase.txt", backup_name)?;
    }
    Ok(())
}

fn generate_new_phrase(words: &[String]) -> io::Result<()> {
    clear_screen();
    
    // Backup existing phrase if it exists
    if let Err(e) = backup_existing_phrase() {
        eprintln!("Warning: Could not backup existing phrase: {}", e);
        pause()?;
    }
    
    let phrase = generate_mnemonic_phrase(words, 12);
    println!("Your 12-word mnemonic phrase:");
    
    let mut file = File::create("phrase.txt")?;
    for word in &phrase {
        println!("{}", word);
        writeln!(file, "{}", word)?;
    }
    println!("\nPhrase saved to phrase.txt");
    pause()?;
    Ok(())
}

fn practice_phrase() -> io::Result<()> {
    if !Path::new("phrase.txt").exists() {
        clear_screen();
        println!("No phrase.txt found. Generating a new phrase...");
        pause()?;
        match read_words() {
            Ok(words) => {
                generate_new_phrase(&words)?;
            }
            Err(e) => {
                eprintln!("Error reading words: {}", e);
                pause()?;
                return Ok(());
            }
        }
    }

    let mut content = String::new();
    File::open("phrase.txt")?.read_to_string(&mut content)?;
    let words: Vec<&str> = content.lines().collect();
    let mut successful_attempts = 0;

    'practice: loop {
        clear_screen();
        println!("\nType each word and press Enter. Press Ctrl+C to exit.");
        println!("If you make a mistake, you'll need to start over.");
        if successful_attempts > 0 {
            println!("Successful attempts: {}", successful_attempts);
        }
        
        for (i, word) in words.iter().enumerate() {
            print!("Word {} > ", i + 1);
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();
            
            if input != *word {
                println!("Incorrect. Starting over...");
                println!("You had {} successful attempts.", successful_attempts);
                successful_attempts = 0;  // Reset counter on mistake
                pause()?;
                continue 'practice;
            }
        }
        
        successful_attempts += 1;
        println!("\nCongratulations! You've correctly typed all words!");
        println!("Total successful attempts: {}", successful_attempts);
        pause()?;
        break Ok(());
    }
}

fn generate_mnemonic_phrase(words: &[String], count: usize) -> Vec<String> {
    let mut rng = thread_rng();
    words.choose_multiple(&mut rng, count)
        .cloned()
        .collect()
}

fn main() {
    // Check for phrase.txt at startup
    if !Path::new("phrase.txt").exists() {
        match read_words() {
            Ok(words) => {
                if let Err(e) = generate_new_phrase(&words) {
                    eprintln!("Error: {}", e);
                    pause().unwrap();
                }
            }
            Err(e) => {
                eprintln!("Error reading words: {}", e);
                pause().unwrap();
            }
        }
    }

    loop {
        clear_screen();
        println!("\nMnemonic Phrase Generator and Practice Tool");
        println!("----------------------------------------");
        println!("1. Generate new phrase");
        println!("2. Practice typing existing phrase");
        println!("3. Exit");
        print!("\nChoose an option > ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();

        match choice.trim() {
            "1" => {
                match read_words() {
                    Ok(words) => {
                        if let Err(e) = generate_new_phrase(&words) {
                            eprintln!("Error: {}", e);
                            pause().unwrap();
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading words: {}", e);
                        pause().unwrap();
                    }
                }
            }
            "2" => {
                if let Err(e) = practice_phrase() {
                    eprintln!("Error: {}", e);
                    pause().unwrap();
                }
            }
            "3" => break,
            _ => {
                println!("Invalid choice.");
                pause().unwrap();
            }
        }
    }
}
