use std::io::Read;
use cfb;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = std::env::args().nth(1)
        .expect("Usage: test_memory_rust <cfb_file>");
    
    println!("Rust native: Opening {}", file_path);
    let comp = cfb::open(&file_path)?;
    
    let mut stream_count = 0;
    let mut total_size = 0u64;
    
    for entry in comp.walk() {
        if entry.is_stream() {
            let mut stream = comp.open_stream(entry.path())?;
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
    }
    
    println!("Completed: {} streams, {} MB total", 
             stream_count, total_size / (1024 * 1024));
    
    Ok(())
}
