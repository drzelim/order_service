use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

fn get_num_cores() -> u32 {
    let num_cores = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1) as u32;
    num_cores
}

fn main() {
    let numbers_count: u32 = 10;
    let numbers: Arc<Vec<u32>> = Arc::new((1..=numbers_count).collect());

    let mut handles = Vec::new();
    let num_cores = get_num_cores();

    let num_threads = if num_cores > numbers_count { numbers_count} else { num_cores };

    let index = Arc::new(AtomicUsize::new(0));

    for _ in 0..num_threads {
        let numbers = Arc::clone(&numbers);
        let index = Arc::clone(&index);

        let handle = thread::spawn(move || {
            loop {
                let i = index.fetch_add(1, Ordering::SeqCst);

                if i >= numbers.len() {
                    break;
                }

                let num = numbers[i];
                println!("{}", num * num);
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
