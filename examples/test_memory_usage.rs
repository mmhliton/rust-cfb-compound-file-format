use std::io::Read;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = std::env::args().nth(1)
        .expect("Usage: test_memory_usage <cfb_file>");
    
    println!("Rust native: Opening {}", file_path);
    let mut comp = cfb::open(&file_path)?;
    
    let mut stream_count = 0;
    let mut total_size = 0u64;
    
    // Collect stream paths first to avoid borrowing conflicts
    let stream_paths: Vec<_> = comp.walk()
        .filter(|entry| entry.is_stream())
        .map(|entry| entry.path().to_owned())
        .collect();
    
    for path in stream_paths {
        let mut stream = comp.open_stream(&path)?;
        let mut buffer = vec![0u8; 8192]; // 8KB buffer
        let mut stream_size = 0u64;
        
        loop {
            match stream.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => stream_size += n as u64,
                Err(e) => return Err(e.into()),
            }
        }
        
        total_size += stream_size;
        stream_count += 1;
        
        if stream_count % 20 == 0 {
            println!("  {} streams processed...", stream_count);
        }
    }
    
    println!("Completed: {} streams, {} MB total", 
             stream_count, total_size / (1024 * 1024));
    
    Ok(())
}
