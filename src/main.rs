use std::env;
use std::fs;
use std::path::{Path}; 
use std::io;

// Final Project Challenge #1: Concurrency and Performance
use rayon::prelude::*; 

// Final Project Challenge #2: Hardlink Analysis Imports
use std::sync::{Arc, Mutex};
use std::collections::HashSet;

// Using the stable 'same-file' crate for unique file identification.
use same_file::Handle;

// Type alias for a unique file identifier (Handle is the unique key).
type FileId = Handle; 

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <path>", args[0]);
        std::process::exit(1);
    }

    let path = Path::new(&args[1]);

    // Shared State Setup
    let files_seen: Arc<Mutex<HashSet<FileId>>> = Arc::new(Mutex::new(HashSet::new()));

    match get_dir_size_unique(path, files_seen) {
        Ok(size) => println!("Apparent size of '{}': {} bytes", path.display(), size),
        Err(e) => eprintln!("Error: {}", e),
    }
}

// CORE SYSTEM FUNCTION: Concurrent and Hardlink-Aware Calculation
fn get_dir_size_unique(path: &Path, files_seen: Arc<Mutex<HashSet<FileId>>>) -> io::Result<u64> {
    
    // Phase 1: Hardlink Logic
    if path.is_file() {
        
        // 1. Get the unique identifier (Handle)
        let file_handle = match Handle::from_path(path) {
            Ok(handle) => handle,
            Err(_) => return Ok(0), 
        };
        
        // 2. Safely acquire the Mutex lock
        let mut seen = files_seen.lock().unwrap();
        
        // 3. Check for uniqueness and insert
        if seen.insert(file_handle) {
            // Count its physical size.
            let metadata = fs::metadata(path)?;
            return Ok(metadata.len());
        } else {
            // This is a hardlink (already counted); size contribution is 0.
            return Ok(0);
        }
    }

    if !path.is_dir() {
        return Ok(0);
    }
    
    // Phase 2: Concurrency Logic
    let mut subdirectories = Vec::new();
    
    for entry in fs::read_dir(path)? {
        if let Ok(entry) = entry {
            subdirectories.push(entry.path());
        }
    }

    let total_sub_size: u64 = subdirectories
        .par_iter() 
        .map(|dir_path| {
            get_dir_size_unique(dir_path, Arc::clone(&files_seen)).unwrap_or(0)
        })
        .sum();

    Ok(total_sub_size)
}