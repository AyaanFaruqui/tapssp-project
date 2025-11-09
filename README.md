# tapssp-project

**Project Topic: Advanced Concurrent and Hardlink-Aware Disk Usage (du) Utility in Rust**

This project is a high-performance command-line interface (CLI) tool written in Rust that calculates and displays the disk usage of a given directory. This project tackles a fundamental "systems" problem by interacting directly with the file system and managing I/O operations, all while leveraging Rust's ownership and borrowing concepts for safe and efficient memory management.

---

## Final Project Enhancements

This utility has been changed beyond a simple recursive traverse:

### 1. Concurrency and Performance Optimization (Challenge 1)
* **Parallel Traversal:** Uses the **`rayon`** crate to parallelize recursive directory analysis across multiple CPU cores. This converts the I/O-bound problem into a parallelized workload, drastically reducing execution time on large or deep directory trees.
* **System APIs:** Explicitly separates sequential I/O operations (`fs::read_dir`) from concurrent computation (`.par_iter().map().sum()`).

### 2. Hardlink and Deduplication Analysis (Challenge 2 - Current Focus)
* **Apparent Size Mode:** The utility is being upgraded to calculate the *apparent* disk usage by correctly handling hardlinks.
* **Safe Concurrent State:** Implements a global, shared tracking mechanism using **`std::sync::Arc<std::sync::Mutex<std::collections::HashSet>>`** to track unique files based on their **Device ID** and **Inode Number** (`st_dev`, `st_ino`). This ensures that the physical blocks of a hard-linked file are counted only once, even when accessed by multiple concurrent threads.

### 3. Future Work (Visualization - Challenge 3)
* Implementation of a Terminal User Interface (TUI) using a crate like `ratatui` to visualize disk usage as an interactive, hierarchical tree.

---

## Core Functionality

* Recursively traverses a specified directory and all its subdirectories.
* Calculates and displays the total disk usage in bytes.
