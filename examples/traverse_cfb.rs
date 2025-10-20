use std::io::Read;
use std::path::Path;
use std::time::Instant;

/// Recursively traverse a compound file starting at `path` (empty path = root).
/// Prints storages and streams; for streams prints size and a short preview.
fn traverse<F: std::io::Read + std::io::Seek>(
    comp: &mut cfb::CompoundFile<F>,
    path: &str,
    depth: usize,
    print_streams: bool,
) {
    // Fetch entry for current path (root is "").
    let entry = match comp.entry(path) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("[warn] Cannot access '{}': {}", path, e);
            return;
        }
    };

    let indent = "  ".repeat(depth);
    if entry.is_storage() {
        println!("{}📁 {}", indent, display_name(path, &entry));
        // Iterate children of this storage
        // Collect child paths first to satisfy borrow checker.
        let child_paths: Vec<String> = comp
            .read_storage(path)
            .unwrap()
            .map(|child| normalize_child_path(path, child.name(), child.is_storage()))
            .collect();
        for child_path in child_paths {
            traverse(comp, &child_path, depth + 1, print_streams);
        }
    } else {
        // Stream: optionally print preview
        if print_streams {
            let mut stream = match comp.open_stream(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{}[error] Cannot open stream '{}': {}", indent, path, e);
                    return;
                }
            };
            let mut buf = [0u8; 64];
            let read_len = stream.read(&mut buf).unwrap_or(0);
            let preview_hex = buf[..read_len]
                .iter()
                .map(|b| format!("{:02x}", b))
                .take(16)
                .collect::<Vec<_>>()
                .join(" ");
            let preview_txt: String = buf[..read_len]
                .iter()
                .map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' })
                .collect();
            println!(
                "{}📄 {} (len={} bytes)\n{}   hex: {}\n{}   txt: {}",
                indent,
                display_name(path, &entry),
                entry.len(),
                indent,
                preview_hex,
                indent,
                preview_txt
            );
        }
    }
}

fn display_name(path: &str, entry: &cfb::Entry) -> String {
    if path.is_empty() { ".(root)".to_string() } else { entry.name().to_string() }
}

fn normalize_child_path(parent: &str, child_name: &str, is_storage: bool) -> String {
    // Storage names may or may not have trailing slash; normalize compound file API expectation.
    if parent.is_empty() {
        if is_storage { child_name.to_string() } else { child_name.to_string() }
    } else {
        format!("{}/{}", parent.trim_end_matches('/'), child_name)
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: traverse_cfb <compound-file-path> [print-streams=true|false]");
        std::process::exit(1);
    }
    let file_path = &args[1];
    let print_streams = args.get(2).map(|s| s.eq_ignore_ascii_case("true")).unwrap_or(true);
    if !Path::new(file_path).exists() {
        eprintln!("File '{}' does not exist", file_path);
        std::process::exit(1);
    }
    let mut comp = match cfb::open(file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to open '{}': {}", file_path, e);
            std::process::exit(1);
        }
    };
    println!("Traversing compound file: {}", file_path);
    let start = Instant::now();
    traverse(&mut comp, "", 0, print_streams);
    let elapsed = start.elapsed();
    // File size for summary
    let file_size = std::fs::metadata(file_path).map(|m| m.len()).unwrap_or(0);
    let human_size = human_readable_size(file_size);
    println!(
        "Traversal completed in {:.3?} (file_size={} bytes / {})",
        elapsed,
        file_size,
        human_size
    );
}

fn human_readable_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < (1024 * 1024) {
        format!("{:.2} KB", bytes as f64 / KB)
    } else if bytes < (1024 * 1024 * 1024) {
        format!("{:.2} MB", bytes as f64 / MB)
    } else {
        format!("{:.2} GB", bytes as f64 / GB)
    }
}
