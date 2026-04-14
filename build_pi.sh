#!/bin/bash
# Build script for AMOR on Raspberry Pi
# Run this on the Pi: ./build_pi.sh

set -e

echo "=== Building AMOR on Pi ==="

cd /home/fuckall/code/amor/lemmelearn

# Pull latest if git exists
if [ -d .git ]; then
    echo "Pulling latest..."
    git pull origin master
fi

# Copy any updated source files
echo "Copying source from laptop..."
scp skedaddle@$(hostname -I | awk '{print $1}'):/home/skedaddle/code/amor/lemmelearn/src/main.rs /home/fuckall/code/amor/lemmelearn/src/ 2>/dev/null || true
scp skedaddle@$(hostname -I | awk '{print $1}'):/home/skedaddle/code/amor/lemmelearn/src/config.rs /home/fuckall/code/amor/lemmelearn/src/ 2>/dev/null || true
scp skedaddle@$(hostname -I | awk '{print $1}'):/home/skedaddle/code/amor/lemmelearn/src/tool/*.rs /home/fuckall/code/amor/lemmelearn/src/tool/ 2>/dev/null || true

# Build
echo "Building..."
cargo build --release

# Replace binary
echo "Replacing binary..."
cp target/release/lemmelearn /home/fuckall/amor_new
cp /home/fuckall/amor_new /home/fuckall/amor 2>/dev/null || sudo cp /home/fuckall/amor_new /home/fuckall/amor

echo "=== Done ==="
echo "Run: ./amor"