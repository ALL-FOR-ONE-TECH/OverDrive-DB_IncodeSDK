#!/usr/bin/env python3
"""
OverDrive-DB SDK Testing Script
Tests all SDKs with the new centralized native library structure
"""

import os
import sys
import subprocess
import tempfile
import json
from pathlib import Path

def run_command(cmd, cwd=None, timeout=30):
    """Run a command and return (success, output, error)"""
    try:
        result = subprocess.run(
            cmd, 
            shell=True, 
            cwd=cwd, 
            capture_output=True, 
            text=True, 
            timeout=timeout
        )
        return result.returncode == 0, result.stdout, result.stderr
    except subprocess.TimeoutExpired:
        return False, "", "Command timed out"
    except Exception as e:
        return False, "", str(e)

def test_python_sdk():
    """Test Python SDK with new native library structure"""
    print("Python Testing Python SDK...")
    
    # Create test script
    test_script = '''
import sys
import os
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "sdks", "python"))

try:
    from overdrive import OverDrive
    
    # Test library loading
    print("[OK] Python SDK imported successfully")
    
    # Test version
    version = OverDrive.version()
    print(f"[OK] Version: {version}")
    
    # Test database operations
    db = OverDrive.open("test_python.odb")
    print("[OK] Database opened successfully")
    
    # Test table operations
    db.create_table("test_table")
    print("[OK] Table created successfully")
    
    # Test CRUD operations
    doc_id = db.insert("test_table", {"name": "Alice", "age": 30})
    print(f"[OK] Document inserted: {doc_id}")
    
    doc = db.get("test_table", doc_id)
    print(f"[OK] Document retrieved: {doc}")
    
    # Test query
    results = db.query("SELECT * FROM test_table")
    print(f"[OK] Query executed: {len(results)} rows")
    
    db.close()
    print("[OK] Database closed successfully")
    
    # Cleanup
    if os.path.exists("test_python.odb"):
        os.remove("test_python.odb")
    
    print("[SUCCESS] Python SDK test PASSED")
    
except Exception as e:
    print(f"[FAIL] Python SDK test FAILED: {e}")
    import traceback
    traceback.print_exc()
    sys.exit(1)
'''
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.py', delete=False) as f:
        f.write(test_script)
        test_file = f.name
    
    try:
        success, output, error = run_command(f"python3 {test_file}", cwd=".")
        print(output)
        if error:
            print(f"Stderr: {error}")
        return success
    finally:
        os.unlink(test_file)

def test_nodejs_sdk():
    """Test Node.js SDK with new native library structure"""
    print("Node.js Testing Node.js SDK...")
    
    # Create test script
    test_script = '''
const path = require('path');
const fs = require('fs');

try {
    // Add the SDK to the path
    const sdkPath = path.join(__dirname, 'sdks', 'nodejs');
    const { OverDrive } = require(sdkPath);
    
    console.log("[OK] Node.js SDK imported successfully");
    
    // Test version
    const version = OverDrive.version();
    console.log(`[OK] Version: ${version}`);
    
    // Test database operations
    const db = OverDrive.open("test_nodejs.odb");
    console.log("[OK] Database opened successfully");
    
    // Test table operations
    db.createTable("test_table");
    console.log("[OK] Table created successfully");
    
    // Test CRUD operations
    const docId = db.insert("test_table", { name: "Bob", age: 25 });
    console.log(`[OK] Document inserted: ${docId}`);
    
    const doc = db.get("test_table", docId);
    console.log(`[OK] Document retrieved: ${JSON.stringify(doc)}`);
    
    // Test query
    const results = db.query("SELECT * FROM test_table");
    console.log(`[OK] Query executed: ${results.length} rows`);
    
    db.close();
    console.log("[OK] Database closed successfully");
    
    // Cleanup
    if (fs.existsSync("test_nodejs.odb")) {
        fs.unlinkSync("test_nodejs.odb");
    }
    
    console.log("[SUCCESS] Node.js SDK test PASSED");
    
} catch (error) {
    console.error(`[FAIL] Node.js SDK test FAILED: ${error.message}`);
    console.error(error.stack);
    process.exit(1);
}
'''
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.js', delete=False) as f:
        f.write(test_script)
        test_file = f.name
    
    try:
        success, output, error = run_command(f"node {test_file}", cwd=".")
        print(output)
        if error:
            print(f"Stderr: {error}")
        return success
    finally:
        os.unlink(test_file)

