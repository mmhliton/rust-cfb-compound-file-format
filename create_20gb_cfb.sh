#!/bin/bash

# Script to create a very large CFB file (20GB) with monitoring
# Usage: ./create_20gb_cfb.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "🚀 Starting 20GB CFB file creation..."
echo "📍 Working directory: $(pwd)"
echo "⏰ Start time: $(date)"
echo "💾 Available disk space:"
df -h .

echo ""
echo "⚡ Building the generator..."
cargo build --example create_large_cfb_v2

echo ""
echo "🔧 Starting large file generation (this will take a long time)..."
echo "📊 Target size: 20GB"
echo "📁 Output file: large_test_20gb.cfb"
echo ""

# Run with progress monitoring
(
    cargo run --example create_large_cfb_v2 -- 20 2>&1 | while IFS= read -r line; do
        echo "$(date '+%H:%M:%S') | $line"
        
        # Extract progress information if available
        if [[ "$line" == *"Progress:"* ]]; then
            echo "$line" >> create_20gb_progress.log
        fi
        
        # Log important milestones
        if [[ "$line" == *"Phase"* ]] || [[ "$line" == *"Successfully created"* ]]; then
            echo "$(date '+%Y-%m-%d %H:%M:%S') - $line" >> create_20gb_milestones.log
        fi
    done
) &

GENERATOR_PID=$!

echo "🔄 Generator process started with PID: $GENERATOR_PID"
echo "📋 Monitor progress with: tail -f create_20gb_progress.log"
echo "📈 View milestones with: tail -f create_20gb_milestones.log"
echo "⏹️  Stop generation with: kill $GENERATOR_PID"

# Function to handle cleanup on exit
cleanup() {
    if kill -0 $GENERATOR_PID 2>/dev/null; then
        echo ""
        echo "🛑 Stopping generator process..."
        kill $GENERATOR_PID 2>/dev/null || true
        wait $GENERATOR_PID 2>/dev/null || true
    fi
}

trap cleanup EXIT INT TERM

# Monitor the process
echo ""
echo "⏳ Monitoring generation process..."
echo "   Press Ctrl+C to stop and cleanup"

# Show periodic updates
while kill -0 $GENERATOR_PID 2>/dev/null; do
    sleep 30
    
    # Show current file size if it exists
    if [[ -f "large_test_20gb.cfb" ]]; then
        FILE_SIZE=$(du -h large_test_20gb.cfb 2>/dev/null | cut -f1 || echo "0")
        echo "📊 $(date '+%H:%M:%S') - Current file size: $FILE_SIZE"
    fi
    
    # Show disk space
    DISK_FREE=$(df -h . | tail -1 | awk '{print $4}')
    echo "💾 $(date '+%H:%M:%S') - Free disk space: $DISK_FREE"
    
    # Show latest progress if available
    if [[ -f "create_20gb_progress.log" ]]; then
        LATEST_PROGRESS=$(tail -1 create_20gb_progress.log 2>/dev/null || echo "No progress yet")
        echo "🔄 $(date '+%H:%M:%S') - Latest: $LATEST_PROGRESS"
    fi
    
    echo "---"
done

# Wait for completion
wait $GENERATOR_PID
EXIT_CODE=$?

echo ""
echo "🏁 Generation process completed!"
echo "⏰ End time: $(date)"

if [[ $EXIT_CODE -eq 0 ]]; then
    echo "✅ SUCCESS: 20GB CFB file created successfully!"
    
    if [[ -f "large_test_20gb.cfb" ]]; then
        FILE_SIZE=$(du -h large_test_20gb.cfb | cut -f1)
        FILE_SIZE_BYTES=$(stat -c%s large_test_20gb.cfb)
        echo "📁 File: large_test_20gb.cfb"
        echo "📊 Size: $FILE_SIZE ($FILE_SIZE_BYTES bytes)"
        
        echo ""
        echo "🔍 Quick structure preview:"
        cargo run --example cfbtool -- ls large_test_20gb.cfb: | head -10
        
        echo ""
        echo "📋 Test commands:"
        echo "   List all contents: cargo run --example cfbtool -- ls --all large_test_20gb.cfb:"
        echo "   View Documents:    cargo run --example cfbtool -- ls --long large_test_20gb.cfb:Documents"
        echo "   Read a file:       cargo run --example cfbtool -- cat large_test_20gb.cfb:Documents/Sub_001/[filename]"
    fi
else
    echo "❌ FAILED: Generation process failed with exit code $EXIT_CODE"
    echo "📋 Check the logs for details:"
    echo "   Progress: create_20gb_progress.log"
    echo "   Milestones: create_20gb_milestones.log"
fi

echo ""
echo "📄 Log files created:"
echo "   📊 Progress: create_20gb_progress.log"
echo "   🏆 Milestones: create_20gb_milestones.log"