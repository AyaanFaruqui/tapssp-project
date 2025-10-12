use std::env;
use std::fs;
use std::path::Path;
use std::io;

fn main() {
    // Get command-line arguments.
    let args: Vec<String> = env::args().collect();

    // Ensure a path is provided.
    if args.len() < 2 {
        eprintln!("Usage: {} <path>", args[0]);
        std::process::exit(1);
    }

    let path = Path::new(&args[1]);

    // Calculate and print the size.
    match get_dir_size(path) {
        Ok(size) => println!("Total size of '{}': {} bytes", path.display(), size),
        Err(e) => eprintln!("Error: {}", e),
    }
}

// Recursively calculates the size of a directory.
fn get_dir_size(path: &Path) -> io::Result<u64> {
    let mut total_size = 0;

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let metadata = entry.metadata()?;

            if metadata.is_dir() {
                total_size += get_dir_size(&entry.path())?;
            } else {
                total_size += metadata.len();
            }
        }
    } else if path.is_file() {
        let metadata = fs::metadata(path)?;
        return Ok(metadata.len());
    }

    Ok(total_size)
}