use std::io::{Cursor, Write};

fn main() -> std::io::Result<()> {
    // Create a simple compound file in memory
    let mut cursor = Cursor::new(Vec::new());
    
    {
        let mut comp = cfb::CompoundFile::create(&mut cursor)?;
        
        // Create a storage directory
        comp.create_storage("TestStorage")?;
        
        // Create a stream with some data
        {
            let mut stream = comp.create_stream("TestStorage/TestStream")?;
            stream.write_all(b"Hello, World! This is test data in a compound file stream.")?;
        }
        
        // Create another stream at root level
        {
            let mut stream = comp.create_stream("RootStream")?;
            stream.write_all(b"This is a root level stream with test data.")?;
        }
        
        comp.flush()?;
    }
    
    // Write the compound file to disk
    std::fs::write("test.cfb", cursor.into_inner())?;
    println!("Created test compound file: test.cfb");
    
    Ok(())
}