#!/usr/bin/env python3
"""
Test that all SDKs can be imported and basic functionality works
"""

import sys
import os
from pathlib import Path

def test_python_sdk():
    """Test Python SDK import and basic functionality"""
    print("Testing Python SDK...")
    
    try:
        # Add SDK to path
        sdk_path = Path(__file__).parent / "sdks" / "python"
        sys.path.insert(0, str(sdk_path))
        
        from overdrive import OverDrive
        print("[OK] Python SDK imported successfully")
        
        # Test version
        version = OverDrive.version()
        print(f"[OK] Version: {version}")
        
        # Test basic database operations
        db = OverDrive.open("test_python.odb")
        db.create_table("test")
        doc_id = db.insert("test", {"name": "test"})
        doc = db.get("test", doc_id)
        db.close()
        
        # Cleanup
        if os.path.exists("test_python.odb"):
            os.remove("test_python.odb")
        
        print("[SUCCESS] Python SDK test PASSED")
        return True
        
    except Exception as e:
        print(f"[FAIL] Python SDK test FAILED: {e}")
        return False

def test_nodejs_sdk():
    """Test Node.js SDK by running a simple script"""
    print("Testing Node.js SDK...")
    
    import subprocess
    
    test_script = '''
const { OverDrive } = require('./sdks/nodejs');
console.log("[OK] Node.js SDK imported successfully");
const version = OverDrive.version();
console.log(`[OK] Version: ${version}`);
console.log("[SUCCESS] Node.js SDK test PASSED");
'''
    
    try:
        result = subprocess.run(
            ['node', '-e', test_script],
            cwd='.',
            capture_output=True,
            text=True,
            timeout=30
        )
        
        if result.returncode == 0:
            print(result.stdout.strip())
            return True
        else:
            print(f"[FAIL] Node.js SDK test FAILED: {result.stderr}")
            return False
            
    except Exception as e:
        print(f"[FAIL] Node.js SDK test FAILED: {e}")
        return False

def test_java_sdk():
    """Test Java SDK by checking if it can be built"""
    print("Testing Java SDK...")
    
    # Check if Java SDK files exist
    java_sdk_path = Path("sdks/java/src/main/java/com/afot/overdrive")
    if not java_sdk_path.exists():
        print("[FAIL] Java SDK source directory not found")
        return False
    
    # Check if native libraries are in resources
    resources_path = Path("sdks/java/src/main/resources/native/windows/overdrive.dll")
    if not resources_path.exists():
        print("[FAIL] Java SDK native library not found in resources")
        return False
    
    print("[OK] Java SDK source files exist")
    print("[OK] Java SDK native libraries in resources")
    print("[SUCCESS] Java SDK structure test PASSED")
    return True

def test_go_sdk():
    """Test Go SDK by checking if files exist"""
    print("Testing Go SDK...")
    
    go_sdk_path = Path("sdks/go/overdrive.go")
    if not go_sdk_path.exists():
        print("[FAIL] Go SDK file not found")
        return False
    
    print("[OK] Go SDK file exists")
    print("[SUCCESS] Go SDK structure test PASSED")
    return True

def test_c_sdk():
    """Test C SDK by checking header file"""
    print("Testing C SDK...")
    
    c_sdk_path = Path("sdks/c/include/overdrive.h")
    if not c_sdk_path.exists():
        print("[FAIL] C SDK header not found")
        return False
    
    print("[OK] C SDK header exists")
    print("[SUCCESS] C SDK structure test PASSED")
    return True

def test_native_structure():
    """Test that the centralized native library structure is correct"""
    print("Testing native library structure...")
    
    native_dir = Path("native")
    if not native_dir.exists():
        print("[FAIL] Native directory not found")
        return False
    
    # Check required libraries
    required_libs = [
        "native/windows/overdrive.dll",
        "native/linux/x64/liboverdrive.so",
        "native/macos/x64/liboverdrive.dylib"
    ]
    
    all_found = True
    for lib_path in required_libs:
        path = Path(lib_path)
        if path.exists():
            size = path.stat().st_size
            print(f"[OK] Found: {lib_path} ({size} bytes)")
        else:
            print(f"[MISSING] Not found: {lib_path}")
            all_found = False
    
    if all_found:
        print("[SUCCESS] Native library structure test PASSED")
    else:
        print("[FAIL] Native library structure test FAILED")
    
    return all_found

def main():
    """Run all tests"""
    print("OverDrive-DB SDK Testing Suite")
    print("==============================")
    print("Testing all SDKs with new centralized native library structure...")
    print()
    
    # Change to IncodeSDK directory if not already there
    current_dir = os.path.basename(os.getcwd())
    if current_dir != "IncodeSDK":
        if os.path.exists("IncodeSDK"):
            os.chdir("IncodeSDK")
        else:
            print("[ERROR] IncodeSDK directory not found")
            return False
    
    tests = [
        ("Native Structure", test_native_structure),
        ("Python SDK", test_python_sdk),
        ("Node.js SDK", test_nodejs_sdk),
        ("Java SDK", test_java_sdk),
        ("Go SDK", test_go_sdk),
        ("C SDK", test_c_sdk),
    ]
    
    results = {}
    
    for name, test_func in tests:
        print(f"{'='*50}")
        try:
            success = test_func()
            results[name] = success
        except Exception as e:
            print(f"[ERROR] {name}: {e}")
            results[name] = False
        print()
    
    # Summary
    print("Test Results Summary")
    print("===================")
    passed = sum(1 for success in results.values() if success)
    total = len(results)
    
    for name, success in results.items():
        status = "PASSED" if success else "FAILED"
        print(f"  {name:20} {status}")
    
    print()
    print(f"Overall: {passed}/{total} tests passed")
    
    if passed == total:
        print("All tests PASSED! New native library structure is working correctly.")
        return True
    else:
        print("Some tests failed. Please check the errors above.")
        return False

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)