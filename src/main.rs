use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;

fn get_num_cores() -> usize {
    thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1) as usize
}

fn main() {
    let numbers_count: usize = 10;
    let numbers: Arc<Vec<i32>> = Arc::new((1..=numbers_count as i32).collect());

    let mut handles = Vec::new();
    let num_cores = get_num_cores();

    let num_threads = num_cores.min(numbers_count);

    let index = Arc::new(AtomicUsize::new(0));
    let (tx , rx) = mpsc::channel();

    for _ in 0..num_threads {
        let numbers = Arc::clone(&numbers);
        let index = Arc::clone(&index);
        let tx1 = tx.clone();

        let handle = thread::spawn(move || {
            loop {
                let i = index.fetch_add(1, Ordering::SeqCst);

                if i >= numbers.len() {
                    break;
                }
                
                let num = numbers[i];
                let square = num * num;
                tx1.send(square).unwrap();
            }
        });

        handles.push(handle);
    }

    drop(tx);

    let mut result = 0;

    for received in rx {
        result += received;
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Sum: {result}");

}
