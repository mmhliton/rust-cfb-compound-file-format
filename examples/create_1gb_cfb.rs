use cfb::{CompoundFile, Version};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start_time = Instant::now();
    let file_path = "large_1gb.cfb";
    
    println!("Creating 1GB compound file: {}", file_path);
    
    // Create the compound file
    let file = File::create(file_path)?;
    let mut comp = CompoundFile::create_with_version(Version::V4, file)?;
    
    // Target size: 1GB = 1,073,741,824 bytes
    let target_size = 1_073_741_824u64;
    let mut total_written = 0u64;
    
    // Create multiple storage layers
    let storage_layers = vec![
        "Documents",
        "Media", 
        "Projects",
        "Archive",
        "Database"
    ];
    
    for (layer_idx, storage_name) in storage_layers.iter().enumerate() {
        println!("Creating storage layer: {}", storage_name);
        comp.create_storage(Path::new(storage_name))?;
        
        // Create sub-storages within each main storage
        for sub_idx in 0..5 {
            let sub_storage_path = format!("{}/SubStorage_{:02}", storage_name, sub_idx);
            comp.create_storage_all(Path::new(&sub_storage_path))?;
            
            // Create nested storages
            for nested_idx in 0..3 {
                let nested_path = format!("{}/Nested_{:02}", sub_storage_path, nested_idx);
                comp.create_storage_all(Path::new(&nested_path))?;
                
                // Create streams in nested storages
                for stream_idx in 0..10 {
                    let stream_path = format!("{}/stream_{:03}.dat", nested_path, stream_idx);
                    
                    // Calculate stream size to reach target
                    let remaining = target_size - total_written;
                    let streams_remaining = (storage_layers.len() - layer_idx) * 5 * 3 * 10 
                                          - (sub_idx * 3 * 10 + nested_idx * 10 + stream_idx);
                    
                    let stream_size = if streams_remaining > 0 {
                        (remaining / streams_remaining as u64).min(50_000_000) // Max 50MB per stream
                    } else {
                        remaining
                    };
                    
                    if stream_size == 0 {
                        break;
                    }
                    
                    // Create and write to stream
                    let mut stream = comp.create_stream(Path::new(&stream_path))?;
                    
                    // Write data in chunks to avoid memory issues
                    let chunk_size = 1024 * 1024; // 1MB chunks
                    let chunk_data = vec![0xAB; chunk_size];
                    let chunks_needed = (stream_size / chunk_size as u64) as usize;
                    let remainder = (stream_size % chunk_size as u64) as usize;
                    
                    for chunk_idx in 0..chunks_needed {
                        // Vary the data pattern
                        let pattern = ((chunk_idx * 17 + stream_idx * 23) % 256) as u8;
                        let varied_data = vec![pattern; chunk_size];
                        stream.write_all(&varied_data)?;
                    }
                    
                    if remainder > 0 {
                        let pattern = ((chunks_needed * 17 + stream_idx * 23) % 256) as u8;
                        let remainder_data = vec![pattern; remainder];
                        stream.write_all(&remainder_data)?;
                    }
                    
                    total_written += stream_size;
                    
                    if stream_idx % 5 == 0 {
                        print!(".");
                        std::io::stdout().flush()?;
                    }
                    
                    if total_written >= target_size {
                        println!("\nTarget size reached!");
                        break;
                    }
                }
                
                if total_written >= target_size {
                    break;
                }
            }
            
            if total_written >= target_size {
                break;
            }
        }
        
        if total_written >= target_size {
            break;
        }
        
        println!("\nCompleted storage layer: {} ({:.2} MB written so far)", 
                storage_name, total_written as f64 / 1_048_576.0);
    }
    
    // Create some additional streams at root level if needed
    if total_written < target_size {
        println!("Adding root level streams to reach target size...");
        let remaining = target_size - total_written;
        let root_streams = (remaining / 10_000_000).max(1); // At least 1 stream
        
        for i in 0..root_streams {
            let stream_path = format!("root_stream_{:03}.dat", i);
            let stream_size = (remaining / (root_streams - i)).min(100_000_000);
            
            let mut stream = comp.create_stream(Path::new(&stream_path))?;
            
            // Write data in chunks
            let chunk_size = 1024 * 1024; // 1MB chunks
            let chunks_needed = (stream_size / chunk_size as u64) as usize;
            let remainder_size = (stream_size % chunk_size as u64) as usize;
            
            for chunk_idx in 0..chunks_needed {
                let pattern = ((chunk_idx * 13 + (i as usize) * 31) % 256) as u8;
                let chunk_data = vec![pattern; chunk_size];
                stream.write_all(&chunk_data)?;
            }
            
            if remainder_size > 0 {
                let pattern = ((chunks_needed * 13 + (i as usize) * 31) % 256) as u8;
                let remainder_data = vec![pattern; remainder_size];
                stream.write_all(&remainder_data)?;
            }
            
            total_written += stream_size;
            
            if total_written >= target_size {
                break;
            }
        }
    }
    
    // Explicitly close by dropping
    drop(comp);
    
    let duration = start_time.elapsed();
    println!("\nCompound file creation completed!");
    println!("File: {}", file_path);
    println!("Total size written: {:.2} MB ({} bytes)", 
             total_written as f64 / 1_048_576.0, total_written);
    println!("Time taken: {:.2} seconds", duration.as_secs_f64());
    
    // Verify file size
    let metadata = std::fs::metadata(file_path)?;
    println!("Actual file size: {:.2} MB ({} bytes)", 
             metadata.len() as f64 / 1_048_576.0, metadata.len());
    
    Ok(())
}