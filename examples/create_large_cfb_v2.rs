use std::io::{Cursor, Write};
use rand::prelude::*;
use rand::distributions::Alphanumeric;

fn main() -> std::io::Result<()> {
    // Start with a smaller test (1GB) to verify the approach works
    let target_size_gb = std::env::args()
        .nth(1)
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(1); // Default to 1GB for testing
    
    let target_size = target_size_gb * 1024 * 1024 * 1024;
    let output_path = format!("large_test_{}gb.cfb", target_size_gb);
    
    println!("Creating large CFB file ({} GB) with random structure...", target_size_gb);
    println!("Output file: {}", output_path);
    
    const CHUNK_SIZE: usize = 512 * 1024; // 512KB chunks for efficiency
    
    let mut cursor = Cursor::new(Vec::new());
    let mut rng = thread_rng();
    
    {
        let mut comp = cfb::CompoundFile::create(&mut cursor)?;
        let mut current_size = 0u64;
        let mut storage_count = 0;
        let mut stream_count = 0;
        
        println!("Phase 1: Creating storage hierarchy...");
        
        // Create main storage categories
        let main_storages = vec![
            "Documents", "Images", "Data", "Config", "Temp", 
            "Archive", "Media", "System", "User", "Cache",
            "Reports", "Logs", "Backup", "Projects", "Resources"
        ];
        
        for main_storage in &main_storages {
            comp.create_storage(main_storage)?;
            storage_count += 1;
            
            // Create sub-storages
            let sub_count = rng.gen_range(3..15);
            for i in 0..sub_count {
                let sub_name = format!("{}/Sub_{:03}", main_storage, i);
                comp.create_storage(&sub_name)?;
                storage_count += 1;
                
                // Create deep nested storages occasionally
                if rng.gen_bool(0.2) {
                    let deep_count = rng.gen_range(1..4);
                    for j in 0..deep_count {
                        let deep_name = format!("{}/Deep_{:02}", sub_name, j);
                        comp.create_storage(&deep_name)?;
                        storage_count += 1;
                    }
                }
            }
        }
        
        println!("Created {} storages", storage_count);
        println!("Phase 2: Creating streams with data...");
        
        let mut phase = 0;
        while current_size < target_size {
            phase += 1;
            
            if phase % 20 == 0 {
                let progress = (current_size as f64 / target_size as f64) * 100.0;
                let size_gb = current_size as f64 / (1024.0 * 1024.0 * 1024.0);
                println!("Progress: {:.1}% - Created {} streams, Size: {:.2} GB", 
                    progress, stream_count, size_gb);
            }
            
            // Select random storage location
            let storage_idx = rng.gen_range(0..main_storages.len());
            let main_storage = &main_storages[storage_idx];
            
            let stream_path = match rng.gen_range(0..10) {
                0..=3 => {
                    // Main storage (40%)
                    let stream_name = generate_random_filename(&mut rng);
                    format!("{}/{}", main_storage, stream_name)
                }
                4..=7 => {
                    // Sub-storage (40%)
                    let sub_idx = rng.gen_range(0..15);
                    let stream_name = generate_random_filename(&mut rng);
                    format!("{}/Sub_{:03}/{}", main_storage, sub_idx, stream_name)
                }
                _ => {
                    // Deep storage (20%)
                    let sub_idx = rng.gen_range(0..15);
                    let deep_idx = rng.gen_range(0..4);
                    let stream_name = generate_random_filename(&mut rng);
                    format!("{}/Sub_{:03}/Deep_{:02}/{}", main_storage, sub_idx, deep_idx, stream_name)
                }
            };
            
            // Generate stream size with better distribution
            let remaining = target_size - current_size;
            let max_stream_size = std::cmp::min(remaining, 50 * 1024 * 1024) as usize; // Max 50MB per stream
            
            let stream_size = match rng.gen_range(0..100) {
                0..=50 => rng.gen_range(1024..512*1024),                    // 1KB - 512KB (50%)
                51..=80 => rng.gen_range(512*1024..5*1024*1024),            // 512KB - 5MB (30%)
                81..=95 => rng.gen_range(5*1024*1024..20*1024*1024),        // 5MB - 20MB (15%)
                _ => rng.gen_range(20*1024*1024..max_stream_size),          // 20MB - 50MB (5%)
            };
            
            let stream_size = std::cmp::min(stream_size, remaining as usize);
            
            // Create the stream
            match comp.create_stream(&stream_path) {
                Ok(mut stream) => {
                    let mut written = 0usize;
                    while written < stream_size {
                        let chunk_size = std::cmp::min(CHUNK_SIZE, stream_size - written);
                        let chunk = generate_random_data(chunk_size, &mut rng);
                        
                        match stream.write_all(&chunk) {
                            Ok(_) => {
                                written += chunk_size;
                                current_size += chunk_size as u64;
                            }
                            Err(e) => {
                                println!("Warning: Failed to write to stream {}: {}", stream_path, e);
                                break;
                            }
                        }
                        
                        if current_size >= target_size {
                            break;
                        }
                    }
                    stream_count += 1;
                }
                Err(_) => {
                    // Try to create parent storage if it doesn't exist
                    if let Some(parent_path) = stream_path.rfind('/') {
                        let parent = &stream_path[..parent_path];
                        if let Err(_) = comp.create_storage(parent) {
                            // Parent already exists or creation failed
                        }
                        // Try creating stream again
                        if let Ok(mut stream) = comp.create_stream(&stream_path) {
                            let chunk = generate_random_data(std::cmp::min(CHUNK_SIZE, stream_size), &mut rng);
                            if let Ok(_) = stream.write_all(&chunk) {
                                current_size += chunk.len() as u64;
                                stream_count += 1;
                            }
                        }
                    }
                }
            }
            
            if current_size >= target_size {
                break;
            }
            
            // Flush periodically
            if phase % 100 == 0 {
                comp.flush()?;
            }
        }
        
        println!("Phase 3: Finalizing compound file...");
        comp.flush()?;
        
        let final_size_gb = current_size as f64 / (1024.0 * 1024.0 * 1024.0);
        println!("Created {} storages and {} streams", storage_count, stream_count);
        println!("Total content size: {:.2} GB", final_size_gb);
    }
    
    println!("Phase 4: Writing to disk...");
    std::fs::write(&output_path, cursor.into_inner())?;
    
    // Verify file size
    let metadata = std::fs::metadata(&output_path)?;
    let file_size_gb = metadata.len() as f64 / (1024.0 * 1024.0 * 1024.0);
    
    println!("✅ Successfully created large CFB file!");
    println!("📁 File: {}", output_path);
    println!("📊 Size: {:.2} GB ({} bytes)", file_size_gb, metadata.len());
    println!("\n🔍 Use cfbtool to explore the file:");
    println!("cargo run --example cfbtool -- ls --all {}:", output_path);
    
    Ok(())
}

