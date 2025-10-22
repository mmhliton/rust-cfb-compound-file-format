use cfb::CompoundFile;
use std::fs::File;
use std::path::Path;
use std::time::Instant;

fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    
    if size == 0 {
        return "0 B".to_string();
    }
    
    let mut size_f = size as f64;
    let mut unit_idx = 0;
    
    while size_f >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size_f /= 1024.0;
        unit_idx += 1;
    }
    
    if unit_idx == 0 {
        format!("{} {}", size, UNITS[unit_idx])
    } else {
        format!("{:.2} {}", size_f, UNITS[unit_idx])
    }
}

fn traverse_and_print_streams(comp: &mut CompoundFile<File>, path: &str, level: usize) -> Result<(usize, u64), Box<dyn std::error::Error>> {
    let mut stream_count = 0;
    let mut total_size = 0u64;
    let indent = "  ".repeat(level);
    
    // Try to list entries in the current storage
    let entries = if path.is_empty() {
        comp.walk()
    } else {
        match comp.walk_storage(Path::new(path)) {
            Ok(iter) => iter,
            Err(_) => {
                println!("{}⚠️  Cannot traverse storage: {}", indent, path);
                return Ok((0, 0));
            }
        }
    };
    
    let mut children: Vec<_> = entries.collect();
    children.sort_by(|a, b| {
        // Sort storages first, then streams
        match (a.is_storage(), b.is_storage()) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.path().to_string_lossy().cmp(&b.path().to_string_lossy()),
        }
    });
    
    for entry in children {
        let entry_path = entry.path().to_string_lossy();
        let name = entry_path.split('/').last().unwrap_or(&entry_path);
        
        if entry.is_stream() {
            let size = entry.len();
            total_size += size;
            stream_count += 1;
            
            println!("{}📄 {} ({})", indent, name, format_size(size));
            
            // Optional: Read and verify stream data (commented out for performance)
            /*
            match comp.open_stream(entry.path()) {
                Ok(mut stream) => {
                    let mut buffer = vec![0u8; 1024]; // Read first 1KB
                    match stream.read(&mut buffer) {
                        Ok(bytes_read) => {
                            println!("{}   ✓ Successfully read {} bytes", indent, bytes_read);
                        },
                        Err(e) => {
                            println!("{}   ⚠️  Error reading stream: {}", indent, e);
                        }
                    }
                },
                Err(e) => {
                    println!("{}   ⚠️  Error opening stream: {}", indent, e);
                }
            }
            */
        } else if entry.is_storage() {
            println!("{}📁 {}/", indent, name);
            
            // Recursively traverse sub-storage
            match traverse_and_print_streams(comp, &entry_path, level + 1) {
                Ok((sub_streams, sub_size)) => {
                    stream_count += sub_streams;
                    total_size += sub_size;
                    if sub_streams > 0 {
                        println!("{}   └─ {} streams, {}", indent, sub_streams, format_size(sub_size));
                    }
                },
                Err(e) => {
                    println!("{}   ⚠️  Error traversing: {}", indent, e);
                }
            }
        }
    }
    
    Ok((stream_count, total_size))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let file_path = if args.len() > 1 {
        &args[1]
    } else {
        "large_1gb.cfb"
    };
    
    println!("Loading compound file: {}", file_path);
    
    if !Path::new(file_path).exists() {
        eprintln!("Error: File '{}' does not exist.", file_path);
        eprintln!("Usage: {} [path_to_cfb_file]", args[0]);
        eprintln!("Default file: large_1gb.cfb");
        std::process::exit(1);
    }
    
    let start_time = Instant::now();
    
    // Open the compound file
    let file = File::open(file_path)?;
    let mut comp = CompoundFile::open(file)?;
    
    // Get file metadata
    let metadata = std::fs::metadata(file_path)?;
    println!("File size: {}", format_size(metadata.len()));
    println!("CFB Version: {:?}", comp.version());
    println!();
    
    println!("🗂️  Compound File Structure:");
    println!("═══════════════════════════════");
    
    // Traverse and print all streams
    let (total_streams, total_stream_size) = traverse_and_print_streams(&mut comp, "", 0)?;
    
    let duration = start_time.elapsed();
    
    println!();
    println!("📊 Summary:");
    println!("═══════════");
    println!("Total streams found: {}", total_streams);
    println!("Total stream data: {}", format_size(total_stream_size));
    println!("File overhead: {}", format_size(metadata.len() - total_stream_size));
    println!("Efficiency: {:.1}%", (total_stream_size as f64 / metadata.len() as f64) * 100.0);
    println!("Traversal time: {:.2} seconds", duration.as_secs_f64());
    
    // Additional statistics
    if total_streams > 0 {
        println!("Average stream size: {}", format_size(total_stream_size / total_streams as u64));
    }
    
    Ok(())
}