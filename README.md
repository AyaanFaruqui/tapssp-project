# tapssp-project: Concurrent, Hardlink-Aware Disk Usage (rdu)

This project is an advanced, high-performance **Terminal User Interface (TUI)** tool written in Rust. It calculates and visualizes the true disk usage of a directory, significantly exceeding the functionality of standard utilities like `du`.

This utility tackles core **"systems"** problems—**concurrency/performance** and **accurate disk accounting**—using Rust's ownership model and advanced crate APIs for a robust solution.

***

##  Final Project Challenges 

The project is complete, successfully integrating all three major challenges:

### 1. Concurrency and Performance Optimization (Challenge 1)
| Feature | Technical Solution |
| :--- | :--- |
| **Parallel Traversal** | The **`rayon`** crate is used to split the recursive directory analysis across multiple CPU cores, converting the I/O-bound problem into a parallelized workload for maximum speed. |
| **System APIs** | Explicitly separates sequential I/O operations (`fs::read_dir`) from concurrent computation (`.par_iter().map().sum()`). |

### 2. Hardlink Analysis and Deduplication (Challenge 2)
| Feature | Technical Solution |
| :--- | :--- |
| **Apparent Size** | The tool calculates the *true physical size* by counting the disk blocks of a unique file **only once**, correctly handling files with multiple hardlinks. |
| **Safe Shared State** | Implements a robust concurrent tracking system using **`std::sync::Arc<std::sync::Mutex<same_file::Handle>>`** to safely share the list of already-counted unique file IDs across all parallel threads. |
| **API Usability** | Uses the stable **`same-file`** crate to reliably retrieve unique file identifiers (Inode/Device Handles) on any operating system. |

### 3. TUI Visualization (Challenge 3)
| Feature | Technical Solution |
| :--- | :--- |
| **Interactive Interface** | Uses the **`ratatui`** and **`crossterm`** crates to render the output in a responsive Terminal User Interface. |
| **Hierarchical View** | The final size data is presented as a navigable, hierarchical directory tree (`DirEntry` struct) for quick analysis. |
| **Visualization** | Directories are color-coded based on their relative size (e.g., Red for very large consumers) for immediate visual feedback. |

***

## Core Functionality

* Recursively traverses a specified directory and all its subdirectories.
* Calculates and displays the total disk usage in bytes.
* Provides a visual TUI for easy navigation and analysis.
