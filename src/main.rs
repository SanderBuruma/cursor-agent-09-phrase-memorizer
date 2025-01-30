use std::error::Error;
use std::fs::File;
use std::io::{self, Write, Read};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::process::Command;
use std::path::Path;
use std::fs;
use colored::*;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{enable_raw_mode, disable_raw_mode};

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
            print!("Word {} > ", (i + 1).to_string().cyan());
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();
            
            if input != *word {
                println!("{}", "Incorrect. Starting over...".red());
                println!("You had {} successful attempts.", successful_attempts);
                successful_attempts = 0;  // Reset counter on mistake
                pause()?;
                continue 'practice;
            }
        }
        
        successful_attempts += 1;
        println!("\n{}", "Congratulations! You've correctly typed all words!".green());
        println!("Total successful attempts: {}", successful_attempts.to_string().yellow());
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

fn list_backup_phrases() -> io::Result<Vec<String>> {
    let mut backups = Vec::new();
    for entry in fs::read_dir(".")? {
        let entry = entry?;
        let name = entry.file_name().into_string().unwrap_or_default();
        if name.starts_with("phrase.") && name.ends_with(".txt") && name != "phrase.txt" {
            backups.push(name);
        }
    }
    backups.sort();
    Ok(backups)
}

fn swap_phrases() -> io::Result<()> {
    clear_screen();
    let backups = list_backup_phrases()?;
    
    if backups.is_empty() {
        println!("No backup phrases found to swap with.");
        pause()?;
        return Ok(());
    }

    println!("\n{}", "Available backup phrases:".bright_yellow());
    for (i, name) in backups.iter().enumerate() {
        println!("{}. {}", (i + 1).to_string().cyan(), name);
    }
    print!("\nSelect backup number to swap with phrase.txt (1-{}) > ", backups.len());
    io::stdout().flush()?;

    enable_raw_mode()?;
    
    // Wait for the '3' key to be released
    loop {
        if let Event::Key(key_event) = event::read()? {
            if key_event.kind == KeyEventKind::Release && key_event.code == KeyCode::Char('3') {
                break;
            }
        }
    }
    
    let selected_idx = loop {
        if let Event::Key(key_event) = event::read()? {
            if key_event.kind == KeyEventKind::Press {
                if let KeyCode::Char(c) = key_event.code {
                    if let Some(digit) = c.to_digit(10) {
                        let idx = digit as usize - 1;
                        if idx < backups.len() {
                            disable_raw_mode()?;
                            println!("{}", c);
                            break idx;
                        }
                    }
                }
            }
        }
    };

    let backup_name = &backups[selected_idx];
    let temp_name = "phrase.temp.txt";

    // Read both files' contents
    let mut current_content = String::new();
    let mut backup_content = String::new();
    
    if Path::new("phrase.txt").exists() {
        File::open("phrase.txt")?.read_to_string(&mut current_content)?;
    }
    File::open(backup_name)?.read_to_string(&mut backup_content)?;

    // Write the swapped contents
    if !current_content.is_empty() {
        fs::write(temp_name, &current_content)?;
        fs::write("phrase.txt", &backup_content)?;
        fs::rename(temp_name, backup_name)?;
    } else {
        fs::write("phrase.txt", &backup_content)?;
        fs::remove_file(backup_name)?;
    }

    println!("\n{}", "Phrases swapped successfully!".green());
    pause()?;
    Ok(())
}

fn main() -> io::Result<()> {
    // Check for phrase.txt at startup
    if !Path::new("phrase.txt").exists() {
        match read_words() {
            Ok(words) => {
                if let Err(e) = generate_new_phrase(&words) {
                    eprintln!("Error: {}", e);
                    pause()?;
                }
            }
            Err(e) => {
                eprintln!("Error reading words: {}", e);
                pause()?;
            }
        }
    }

    loop {
        clear_screen();
        println!("\n{}", "Mnemonic Phrase Generator and Practice Tool".bright_yellow().bold());
        println!("{}", "----------------------------------------".bright_blue());
        println!("{}. Generate new phrase", "1".cyan());
        println!("{}. Practice typing existing phrase", "2".cyan());
        println!("{}. Swap with backup phrase", "3".cyan());
        println!("{}. Exit", "4".cyan());
        print!("\nPress a key to select option > ");
        io::stdout().flush()?;

        enable_raw_mode()?;
        
        let key = loop {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char(c) if c.is_digit(10) && c >= '1' && c <= '4' => {
                        disable_raw_mode()?;
                        println!("{}", c);
                        break key_event.code;
                    }
                    _ => continue,
                }
            }
        };
        
        match key {
            KeyCode::Char('1') => {
                match read_words() {
                    Ok(words) => {
                        if let Err(e) = generate_new_phrase(&words) {
                            eprintln!("{}", e.to_string().red());
                            pause()?;
                        }
                    }
                    Err(e) => {
                        eprintln!("{}", e.to_string().red());
                        pause()?;
                    }
                }
            }
            KeyCode::Char('2') => {
                if let Err(e) = practice_phrase() {
                    eprintln!("{}", e.to_string().red());
                    pause()?;
                }
            }
            KeyCode::Char('3') => {
                if let Err(e) = swap_phrases() {
                    eprintln!("{}", e.to_string().red());
                    pause()?;
                }
            }
            KeyCode::Char('4') => break,
            _ => unreachable!(),
        }
    }
    Ok(())
}


