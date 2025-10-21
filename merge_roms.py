#!/usr/bin/env python3
"""
ROM Merger Utility for Arcade Games

This script merges multiple ROM files into a single binary file
that can be loaded by the emulator.
"""

import sys
import os
from pathlib import Path

# ROM layouts for different games
ROM_LAYOUTS = {
    'space_invaders': [
        # Space Invaders ROM layout
        'invaders.h',
        'invaders.g',
        'invaders.f',
        'invaders.e',
    ],
}

def merge_roms(rom_dir, game_name, output_file):
    """Merge ROM files according to the game's layout."""
    
    if game_name not in ROM_LAYOUTS:
        print(f"Error: Unknown game '{game_name}'")
        print(f"Supported games: {', '.join(ROM_LAYOUTS.keys())}")
        return False
    
    layout = ROM_LAYOUTS[game_name]
    rom_path = Path(rom_dir)
    
    # Check if all required files exist
    missing_files = []
    for filename in layout:
        file_path = rom_path / filename
        if not file_path.exists():
            missing_files.append(filename)
    
    if missing_files:
        print(f"Error: Missing ROM files:")
        for f in missing_files:
            print(f"  - {f}")
        print(f"\nAvailable files in {rom_dir}:")
        for f in sorted(rom_path.glob('*')):
            if f.is_file():
                print(f"  - {f.name}")
        return False
    
    # Merge the files
    print(f"Merging {game_name} ROM files...")
    with open(output_file, 'wb') as out:
        for filename in layout:
            file_path = rom_path / filename
            print(f"  Adding {filename} ({file_path.stat().st_size} bytes)")
            with open(file_path, 'rb') as f:
                out.write(f.read())
    
    total_size = Path(output_file).stat().st_size
    print(f"\nSuccess! Created {output_file} ({total_size} bytes / 0x{total_size:04X})")
    return True

def auto_detect_game(rom_dir):
    """Try to auto-detect which game based on available files."""
    rom_path = Path(rom_dir)
    available_files = set(f.name for f in rom_path.glob('*') if f.is_file())
    
    for game_name, layout in ROM_LAYOUTS.items():
        if all(f in available_files for f in layout):
            return game_name
    
    return None

def main():
    if len(sys.argv) < 3:
        print("ROM Merger Utility")
        print("\nUsage:")
        print("  python3 merge_roms.py <rom_directory> <game_name> [output_file]")
        print("  python3 merge_roms.py <rom_directory> auto [output_file]")
        print("\nSupported games:")
        for game in ROM_LAYOUTS.keys():
            print(f"  - {game}")
        print("\nExample:")
        print("  python3 merge_roms.py ./space_invaders_roms space_invaders space_invaders.rom")
        print("  python3 merge_roms.py ./space_invaders_roms auto")
        sys.exit(1)
    
    rom_dir = sys.argv[1]
    game_name = sys.argv[2]
    
    if not os.path.isdir(rom_dir):
        print(f"Error: Directory '{rom_dir}' not found")
        sys.exit(1)
    
    # Auto-detect game if requested
    if game_name == 'auto':
        detected_game = auto_detect_game(rom_dir)
        if detected_game:
            print(f"Auto-detected game: {detected_game}")
            game_name = detected_game
        else:
            print("Error: Could not auto-detect game from available files")
            print("\nAvailable files:")
            for f in sorted(Path(rom_dir).glob('*')):
                if f.is_file():
                    print(f"  - {f.name}")
            sys.exit(1)
    
    # Determine output filename
    if len(sys.argv) >= 4:
        output_file = sys.argv[3]
    else:
        output_file = f"{game_name}.rom"
    
    success = merge_roms(rom_dir, game_name, output_file)
    sys.exit(0 if success else 1)

if __name__ == '__main__':
    main()
