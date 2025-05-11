#!/bin/sh
cargo objcopy --release -- -O ihex ledragon.hex
echo "connect teensy and enable bootloader mode..."
teensy_loader_cli --mcu=TEENSY41 -w ledragon.hex
echo "done!"
