use std::error::Error;
use std::fs::File;
use std::io::Write;
use rand::seq::SliceRandom;
use rand::thread_rng;

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

fn generate_mnemonic_phrase(words: &[String], count: usize) -> Vec<String> {
    let mut rng = thread_rng();
    words.choose_multiple(&mut rng, count)
        .cloned()
        .collect()
}

fn main() {
    match read_words() {
        Ok(words) => {
            let phrase = generate_mnemonic_phrase(&words, 12);
            println!("Your 12-word mnemonic phrase:");
            
            // Save to file
            let mut file = File::create("phrase.txt").expect("Failed to create file");
            for word in &phrase {
                println!("{}", word);
                writeln!(file, "{}", word).expect("Failed to write to file");
            }
            println!("\nPhrase saved to phrase.txt");
        }
        Err(e) => eprintln!("Error reading words: {}", e),
    }
}
