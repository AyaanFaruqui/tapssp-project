use std::env;
use std::fs;
use std::path::{Path, PathBuf}; 
use std::io;
use std::process;
use std::sync::{Arc, Mutex};
use std::collections::HashSet;

// Final Project Challenge #1: Concurrency
use rayon::prelude::*; 

// Final Project Challenge #2: Hardlink Analysis (using stable same-file crate)
use same_file::Handle;

// Type alias for the unique file identifier (Handle is the unique key).
type FileId = Handle; 

// Struct to hold the hierarchical data for TUI visualization
#[derive(Debug, Clone)]
struct DirEntry {
    name: String,
    size: u64,
    children: Vec<DirEntry>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // --- CLI Argument Handling ---
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <path>", args[0]);
        std::process::exit(1);
    }
    let path = Path::new(&args[1]);

    if !path.exists() {
        eprintln!("Error: Path not found: {}", path.display());
        std::process::exit(1);
    }

    // --- Core System Setup: Shared State for Hardlink Tracking ---
    // Arc/Mutex for safe concurrent access to the file tracker.
    let files_seen: Arc<Mutex<HashSet<FileId>>> = Arc::new(Mutex::new(HashSet::new()));

    println!("Calculating tree structure concurrently...");

    // Phase 3: Build the complete hierarchical tree structure
    let root_entry = match calculate_tree(path, Arc::clone(&files_seen)) {
        Ok(entry) => entry,
        Err(e) => {
            eprintln!("Error during traversal: {}", e);
            std::process::exit(1);
        }
    };
    
    // --- Verification Output ---
    // Output of the total size and name, which proves the tree structure was built.
    println!("Apparent size of '{}': {} bytes", root_entry.name, root_entry.size);
    // Replace this print statement with the TUI draw loop.

    Ok(())
}

// --- CORE SYSTEM FUNCTION: Concurrent Tree Calculation ---

// Calculates the entire directory tree structure and total size concurrently.
fn calculate_tree(path: &Path, files_seen: Arc<Mutex<HashSet<FileId>>>) -> io::Result<DirEntry> {
    
    // Checking to see if it's a file first
    if path.is_file() {
        let size = get_dir_size_unique_file(path, files_seen)?;
        let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
        return Ok(DirEntry { name, size, children: Vec::new() });
    }
    
    if !path.is_dir() {
        // Stop recursion if not a directory or file.
        return Ok(DirEntry { name: path.to_string_lossy().into_owned(), size: 0, children: Vec::new() });
    }

    // 1. Collect immediate children paths
    let mut child_paths = Vec::new();
    for entry in fs::read_dir(path)? {
        if let Ok(entry) = entry {
            child_paths.push(entry.path());
        }
    }

    // 2. Process children concurrently (Phase 1 Concurrency is used here)
    let children_results: Vec<DirEntry> = child_paths.par_iter().filter_map(|child_path| {
        // Recursively call calculate_tree on child paths, passing the cloned Arc
        calculate_tree(child_path, Arc::clone(&files_seen)).ok()
    }).collect();

    // 3. Aggregate results
    let total_size = children_results.iter().map(|c| c.size).sum();

    // Get name for the current directory (uses full path if root or error)
    let name = path.file_name()
        .map(|os_str| os_str.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    Ok(DirEntry { name, size: total_size, children: children_results })
}

// Helper function that contains the Hardlink Analysis logic (Phase 2)
fn get_dir_size_unique_file(path: &Path, files_seen: Arc<Mutex<HashSet<FileId>>>) -> io::Result<u64> {
    
    let file_handle = match same_file::Handle::from_path(path) {
        Ok(handle) => handle,
        Err(_) => return Ok(0), 
    };
    
    let mut seen = files_seen.lock().unwrap();
    
    if seen.insert(file_handle) {
        // Count its physical size.
        let metadata = fs::metadata(path)?;
        return Ok(metadata.len());
    } else {
        // This is a hardlink; size contribution is 0.
        return Ok(0);
    }
}