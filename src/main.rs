use std::env;
use std::fs;
use std::process;

fn count_lines(content: &str) -> usize {
    content.lines().count()
}

fn count_words(content: &str) -> usize {
    content.split_whitespace().count()
}

fn count_chars(content: &str) -> usize {
    content.chars().count()
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: wc [-c | -l | -w] <filename>");
        process::exit(1);
    }

    let mut mode = "-w"; 
    let mut filename = &args[1];

    if args.len() == 3 {
        mode = &args[1];
        filename = &args[2];
    }

    // Читаем содержимое файла
    let content = match fs::read_to_string(filename) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Error reading file: {}", filename);
            process::exit(1);
        }
    };

    // Обрабатываем режим и выводим результат
    match mode {
        "-l" => println!("{}", count_lines(&content)),
        "-c" => println!("{}", count_chars(&content)),
        "-w" | _ => println!("{}", count_words(&content)),
    }
}
