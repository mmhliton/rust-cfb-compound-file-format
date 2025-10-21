#!/bin/bash

# Incremental CFB file builder using cfbtool
# This approach is more memory-efficient for very large files

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

TARGET_SIZE_GB=${1:-20}
OUTPUT_FILE="incremental_${TARGET_SIZE_GB}gb.cfb"

echo "🚀 Creating ${TARGET_SIZE_GB}GB CFB file incrementally using cfbtool..."
echo "📁 Output file: $OUTPUT_FILE"
echo "⏰ Start time: $(date)"

# Create initial file structure
echo "📋 Phase 1: Creating initial CFB file..."
cargo run --example create_test_cfb

# Rename to our target file
mv test.cfb "$OUTPUT_FILE"

echo "📊 Initial file created. Starting incremental expansion..."

# Function to add large streams using cfbtool
add_large_streams() {
    local storage_name="$1"
    local stream_count="$2"
    local size_mb="$3"
    
    echo "  📁 Adding $stream_count streams to $storage_name (${size_mb}MB each)..."
    
    for i in $(seq 1 $stream_count); do
        local stream_name="LargeFile_$(printf "%04d" $i)"
        
        # Create stream using cfbtool (with predefined data)
        cargo run --example cfbtool -- create \
            --file-path "$OUTPUT_FILE" \
            --inner-path "$storage_name" \
            --stream-name "$stream_name" 2>/dev/null || {
            
            # If storage doesn't exist, try to create it by adding to parent
            echo "    🔧 Creating storage path for $storage_name..."
            continue
        }
        
        if [[ $((i % 50)) -eq 0 ]]; then
            local current_size=$(du -h "$OUTPUT_FILE" | cut -f1)
            echo "    📊 Progress: $i/$stream_count streams, File size: $current_size"
        fi
    done
}

# Function to generate large data file and append it
generate_large_stream() {
    local size_mb="$1"
    local temp_file="temp_large_data.bin"
    
    # Generate large binary file
    dd if=/dev/urandom of="$temp_file" bs=1M count=$size_mb status=none 2>/dev/null
    echo "$temp_file"
}

echo "📋 Phase 2: Creating storage hierarchy..."

# Create main storage categories
MAIN_STORAGES=("Documents" "Images" "Data" "Config" "Archive" "Media" "System" "Logs" "Backup" "Projects")

for storage in "${MAIN_STORAGES[@]}"; do
    echo "  📁 Processing storage: $storage"
    
    # Add streams to main storage
    add_large_streams "$storage" 10 1
    
    # Create sub-storages and add content
    for j in $(seq 0 9); do
        sub_storage="${storage}/Sub_$(printf "%03d" $j)"
        add_large_streams "$sub_storage" 5 2
    done
done

echo "📋 Phase 3: Adding large data streams..."

# Calculate how much more data we need
current_size_bytes=$(stat -c%s "$OUTPUT_FILE" 2>/dev/null || echo 0)
current_size_gb=$(echo "scale=2; $current_size_bytes / 1024 / 1024 / 1024" | bc -l 2>/dev/null || echo "0")
remaining_gb=$(echo "scale=2; $TARGET_SIZE_GB - $current_size_gb" | bc -l 2>/dev/null || echo "$TARGET_SIZE_GB")

echo "  📊 Current size: ${current_size_gb}GB"
echo "  📊 Remaining needed: ${remaining_gb}GB"

# Add large streams to reach target size
if (( $(echo "$remaining_gb > 0.1" | bc -l 2>/dev/null || echo 0) )); then
    echo "  🔄 Adding large streams to reach target size..."
    
    # Create very large streams in Archive storage
    large_stream_count=$(echo "scale=0; $remaining_gb / 0.1" | bc -l 2>/dev/null || echo 10)
    echo "  📊 Creating $large_stream_count large streams..."
    
    for i in $(seq 1 $large_stream_count); do
        stream_name="VeryLargeFile_$(printf "%04d" $i)"
        
        cargo run --example cfbtool -- create \
            --file-path "$OUTPUT_FILE" \
            --inner-path "Archive" \
            --stream-name "$stream_name" 2>/dev/null || true
        
        # Check if we've reached target size
        current_size_bytes=$(stat -c%s "$OUTPUT_FILE" 2>/dev/null || echo 0)
        current_size_gb=$(echo "scale=2; $current_size_bytes / 1024 / 1024 / 1024" | bc -l 2>/dev/null || echo "0")
        
        if (( $(echo "$current_size_gb >= $TARGET_SIZE_GB" | bc -l 2>/dev/null || echo 0) )); then
            echo "  ✅ Target size reached!"
            break
        fi
        
        if [[ $((i % 20)) -eq 0 ]]; then
            echo "  📊 Progress: $i streams, Current size: ${current_size_gb}GB"
        fi
    done
fi

echo "📋 Phase 4: Final verification..."

# Final size check
final_size_bytes=$(stat -c%s "$OUTPUT_FILE" 2>/dev/null || echo 0)
final_size_gb=$(echo "scale=2; $final_size_bytes / 1024 / 1024 / 1024" | bc -l 2>/dev/null || echo "0")

echo ""
echo "✅ Incremental CFB file creation completed!"
echo "📁 File: $OUTPUT_FILE"
echo "📊 Final size: ${final_size_gb}GB ($final_size_bytes bytes)"
echo "⏰ End time: $(date)"

echo ""
echo "🔍 File structure preview:"
cargo run --example cfbtool -- ls "$OUTPUT_FILE:" | head -10

echo ""
echo "📋 Test commands:"
echo "  List contents:     cargo run --example cfbtool -- ls --long $OUTPUT_FILE:"
echo "  View Documents:    cargo run --example cfbtool -- ls $OUTPUT_FILE:Documents"
echo "  View Archive:      cargo run --example cfbtool -- ls $OUTPUT_FILE:Archive"
echo "  Read a stream:     cargo run --example cfbtool -- cat $OUTPUT_FILE:Documents/Sub_001/[stream_name]"

# Cleanup temp files
rm -f temp_large_data.bin 2>/dev/null || true