use std::env;
use std::fs;
use std::path::Path; 
use std::io;
use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use std::error::Error;

// TUI Imports (Phase 3: Visualization)
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::layout::Margin; 
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

// Phase 1: Concurrency (Rayon)
use rayon::prelude::*; 

// Phase 2: Hardlink Analysis (Same-File API)
use same_file::Handle;

// Type alias for the unique file identifier (Inode/Device Handle).
type FileId = Handle; 

// Data structure for TUI visualization
#[derive(Debug, Clone)]
struct DirEntry {
    name: String,
    size: u64,
    children: Vec<DirEntry>,
}

fn main() -> Result<(), Box<dyn Error>> {
    // --- Argument Handling ---
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

    // Phase 2: Shared State Setup (Arc/Mutex for safe concurrent access to file tracker)
    let files_seen: Arc<Mutex<HashSet<FileId>>> = Arc::new(Mutex::new(HashSet::new()));

    // Phase 1 & 3: Build the tree structure concurrently
    let root_entry = match calculate_tree(path, Arc::clone(&files_seen)) {
        Ok(entry) => entry,
        Err(e) => {
            eprintln!("Error during traversal: {}", e);
            std::process::exit(1);
        }
    };
    
    // Phase 3: Run the visual TUI interface
    run_tui(&root_entry)?;

    Ok(())
}

// --- CORE SYSTEM FUNCTION: Concurrent Tree Calculation ---

// Recursively calculates the data structure, leveraging Rayon for parallelism.
fn calculate_tree(path: &Path, files_seen: Arc<Mutex<HashSet<FileId>>>) -> io::Result<DirEntry> {
    
    // Base case: Handle single files using Phase 2 logic (Hardlink Analysis)
    if path.is_file() {
        let size = get_dir_size_unique_file(path, files_seen)?;
        let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
        return Ok(DirEntry { name, size, children: Vec::new() });
    }
    
    if !path.is_dir() {
        return Ok(DirEntry { name: path.to_string_lossy().into_owned(), size: 0, children: Vec::new() });
    }

    // 1. Sequential I/O: Collect immediate children paths
    let mut child_paths = Vec::new();
    for entry in fs::read_dir(path)? {
        if let Ok(entry) = entry {
            child_paths.push(entry.path());
        }
    }

    // 2. Parallel Processing (Rayon): Recursively calculate children concurrently
    let children_results: Vec<DirEntry> = child_paths.par_iter().filter_map(|child_path| {
        calculate_tree(child_path, Arc::clone(&files_seen)).ok()
    }).collect();

    // 3. Aggregate size
    let total_size = children_results.iter().map(|c| c.size).sum();

    let name = path.file_name()
        .map(|os_str| os_str.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());

    Ok(DirEntry { name, size: total_size, children: children_results })
}

// Phase 2: Hardlink Analysis (Apparent Size Logic)
fn get_dir_size_unique_file(path: &Path, files_seen: Arc<Mutex<HashSet<FileId>>>) -> io::Result<u64> {
    
    // Get unique system file handle (Inode/Device)
    let file_handle = match same_file::Handle::from_path(path) {
        Ok(handle) => handle,
        Err(_) => return Ok(0), 
    };
    
    // Safely lock the shared set
    let mut seen = files_seen.lock().unwrap();
    
    // Count size only if the handle is new (deduplication)
    if seen.insert(file_handle) {
        let metadata = fs::metadata(path)?;
        return Ok(metadata.len());
    } else {
        return Ok(0); // Hardlink: Size is 0
    }
}

// --- TUI RENDERING LOGIC (Phase 3) ---

fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    format!("{:.1} {}", size, UNITS[unit_index])
}

fn build_list_items(entry: &DirEntry, items: &mut Vec<ListItem>, level: usize) {
    let size_unit = format_size(entry.size);
    let prefix = "  ".repeat(level);
    
    let color = if level == 0 {
        Color::Yellow
    } else if entry.size > 50_000_000 { 
        Color::Red
    } else if entry.size > 10_000_000 { 
        Color::LightYellow
    }
    else {
        Color::Green
    };

    let text = format!("{}{} | {}", prefix, entry.name, size_unit);
    items.push(ListItem::new(text).style(Style::default().fg(color)));

    for child in &entry.children {
        build_list_items(child, items, level + 1);
    }
}

fn run_tui(root_entry: &DirEntry) -> Result<(), Box<dyn Error>> { 
    // Setup terminal for TUI (raw mode, alternate screen)
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app_state = ListState::default();
    app_state.select(Some(0));

    // Main TUI Loop
    loop {
        terminal.draw(|f| {
            let size = f.area(); 
            let block = Block::default()
                .title(format!("rdu: Disk Usage of {}", root_entry.name))
                .borders(Borders::ALL);
            f.render_widget(block, size);

            let mut list_items = Vec::new();
            build_list_items(root_entry, &mut list_items, 0);

            let list = List::new(list_items)
                .block(Block::default().title("Directory Tree").borders(Borders::NONE))
                .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

            f.render_stateful_widget(list, size.inner(Margin::new(1, 1)), &mut app_state);

        })?;

        // Event handling (Exit on 'q' or Esc)
        if let Event::Key(key) = event::read()? {
            if KeyCode::Char('q') == key.code || KeyCode::Esc == key.code {
                break;
            }
        }
    }

    // Restore terminal state upon exit
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}