fn generate_random_filename(rng: &mut ThreadRng) -> String {
    let extensions = vec![
        "txt", "dat", "bin", "log", "xml", "json", "csv", "doc", "pdf", 
        "xlsx", "pptx", "zip", "tar", "cfg", "ini", "tmp", "bak", "old"
    ];
    let prefixes = vec![
        "data", "file", "document", "report", "log", "config", "temp", 
        "backup", "archive", "cache", "index", "metadata", "summary", "detail"
    ];
    
    let prefix = prefixes.choose(rng).unwrap();
    let extension = extensions.choose(rng).unwrap();
    let random_suffix: String = (0..6)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect();
    
    format!("{}_{}.{}", prefix, random_suffix, extension)
}

fn generate_random_data(size: usize, rng: &mut ThreadRng) -> Vec<u8> {
    let mut data = Vec::with_capacity(size);
    
    // Different data patterns for variety
    let pattern_type = rng.gen_range(0..6);
    
    match pattern_type {
        0 => {
            // Random binary data
            data.resize(size, 0);
            rng.fill_bytes(&mut data);
        }
        1 => {
            // Text-like data
            let text_chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789 \n\t.,;:!?()[]{}";
            for _ in 0..size {
                data.push(text_chars[rng.gen_range(0..text_chars.len())]);
            }
        }
        2 => {
            // Structured data pattern
            let pattern = b"<record><id>12345</id><data>ABCDEFGHIJKLMNOP</data><timestamp>2025-10-18</timestamp></record>\n";
            for i in 0..size {
                data.push(pattern[i % pattern.len()]);
            }
        }
        3 => {
            // JSON-like pattern
            let pattern = b"{\"id\":12345,\"name\":\"sample_data\",\"value\":\"XXXXXXXXXXXXXXXX\",\"active\":true}\n";
            for i in 0..size {
                data.push(pattern[i % pattern.len()]);
            }
        }
        4 => {
            // Log file pattern
            let pattern = b"2025-10-18 12:00:00 [INFO] Processing record 12345 - Status: OK - Duration: 123ms\n";
            for i in 0..size {
                data.push(pattern[i % pattern.len()]);
            }
        }
        _ => {
            // Mixed binary/text
            for i in 0..size {
                if i % 64 < 32 {
                    data.push(rng.gen());
                } else {
                    data.push(b'A' + (i % 26) as u8);
                }
            }
        }
    }
    
    data
}