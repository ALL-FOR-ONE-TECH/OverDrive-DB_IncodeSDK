#!/usr/bin/env python3
"""
Test what functions are available in the DLL
"""

import ctypes
import os
from pathlib import Path

def test_dll_functions():
    """Test what functions are available in our DLL"""
    dll_path = Path("IncodeSDK/native/windows/overdrive.dll")
    
    if not dll_path.exists():
        print(f"DLL not found at {dll_path}")
        return
    
    print(f"Testing DLL at: {dll_path}")
    print(f"File size: {dll_path.stat().st_size} bytes")
    
    try:
        # Load the DLL
        lib = ctypes.cdll.LoadLibrary(str(dll_path))
        print("DLL loaded successfully")
        
        # Test common function names
        functions_to_test = [
            "overdrive_open",
            "overdrive_close", 
            "overdrive_version",
            "overdrive_create_table",
            "overdrive_insert",
            "overdrive_query",
            # Alternative names
            "odb_open",
            "odb_version",
            # Check if it's a different naming convention
            "overdriveOpen",
            "overdriveVersion",
        ]
        
        found_functions = []
        for func_name in functions_to_test:
            try:
                func = getattr(lib, func_name)
                found_functions.append(func_name)
                print(f"[OK] Found function: {func_name}")
            except AttributeError:
                print(f"[MISSING] Function not found: {func_name}")
        
        if found_functions:
            print(f"\nFound {len(found_functions)} functions:")
            for func in found_functions:
                print(f"  - {func}")
        else:
            print("\nNo expected functions found in DLL")
            
        # Try to call version if available
        if "overdrive_version" in found_functions:
            try:
                version_func = lib.overdrive_version
                version_func.restype = ctypes.c_char_p
                version = version_func()
                if version:
                    print(f"\nVersion: {version.decode('utf-8')}")
                else:
                    print("\nVersion function returned NULL")
            except Exception as e:
                print(f"\nError calling version function: {e}")
                
    except Exception as e:
        print(f"Error loading DLL: {e}")

if __name__ == "__main__":
    test_dll_functions()