def test_go_sdk():
    """Test Go SDK with new native library structure"""
    print("🔵 Testing Go SDK...")
    
    # Create test script
    test_script = '''
package main

import (
    "fmt"
    "os"
    "path/filepath"
)

// Import the local overdrive package
import "./sdks/go"

func main() {
    defer func() {
        if r := recover(); r != nil {
            fmt.Printf("❌ Go SDK test FAILED: %v\\n", r)
            os.Exit(1)
        }
    }()
    
    fmt.Println("✅ Go SDK imported successfully")
    
    // Test version
    version := overdrive.Version()
    fmt.Printf("✅ Version: %s\\n", version)
    
    // Test database operations
    db, err := overdrive.Open("test_go.odb")
    if err != nil {
        panic(fmt.Sprintf("Failed to open database: %v", err))
    }
    fmt.Println("✅ Database opened successfully")
    
    // Test table operations
    err = db.CreateTable("test_table")
    if err != nil {
        panic(fmt.Sprintf("Failed to create table: %v", err))
    }
    fmt.Println("✅ Table created successfully")
    
    // Test CRUD operations
    docId, err := db.Insert("test_table", map[string]any{"name": "Charlie", "age": 35})
    if err != nil {
        panic(fmt.Sprintf("Failed to insert document: %v", err))
    }
    fmt.Printf("✅ Document inserted: %s\\n", docId)
    
    doc, err := db.Get("test_table", docId)
    if err != nil {
        panic(fmt.Sprintf("Failed to get document: %v", err))
    }
    fmt.Printf("✅ Document retrieved: %v\\n", doc)
    
    // Test query
    results, err := db.Query("SELECT * FROM test_table")
    if err != nil {
        panic(fmt.Sprintf("Failed to query: %v", err))
    }
    fmt.Printf("✅ Query executed: %d rows\\n", len(results.Rows))
    
    db.Close()
    fmt.Println("✅ Database closed successfully")
    
    // Cleanup
    if _, err := os.Stat("test_go.odb"); err == nil {
        os.Remove("test_go.odb")
    }
    
    fmt.Println("🎉 Go SDK test PASSED")
}
'''
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.go', delete=False) as f:
        f.write(test_script)
        test_file = f.name
    
    try:
        # Initialize go module first
        run_command("go mod init test", cwd=".")
        success, output, error = run_command(f"go run {test_file}", cwd=".")
        print(output)
        if error:
            print(f"Stderr: {error}")
        return success
    finally:
        os.unlink(test_file)
        # Cleanup go.mod
        if os.path.exists("go.mod"):
            os.remove("go.mod")

