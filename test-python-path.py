#!/usr/bin/env python3
"""
Test Python SDK library path resolution
"""

import sys
import os
import platform
from pathlib import Path

# Add SDK to path
sdk_path = Path(__file__).parent / "sdks" / "python"
sys.path.insert(0, str(sdk_path))

def test_library_path():
    """Test the library path resolution"""
    print("Testing Python SDK library path resolution...")
    
    # Simulate the _find_library function logic
    system = platform.system()
    machine = platform.machine().lower()
    
    print(f"System: {system}")
    print(f"Machine: {machine}")
    
    # Determine library name and platform-specific path
    if system == "Windows":
        lib_name = "overdrive.dll"
        platform_dir = "windows"
    elif system == "Darwin":
        lib_name = "liboverdrive.dylib"
        if machine in ["arm64", "aarch64"]:
            platform_dir = "macos/arm64"
        else:
            platform_dir = "macos/x64"
    else:  # Linux and others
        lib_name = "liboverdrive.so"
        if machine in ["arm64", "aarch64"]:
            platform_dir = "linux/arm64"
        else:
            platform_dir = "linux/x64"

    print(f"Library name: {lib_name}")
    print(f"Platform dir: {platform_dir}")
    
    # Test the search paths from the Python SDK
    # The __file__ in the SDK would be: IncodeSDK/sdks/python/overdrive/__init__.py
    # So Path(__file__).parent.parent.parent would be: IncodeSDK/sdks/python/../../ = IncodeSDK/
    
    # Simulate being in the overdrive package
    simulated_file = Path("IncodeSDK/sdks/python/overdrive/__init__.py")
    
    search_paths = [
        # 1. NEW STRUCTURE: Centralized native libraries
        simulated_file.parent.parent.parent.parent / "native" / platform_dir / lib_name,
        simulated_file.parent.parent.parent.parent / "native" / "windows" / lib_name,  # Windows fallback
        
        # 2. BACKWARD COMPATIBILITY: Old locations (for transition period)
        simulated_file.parent / lib_name,
        simulated_file.parent / "lib" / lib_name,
        simulated_file.parent.parent / "lib" / lib_name,
        simulated_file.parent.parent / "target" / "release" / lib_name,
        simulated_file.parent.parent.parent.parent / "target" / "release" / lib_name,
    ]
    
    print("\nSearch paths:")
    for i, path in enumerate(search_paths, 1):
        abs_path = path.resolve()
        exists = abs_path.exists()
        size = abs_path.stat().st_size if exists else 0
        status = f"EXISTS ({size} bytes)" if exists else "NOT FOUND"
        print(f"  {i}. {abs_path} - {status}")
        
        if exists and size > 100_000:
            print(f"     -> This would be selected!")
            return str(abs_path)
    
    print("\nNo suitable library found in search paths")
    return None

if __name__ == "__main__":
    found_path = test_library_path()