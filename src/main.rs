mod tokenizer;
use tokenizer::frequency_counter;
use postgres::{Client, NoTls, Error};
mod db;
use db::{words, users, vocab_user};
use std::env;

fn main() {
    let db_url = env::var("DATABASE_URL").unwrap();
    let mut client = Client::connect(&db_url, NoTls).unwrap();

    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: {} [--user <id>] [--file <filename>]", args[0]);
        std::process::exit(1);
    }

    let mut user_id: Option<i32> = None;
    let mut file_name: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--user" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --user requires an ID");
                    std::process::exit(1);
                }
                user_id = match args[i + 1].parse::<i32>() {
                    Ok(id) => Some(id),
                    Err(_) => {
                        eprintln!("Error: User ID must be a valid integer");
                        std::process::exit(1);
                    }
                };
                i += 2;
            }
            "--file" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --file requires a filename");
                    std::process::exit(1);
                }
                file_name = Some(args[i + 1].clone());
                i += 2;
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                std::process::exit(1);
            }
        }
    }

    if let Some(file) = file_name && let Some(id) = user_id {
        println!("File: {}", file);

        let freq = frequency_counter(&file).unwrap();
        let total_words: u32 = freq.iter().map(|(_, count)| count).sum();
        let total_unique_words: usize = freq.len();

        let words: Vec<String> = freq.into_iter().map(|(word, _)| word).collect();

        let known_user_words_ids: Vec<i32> = vocab_user::get_words_for_user(&mut client, id).unwrap(); 
        let known_words: Vec<String> = known_user_words_ids
            .into_iter()
            .filter_map(|id| words::get_text_word(&mut client, id).ok().flatten())
            .collect();
        let unknown_words: Vec<String> = words
            .into_iter()
            .filter(|word| !known_words.contains(word))
            .collect();

        let total_unknown_words: usize = unknown_words.len();
        let total_known_words: usize = total_unique_words - total_unknown_words;
        let known_percentage: f32 = if total_unique_words > 0 {
            (total_known_words as f32) / (total_unique_words as f32) * 100.0
        } else {
            0.0
        };

        let new_words = words::insert_multiple_words(&mut client, &unknown_words, "english").unwrap();

        let unknown_word_ids = words::get_id_words(&mut client, &unknown_words, "english").unwrap();

        println!("Total words: {}", total_words);
        println!("Unique words: {}", total_unique_words);
        println!("Known words: {}", total_known_words);
        println!("Unkown words: {}", total_unknown_words);
        println!("Known Percentage: {}%", known_percentage);

        let batch_size = 5;
        let total = unknown_word_ids.len();

        for batch_start in (0..total).step_by(batch_size) {
            let batch_end = usize::min(batch_start + batch_size, total);
            let batch = &unknown_word_ids[batch_start..batch_end];

            println!("\nUnknown words:");
            for (i, id_option) in batch.iter().enumerate() {
                if let Some(id) = id_option {
                    let texto = words::get_text_word(&mut client, *id)
                        .unwrap()
                        .unwrap_or_else(|| "Unknown".to_string());
                    println!("{}. {} (ID: {})", i+1, texto, id);
                } else {
                    println!("{}. (Not found in DB)", i+1);
                }
            }


            println!("\nDo you want to mark them as known? (y/n)");
            let mut resposta = String::new();
            std::io::stdin().read_line(&mut resposta).unwrap();

            if resposta.trim().to_lowercase() == "y" {
                let word_ids: Vec<i32> = batch.iter().filter_map(|id_option| *id_option).collect();
                if !word_ids.is_empty(){
                    vocab_user::insert_vocab_users_multiple(&mut client, user_id.unwrap(), &word_ids).unwrap();
                }
            }
        }

    }
}