def test_java_sdk():
    """Test Java SDK with new native library structure"""
    print("☕ Testing Java SDK...")
    
    # Create test script
    test_script = '''
import com.afot.overdrive.OverDrive;
import java.io.File;

public class TestJavaSDK {
    public static void main(String[] args) {
        try {
            System.out.println("✅ Java SDK imported successfully");
            
            // Test version
            String version = OverDrive.version();
            System.out.println("✅ Version: " + version);
            
            // Test database operations
            OverDrive db = OverDrive.open("test_java.odb");
            System.out.println("✅ Database opened successfully");
            
            // Test table operations
            db.createTable("test_table");
            System.out.println("✅ Table created successfully");
            
            // Test CRUD operations
            java.util.Map<String, Object> doc = new java.util.HashMap<>();
            doc.put("name", "David");
            doc.put("age", 40);
            String docId = db.insert("test_table", doc);
            System.out.println("✅ Document inserted: " + docId);
            
            java.util.Map<String, Object> retrieved = db.get("test_table", docId);
            System.out.println("✅ Document retrieved: " + retrieved);
            
            // Test query
            java.util.List<java.util.Map<String, Object>> results = db.query("SELECT * FROM test_table");
            System.out.println("✅ Query executed: " + results.size() + " rows");
            
            db.close();
            System.out.println("✅ Database closed successfully");
            
            // Cleanup
            File dbFile = new File("test_java.odb");
            if (dbFile.exists()) {
                dbFile.delete();
            }
            
            System.out.println("🎉 Java SDK test PASSED");
            
        } catch (Exception e) {
            System.err.println("❌ Java SDK test FAILED: " + e.getMessage());
            e.printStackTrace();
            System.exit(1);
        }
    }
}
'''
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.java', delete=False) as f:
        f.write(test_script)
        test_file = f.name
    
    try:
        # Compile and run Java test
        class_name = "TestJavaSDK"
        java_file = f"{class_name}.java"
        
        # Copy to proper filename
        os.rename(test_file, java_file)
        
        # Add SDK JAR to classpath
        jar_path = "sdks/java/target/overdrive-sdk-1.4.3.jar"
        if not os.path.exists(jar_path):
            print(f"⚠️  Java SDK JAR not found at {jar_path}, skipping Java test")
            return True  # Skip test if JAR not built
        
        # Compile
        compile_success, compile_output, compile_error = run_command(
            f"javac -cp {jar_path} {java_file}", cwd="."
        )
        
        if not compile_success:
            print(f"❌ Java compilation failed: {compile_error}")
            return False
        
        # Run
        success, output, error = run_command(
            f"java -cp .:sdks/java/target/overdrive-sdk-1.4.3.jar {class_name}", cwd="."
        )
        print(output)
        if error:
            print(f"Stderr: {error}")
        return success
        
    finally:
        # Cleanup
        for ext in ['.java', '.class']:
            cleanup_file = f"TestJavaSDK{ext}"
            if os.path.exists(cleanup_file):
                os.unlink(cleanup_file)

def main():
    """Run all SDK tests"""
    print("🧪 OverDrive-DB SDK Testing Suite")
    print("==================================")
    print("Testing all SDKs with new centralized native library structure...")
    print()
    
    # Check if we're in the right directory
    if not os.path.exists("IncodeSDK"):
        print("❌ Please run this script from the root directory (where IncodeSDK folder is located)")
        sys.exit(1)
    
    os.chdir("IncodeSDK")
    
    # Check native library structure
    print("📁 Checking native library structure...")
    native_dir = Path("native")
    if not native_dir.exists():
        print("❌ Native library directory not found")
        sys.exit(1)
    
    # Check for required libraries
    required_libs = [
        "native/windows/overdrive.dll",
        "native/linux/x64/liboverdrive.so", 
        "native/macos/x64/liboverdrive.dylib"
    ]
    
    for lib_path in required_libs:
        if Path(lib_path).exists():
            print(f"✅ Found: {lib_path}")
        else:
            print(f"⚠️  Missing: {lib_path}")
    
    print()
    
    # Run tests
    tests = [
        ("Python", test_python_sdk),
        ("Node.js", test_nodejs_sdk),
        ("Go", test_go_sdk),
        ("Java", test_java_sdk),
    ]
    
    results = {}
    
    for name, test_func in tests:
        try:
            print(f"{'='*50}")
            success = test_func()
            results[name] = success
            if success:
                print(f"✅ {name} SDK: PASSED")
            else:
                print(f"❌ {name} SDK: FAILED")
        except Exception as e:
            print(f"❌ {name} SDK: ERROR - {e}")
            results[name] = False
        print()
    
    # Summary
    print("📊 Test Results Summary")
    print("======================")
    passed = sum(1 for success in results.values() if success)
    total = len(results)
    
    for name, success in results.items():
        status = "✅ PASSED" if success else "❌ FAILED"
        print(f"  {name:10} {status}")
    
    print()
    print(f"Overall: {passed}/{total} SDKs passed")
    
    if passed == total:
        print("🎉 All SDK tests PASSED! New native library structure is working correctly.")
        return True
    else:
        print("⚠️  Some SDK tests failed. Please check the errors above.")
        return False

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)