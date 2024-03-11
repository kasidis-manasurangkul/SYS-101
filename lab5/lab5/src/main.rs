use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

const MAX_COUNT: u32 = 10000000;  // Reduced for demonstration

// Function to measure the runtime of a counter
fn measure_runtime<F>(f: F) -> u128
where
    F: FnOnce() -> (),
{
    let start = Instant::now();
    f();
    start.elapsed().as_millis()
}

fn main() {
    // Print headers for the CSV file
    println!("Type,Parameter,Time");

    // // Iterate over different numbers of threads for the traditional and scalable counters
    // for num_threads in 1..=8 {  // Example range for threads
    //     // Measure traditional counter
    //     let time_traditional = measure_runtime(|| traditional_counter(num_threads));
    //     println!("\"Traditional\",{},{}", num_threads, time_traditional);

    //     // Measure scalable counter with a fixed approximation factor
    //     let time_scalable = measure_runtime(|| scalable_counter(num_threads, 1000));
    //     println!("\"Scalable\",{},{}", num_threads, time_scalable);
    // }

    // Iterate over different approximation factors for the scalable counter
    let num_threads = 4; // Fixed number of threads for this part
    for approx_factor in [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024].iter() {
        let time_scalable = measure_runtime(|| scalable_counter(num_threads, *approx_factor));
        // The following line seems redundant since it duplicates the above line.
        // You might want to remove one of them or adjust according to your needs.
        println!("\"ApproximationFactor\",{},{}", approx_factor, time_scalable);
    }
}

// Traditional Counter
fn traditional_counter(num_threads: usize) {
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..num_threads {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            while *counter.lock().unwrap() < MAX_COUNT {
                *counter.lock().unwrap() += 1;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

// Scalable Counter
fn scalable_counter(num_threads: usize, approx_factor: u32) {
    let global_counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    for _ in 0..num_threads {
        let global_counter = Arc::clone(&global_counter);
        let handle = thread::spawn(move || {
            let mut local_count = 0;
            while *global_counter.lock().unwrap() + local_count < MAX_COUNT {
                local_count += 1;
                if local_count >= approx_factor {
                    *global_counter.lock().unwrap() += local_count;
                    local_count = 0;
                }
            }
            *global_counter.lock().unwrap() += local_count;
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
