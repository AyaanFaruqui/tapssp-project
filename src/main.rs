use std::env;
use std::fs;
use std::path::{Path, PathBuf}; 
use std::io;

// Final Project Challenge #1: Concurrency and Performance
// Rayon is used to parallelize the CPU-intensive recursive calls.
use rayon::prelude::*; 

fn main() {
    // --- CLI Argument Handling (Standard Rust Pattern) ---
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <path>", args[0]);
        std::process::exit(1);
    }

    let path = Path::new(&args[1]);

    // --- Core Logic Execution ---
    match get_dir_size(path) {
        // Current output reflects LOGICAL size (counts hardlinks multiple times)
        Ok(size) => println!("Total size of '{}': {} bytes", path.display(), size),
        Err(e) => eprintln!("Error: {}", e),
    }
    
}

// --- CORE SYSTEM FUNCTION: Concurrent Disk Usage Calculation ---

// This function recursively calculates the size of a directory.
// The key improvement for the final project is the use of Rayon for parallelism.
fn get_dir_size(path: &Path) -> io::Result<u64> {
    if path.is_file() {
        // Base case: return file size
        return Ok(fs::metadata(path)?.len());
    }

    if !path.is_dir() {
        return Ok(0);
    }

    let mut current_dir_files_size = 0;
    let mut subdirectories = Vec::new();

    // 1. SEQUENTIAL I/O STEP: Read the contents of the current directory.
    
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;

        if metadata.is_dir() {
            // Collect path: These will be processed concurrently by Rayon.
            subdirectories.push(entry.path()); 
        } else {
            // Accumulate size of files directly in this directory.
            current_dir_files_size += metadata.len();
        }
    }

    // 2. PARALLEL CPU STEP: Recursively process subdirectories concurrently.
    // This leverages multiple CPU cores to maximize performance (addressing 'systems' performance).
    let total_sub_size: u64 = subdirectories
        .par_iter() // Converts the standard Vec iterator into a parallel iterator.
        .map(|dir_path| {
            // Each recursive call to a subdirectory is executed in a separate Rayon thread.
            // unwrap_or(0) provides graceful non-fatal error handling (e.g., Permission Denied).
            get_dir_size(dir_path).unwrap_or(0) 
        })
        .sum(); // Efficiently and safely aggregates results from all parallel threads.

    // Final total is the sum of local files and the parallel subdirectory totals.
    Ok(current_dir_files_size + total_sub_size)
}