#!/bin/bash

# CFB Large File Creation Summary & Monitor

echo "🚀 CFB LARGE FILE CREATION SUITE"
echo "=================================="
echo ""

cd "$(dirname "$0")"

echo "📊 CURRENT STATUS:"
echo "=================="

# Show current CFB files
echo "📁 Created CFB files:"
if ls *.cfb &>/dev/null; then
    ls -lah *.cfb | while read -r line; do
        echo "   $line"
    done
else
    echo "   No CFB files found yet"
fi

echo ""

# Check 20GB process status
if pgrep -f "create_large_cfb_v2.*20" >/dev/null; then
    echo "🔄 20GB File Creation: RUNNING"
    
    if [[ -f "create_20gb.log" ]]; then
        echo "📈 Latest progress:"
        tail -3 create_20gb.log | sed 's/^/   /'
    fi
    
    if [[ -f "large_test_20gb.cfb" ]]; then
        size=$(du -h large_test_20gb.cfb 2>/dev/null | cut -f1 || echo "0")
        echo "📊 Current file size: $size"
    fi
else
    echo "⏸️  20GB File Creation: NOT RUNNING"
fi

echo ""
echo "🔧 AVAILABLE TOOLS:"
echo "=================="
echo "1. ✅ create_large_cfb_v2.rs     - Direct Rust generator (fast, single process)"
echo "2. 🔄 create_20gb_cfb.sh         - Monitored 20GB creation with progress tracking"
echo "3. 📦 create_incremental_cfb.sh  - Incremental building using cfbtool"
echo "4. 🔍 cfbtool.rs                 - Command-line tool for CFB exploration"

echo ""
echo "🎯 USAGE EXAMPLES:"
echo "=================="

echo ""
echo "📋 Create different sized files:"
echo "   cargo run --example create_large_cfb_v2 -- 1    # 1GB file"
echo "   cargo run --example create_large_cfb_v2 -- 5    # 5GB file"
echo "   cargo run --example create_large_cfb_v2 -- 20   # 20GB file"

echo ""
echo "🔍 Explore created files:"
echo "   cargo run --example cfbtool -- ls --long [file].cfb:"
echo "   cargo run --example cfbtool -- ls [file].cfb:Documents"
echo "   cargo run --example cfbtool -- cat [file].cfb:path/to/stream"

echo ""
echo "🏗️ BUILD STRUCTURE EXAMPLE:"
echo "=========================="
echo "   📂 Root"
echo "   ├── 📁 Documents (Sub_000 to Sub_014)"
echo "   │   ├── 📄 Various files (data_ABC123.txt, report_XYZ789.pdf, etc.)"
echo "   │   └── 📁 Deep_00 to Deep_03 (nested folders)"
echo "   ├── 📁 Images, Data, Config, Archive..."
echo "   └── 📄 Random streams with different data patterns"

echo ""
echo "📊 FILE CONTENT PATTERNS:"
echo "========================"
echo "   🔸 Random binary data (25%)"
echo "   🔸 Text-like content (25%)"  
echo "   🔸 Structured XML/JSON patterns (25%)"
echo "   🔸 Log file patterns (25%)"

echo ""
echo "💾 DISK USAGE:"
df -h . | head -2

echo ""
echo "🎮 QUICK DEMO COMMANDS:"
echo "======================"

if [[ -f "large_test_1gb.cfb" ]]; then
    echo "📊 Explore the 1GB file we created:"
    echo "   cargo run --example cfbtool -- ls large_test_1gb.cfb:"
    echo "   cargo run --example cfbtool -- ls --long large_test_1gb.cfb:Documents"
    echo ""
    
    echo "🔍 Quick peek at 1GB file structure:"
    cargo run --example cfbtool -- ls large_test_1gb.cfb: 2>/dev/null | head -5 | sed 's/^/   /'
fi

echo ""
echo "🚀 START NEW CREATION:"
echo "======================"
echo "   ./create_20gb_cfb.sh              # Monitored 20GB creation"
echo "   ./create_incremental_cfb.sh 5     # 5GB incremental creation"

echo ""
echo "📈 MONITOR PROGRESS:"
echo "==================="
echo "   tail -f create_20gb.log           # Watch 20GB creation log"
echo "   watch 'ls -lah *.cfb'             # Watch file sizes grow"
echo "   ps aux | grep create_large        # Check running processes"

if [[ -f "create_20gb.log" ]]; then
    echo ""
    echo "📋 Current 20GB progress:"
    echo "========================="
    tail -1 create_20gb.log 2>/dev/null | sed 's/^/   /' || echo "   No progress yet"
fi