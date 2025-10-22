use cfb::CompoundFile;
use std::fs::File;
use std::io::{Write, Seek, SeekFrom};
use std::path::Path;
use std::time::Instant;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

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

fn collect_all_streams(comp: &CompoundFile<File>) -> Vec<std::path::PathBuf> {
    let mut streams = Vec::new();
    
    for entry in comp.walk() {
        if entry.is_stream() {
            streams.push(entry.path().to_path_buf());
        }
    }
    
    streams
}

fn generate_random_data(size: usize, seed: u64) -> Vec<u8> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    (0..size).map(|_| rng.gen::<u8>()).collect()
}

fn modify_stream_randomly(comp: &mut CompoundFile<File>, stream_path: &Path, modification_type: usize) -> Result<(u64, u64), Box<dyn std::error::Error>> {
    let original_size = {
        let stream = comp.open_stream(stream_path)?;
        stream.len()
    };
    
    let mut rng = rand::thread_rng();
    
    match modification_type {
        0 => {
            // Type 1: Overwrite beginning with random data
            let mut stream = comp.open_stream(stream_path)?;
            let overwrite_size = (original_size / 4).min(1024 * 1024).max(1024) as usize; // 25% or 1MB max, 1KB min
            
            let new_data = generate_random_data(overwrite_size, rng.gen());
            stream.seek(SeekFrom::Start(0))?;
            stream.write_all(&new_data)?;
            
            println!("    • Overwrote beginning {} with random data", format_size(overwrite_size as u64));
            Ok((original_size, original_size))
        },
        1 => {
            // Type 2: Append random data
            let mut stream = comp.open_stream(stream_path)?;
            let append_size = rng.gen_range(1024..=1024*1024); // 1KB to 1MB
            
            let new_data = generate_random_data(append_size, rng.gen());
            stream.seek(SeekFrom::End(0))?;
            stream.write_all(&new_data)?;
            
            let new_size = original_size + append_size as u64;
            println!("    • Appended {} of random data", format_size(append_size as u64));
            Ok((original_size, new_size))
        },
        2 => {
            // Type 3: Truncate and resize
            let mut stream = comp.open_stream(stream_path)?;
            let new_size = if original_size > 2048 {
                // Resize to 50-90% of original size
                let factor = rng.gen_range(0.5..0.9);
                (original_size as f64 * factor) as u64
            } else {
                // Small streams: just double them
                original_size * 2
            };
            
            stream.set_len(new_size)?;
            
            if new_size > original_size {
                // Fill new space with pattern
                let fill_size = new_size - original_size;
                let pattern = rng.gen::<u8>();
                let fill_data = vec![pattern; fill_size as usize];
                stream.seek(SeekFrom::Start(original_size))?;
                stream.write_all(&fill_data)?;
                println!("    • Resized from {} to {} (grew by {})", 
                        format_size(original_size), format_size(new_size), format_size(fill_size));
            } else {
                println!("    • Resized from {} to {} (shrunk by {})", 
                        format_size(original_size), format_size(new_size), format_size(original_size - new_size));
            }
            
            Ok((original_size, new_size))
        },
        3 => {
            // Type 4: Random overwrites at multiple positions
            let mut stream = comp.open_stream(stream_path)?;
            let num_patches = rng.gen_range(2..=5);
            let patch_size = (original_size / 20).min(4096).max(256) as usize; // Small patches
            
            let mut total_modified = 0;
            for i in 0..num_patches {
                if original_size <= patch_size as u64 {
                    break;
                }
                
                let max_pos = original_size - patch_size as u64;
                let pos = rng.gen_range(0..=max_pos);
                
                let patch_data = generate_random_data(patch_size, rng.gen::<u64>() + i as u64);
                stream.seek(SeekFrom::Start(pos))?;
                stream.write_all(&patch_data)?;
                total_modified += patch_size;
            }
            
            println!("    • Applied {} random patches, {} total modified", 
                    num_patches, format_size(total_modified as u64));
            Ok((original_size, original_size))
        },
        _ => {
            // Default: simple overwrite
            let mut stream = comp.open_stream(stream_path)?;
            let data = b"MODIFIED_DATA_MARKER";
            stream.seek(SeekFrom::Start(0))?;
            stream.write_all(data)?;
            
            println!("    • Added modification marker");
            Ok((original_size, original_size))
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let file_path = if args.len() > 1 {
        &args[1]
    } else {
        "large_1gb.cfb"
    };
    
    let num_modifications = if args.len() > 2 {
        args[2].parse::<usize>().unwrap_or(3)
    } else {
        3
    };
    
    println!("Randomly modifying {} streams in: {}", num_modifications, file_path);
    
    if !Path::new(file_path).exists() {
        eprintln!("Error: File '{}' does not exist.", file_path);
        eprintln!("Usage: {} [path_to_cfb_file] [num_modifications]", args[0]);
        eprintln!("Default: large_1gb.cfb 3");
        std::process::exit(1);
    }
    
    let start_time = Instant::now();
    
    // Open the compound file for modification
    let file = File::options().read(true).write(true).open(file_path)?;
    let mut comp = CompoundFile::open(file)?;
    
    // Collect all streams
    println!("Scanning for streams...");
    let all_streams = collect_all_streams(&comp);
    
    if all_streams.is_empty() {
        println!("No streams found in the compound file!");
        return Ok(());
    }
    
    println!("Found {} streams total", all_streams.len());
    
    let modifications_to_make = num_modifications.min(all_streams.len());
    println!("Will modify {} streams", modifications_to_make);
    println!();
    
    // Randomly select streams to modify
    let mut rng = rand::thread_rng();
    let mut selected_indices: Vec<usize> = (0..all_streams.len()).collect();
    
    // Fisher-Yates shuffle to get random selection
    for i in (1..selected_indices.len()).rev() {
        let j = rng.gen_range(0..=i);
        selected_indices.swap(i, j);
    }
    
    let mut total_size_before = 0u64;
    let mut total_size_after = 0u64;
    
    for i in 0..modifications_to_make {
        let stream_index = selected_indices[i];
        let stream_path = &all_streams[stream_index];
        let modification_type = rng.gen_range(0..5);
        
        println!("{}. Modifying: {}", i + 1, stream_path.to_string_lossy());
        
        match modify_stream_randomly(&mut comp, stream_path, modification_type) {
            Ok((before, after)) => {
                total_size_before += before;
                total_size_after += after;
                println!("    ✓ Success: {} → {}", format_size(before), format_size(after));
            },
            Err(e) => {
                println!("    ✗ Error: {}", e);
            }
        }
        println!();
    }
    
    // Explicitly flush changes
    drop(comp);
    
    let duration = start_time.elapsed();
    
    println!("🔄 Modification Summary:");
    println!("═════════════════════");
    println!("Streams modified: {}", modifications_to_make);
    println!("Total stream data before: {}", format_size(total_size_before));
    println!("Total stream data after: {}", format_size(total_size_after));
    
    if total_size_after > total_size_before {
        println!("Net change: +{}", format_size(total_size_after - total_size_before));
    } else if total_size_before > total_size_after {
        println!("Net change: -{}", format_size(total_size_before - total_size_after));
    } else {
        println!("Net change: No size change");
    }
    
    println!("Modification time: {:.2} seconds", duration.as_secs_f64());
    
    // Verify file integrity
    println!();
    println!("Verifying file integrity...");
    let verify_file = File::open(file_path)?;
    match CompoundFile::open(verify_file) {
        Ok(_) => println!("✓ File integrity verified - compound file is still valid"),
        Err(e) => println!("✗ File integrity check failed: {}", e),
    }
    
    Ok(())
}