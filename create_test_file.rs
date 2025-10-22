use std::io::Write;
use cfb;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut comp = cfb::create("/tmp/test_small.cfb")?;
    
    comp.create_storage("/TestStorage/")?;
    
    let mut stream1 = comp.create_stream("/TestStorage/stream1")?;
    stream1.write_all(b"Hello, world! This is test data for stream 1.")?;
    drop(stream1);
    
    let mut stream2 = comp.create_stream("/stream2")?;  
    stream2.write_all(b"Another test stream with different data pattern.")?;
    drop(stream2);
    
    comp.flush()?;
    println!("Created test CFB file: /tmp/test_small.cfb");
    
    Ok(())
}
