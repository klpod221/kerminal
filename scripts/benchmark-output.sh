#!/bin/bash
# Kerminal Terminal Output Benchmark
# This script pumps a large amount of data to test the output buffer coalescing.

echo "================================================="
echo "Kerminal Output Benchmark"
echo "================================================="
echo "This test will generate ~200MB of text output."
echo "Before starting, open DevTools (Ctrl+Shift+I) and note your terminal ID."
echo "You can check metrics by running in console:"
echo "  await window.getMetrics_YOUR_TERMINAL_ID()"
echo "================================================="
echo "Press Enter to start..."
read

echo "Running 'yes | head -c 200M'..."
time yes "benchmark data line to test terminal coalescing and rendering speed, utf8 testing: 🚀 🍎 🍌" | head -c 200M

echo ""
echo "Done! Run the getMetrics function in DevTools to see how many chunks were coalesced and dropped."
