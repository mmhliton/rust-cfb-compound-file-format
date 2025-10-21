use std::io::{Cursor, Write};
use std::fs::File;
use std::path::Path;
use rand::prelude::*;
use rand::distributions::Alphanumeric;

fn main() -> std::io::Result<()> {
    println!("Creating very large CFB file (20GB) with random structure...");
    
    // Target size: 20GB = 20 * 1024 * 1024 * 1024 bytes
    const TARGET_SIZE: u64 = 20 * 1024 * 1024 * 1024;
    const CHUNK_SIZE: usize = 1024 * 1024; // 1MB chunks
    
    // Create initial compound file
    let output_path = "large_test_20gb.cfb";
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
            "Archive", "Media", "System", "User", "Cache"
        ];
        
        for main_storage in &main_storages {
            comp.create_storage(main_storage)?;
            storage_count += 1;
            
            // Create sub-storages
            for i in 0..rng.gen_range(5..20) {
                let sub_name = format!("{}/Sub_{:03}", main_storage, i);
                comp.create_storage(&sub_name)?;
                storage_count += 1;
                
                // Create deep nested storages occasionally
                if rng.gen_bool(0.3) {
                    for j in 0..rng.gen_range(1..5) {
                        let deep_name = format!("{}/Deep_{:02}", sub_name, j);
                        comp.create_storage(&deep_name)?;
                        storage_count += 1;
                    }
                }
            }
        }
        
        println!("Created {} storages", storage_count);
        println!("Phase 2: Creating streams with large data...");
        
        // Create streams with varying sizes
        let mut phase = 0;
        while current_size < TARGET_SIZE {
            phase += 1;
            if phase % 100 == 0 {
                println!("Phase {}: Created {} streams, Size: {:.2} GB", 
                    phase, stream_count, current_size as f64 / (1024.0 * 1024.0 * 1024.0));
            }
            
            // Select random storage
            let storage_idx = rng.gen_range(0..main_storages.len());
            let main_storage = &main_storages[storage_idx];
            
            // Decide on stream location (main storage vs sub-storage)
            let stream_path = if rng.gen_bool(0.4) {
                // Main storage
                let stream_name = generate_random_filename(&mut rng);
                format!("{}/{}", main_storage, stream_name)
            } else {
                // Sub-storage
                let sub_idx = rng.gen_range(0..20);
                let stream_name = generate_random_filename(&mut rng);
                format!("{}/Sub_{:03}/{}", main_storage, sub_idx, stream_name)
            };
            
            // Generate stream size (vary from small to very large)
            let stream_size = match rng.gen_range(0..100) {
                0..=60 => rng.gen_range(1024..1024*1024),           // 1KB - 1MB (60%)
                61..=85 => rng.gen_range(1024*1024..10*1024*1024),  // 1MB - 10MB (25%)
                86..=95 => rng.gen_range(10*1024*1024..100*1024*1024), // 10MB - 100MB (10%)
                _ => rng.gen_range(100*1024*1024..500*1024*1024),   // 100MB - 500MB (5%)
            };
            
            // Create the stream with random data
            match comp.create_stream(&stream_path) {
                Ok(mut stream) => {
                    let mut written = 0;
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
                        
                        // Check if we've reached target size
                        if current_size >= TARGET_SIZE {
                            break;
                        }
                    }
                    stream_count += 1;
                }
                Err(e) => {
                    println!("Warning: Failed to create stream {}: {}", stream_path, e);
                }
            }
            
            // Break if we've reached target size
            if current_size >= TARGET_SIZE {
                break;
            }
            
            // Flush periodically to manage memory
            if phase % 50 == 0 {
                comp.flush()?;
            }
        }
        
        println!("Phase 3: Finalizing compound file...");
        comp.flush()?;
        
        println!("Created {} storages and {} streams", storage_count, stream_count);
        println!("Total size: {:.2} GB", current_size as f64 / (1024.0 * 1024.0 * 1024.0));
    }
    
    println!("Phase 4: Writing to disk...");
    std::fs::write(output_path, cursor.into_inner())?;
    
    // Verify file size
    let metadata = std::fs::metadata(output_path)?;
    let file_size_gb = metadata.len() as f64 / (1024.0 * 1024.0 * 1024.0);
    
    println!("✅ Successfully created large CFB file!");
    println!("📁 File: {}", output_path);
    println!("📊 Size: {:.2} GB ({} bytes)", file_size_gb, metadata.len());
    
    Ok(())
}

fn generate_random_filename(rng: &mut ThreadRng) -> String {
    let extensions = vec!["txt", "dat", "bin", "log", "xml", "json", "csv", "doc", "pdf"];
    let prefixes = vec!["data", "file", "document", "report", "log", "config", "temp", "backup"];
    
    let prefix = prefixes.choose(rng).unwrap();
    let extension = extensions.choose(rng).unwrap();
    let random_suffix: String = (0..8)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect();
    
    format!("{}_{}.{}", prefix, random_suffix, extension)
}

fn generate_random_data(size: usize, rng: &mut ThreadRng) -> Vec<u8> {
    let mut data = Vec::with_capacity(size);
    
    // Mix of different data patterns for realism
    let pattern_type = rng.gen_range(0..5);
    
    match pattern_type {
        0 => {
            // Random binary data
            for _ in 0..size {
                data.push(rng.gen());
            }
        }
        1 => {
            // Text-like data
            let text_chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789 \n\t.,;:!?";
            for _ in 0..size {
                data.push(text_chars[rng.gen_range(0..text_chars.len())]);
            }
        }
        2 => {
            // Repeated patterns (like compressed data)
            let pattern = vec![0xDE, 0xAD, 0xBE, 0xEF];
            for i in 0..size {
                data.push(pattern[i % pattern.len()]);
            }
        }
        3 => {
            // Structured data (simulating XML/JSON)
            let template = b"<data id=\"000000\" value=\"XXXXXXXXXXXXXXXX\" timestamp=\"2025-10-18T00:00:00Z\"/>\n";
            for i in 0..size {
                data.push(template[i % template.len()]);
            }
        }
        _ => {
            // Mixed content
            for i in 0..size {
                if i % 1024 < 512 {
                    data.push(rng.gen());
                } else {
                    data.push(b' ');
                }
            }
        }
    }
    
    data
}