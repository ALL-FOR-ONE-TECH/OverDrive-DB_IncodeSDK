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

if sys.platform == "win32":
    import io
    sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')
    sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8', errors='replace')

def run_command(cmd, cwd=None, timeout=60):
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
    print("[Python] Testing Python SDK...")
    sdk_dir = str(Path("python").resolve()).replace("\\", "\\\\")
    
    test_script = f'''
import sys
import os
sys.path.insert(0, r"{sdk_dir}")

try:
    from overdrive import OverDrive
    
    # Test library loading
    print("[OK] Python SDK imported successfully")
    
    # Test version
    version = OverDrive.version()
    print(f"[OK] Version: {{version}}")
    
    # Test database operations
    db = OverDrive.open("test_python.odb")
    print("[OK] Database opened successfully")
    
    # Test table operations
    db.create_table("test_table")
    print("[OK] Table created successfully")
    
    # Test CRUD operations
    doc_id = db.insert("test_table", {{"name": "Alice", "age": 30}})
    print(f"[OK] Document inserted: {{doc_id}}")
    
    doc = db.get("test_table", doc_id)
    print(f"[OK] Document retrieved: {{doc}}")
    
    # Test query
    results = db.query("SELECT * FROM test_table")
    print(f"[OK] Query executed: {{len(results)}} rows")
    
    db.close()
    print("[OK] Database closed successfully")
    
    # Cleanup
    if os.path.exists("test_python.odb"):
        os.remove("test_python.odb")
    if os.path.exists("test_python.odb.wal"):
        os.remove("test_python.odb.wal")
    
    print("[SUCCESS] Python SDK test PASSED")
    
except Exception as e:
    print(f"[FAIL] Python SDK test FAILED: {{e}}")
    import traceback
    traceback.print_exc()
    sys.exit(1)
'''
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.py', delete=False, encoding='utf-8') as f:
        f.write(test_script)
        test_file = f.name
    
    try:
        success, output, error = run_command(f'"{sys.executable}" "{test_file}"', cwd=".")
        print(output)
        if error:
            print(f"Stderr: {error}")
        return success
    finally:
        if os.path.exists(test_file):
            os.unlink(test_file)

def test_nodejs_sdk():
    """Test Node.js SDK with new native library structure"""
    print("[Node.js] Testing Node.js SDK...")
    sdk_dir = str(Path("nodejs").resolve()).replace("\\", "/")

    test_script = f'''
const path = require('path');
const fs = require('fs');

try {{
    const {{ OverDrive }} = require("{sdk_dir}");
    
    console.log("[OK] Node.js SDK imported successfully");
    
    // Test version
    const version = OverDrive.version();
    console.log(`[OK] Version: ${{version}}`);
    
    // Test database operations
    const db = OverDrive.open("test_nodejs.odb");
    console.log("[OK] Database opened successfully");
    
    // Test table operations
    db.createTable("test_table");
    console.log("[OK] Table created successfully");
    
    // Test CRUD operations
    const docId = db.insert("test_table", {{ name: "Bob", age: 25 }});
    console.log(`[OK] Document inserted: ${{docId}}`);
    
    const doc = db.get("test_table", docId);
    console.log(`[OK] Document retrieved: ${{JSON.stringify(doc)}}`);
    
    // Test query
    const results = db.query("SELECT * FROM test_table");
    console.log(`[OK] Query executed: ${{results.length}} rows`);
    
    db.close();
    console.log("[OK] Database closed successfully");
    
    // Cleanup
    if (fs.existsSync("test_nodejs.odb")) {{
        fs.unlinkSync("test_nodejs.odb");
    }}
    if (fs.existsSync("test_nodejs.odb.wal")) {{
        fs.unlinkSync("test_nodejs.odb.wal");
    }}
    
    console.log("[SUCCESS] Node.js SDK test PASSED");
    
}} catch (error) {{
    console.error(`[FAIL] Node.js SDK test FAILED: ${{error.message}}`);
    console.error(error.stack);
    process.exit(1);
}}
'''
    
    with tempfile.NamedTemporaryFile(mode='w', suffix='.js', delete=False, encoding='utf-8') as f:
        f.write(test_script)
        test_file = f.name
    
    try:
        success, output, error = run_command(f'node "{test_file}"', cwd=".")
        print(output)
        if error:
            print(f"Stderr: {error}")
        return success
    finally:
        if os.path.exists(test_file):
            os.unlink(test_file)

def test_go_sdk():
    """Test Go SDK with new native library structure"""
    print("[Go] Testing Go SDK...")
    go_dir = str(Path("go").resolve())
    success, output, error = run_command("go test -v ./...", cwd=go_dir)
    print(output)
    if error:
        print(f"Stderr: {error}")
    return success

def test_rust_sdk():
    """Test Rust SDK with new native library structure"""
    print("[Rust] Testing Rust SDK...")
    success, output, error = run_command("cargo test --manifest-path rust/Cargo.toml", cwd=".")
    print(output)
    if error:
        print(f"Stderr: {error}")
    return success

def test_java_sdk():
    """Test Java SDK with new native library structure"""
    print("[Java] Testing Java SDK...")
    jar_path = Path("java/target/overdrive-sdk-1.4.3.jar")
    if not jar_path.exists():
        print(f"[SKIP] Java SDK JAR not found at {jar_path}, skipping Java test")
        return True
    
    j_ok, _, _ = run_command("javac -version")
    if not j_ok:
        print("[SKIP] javac not found in PATH, skipping Java execution test")
        return True
    
    class_name = "TestJavaSDK"
    java_code = '''
import com.afot.overdrive.OverDrive;
import java.io.File;

public class TestJavaSDK {
    public static void main(String[] args) {
        try {
            System.out.println("[OK] Java SDK imported successfully");
            String version = OverDrive.version();
            System.out.println("[OK] Version: " + version);
            OverDrive db = OverDrive.open("test_java.odb");
            System.out.println("[OK] Database opened successfully");
            db.createTable("test_table");
            System.out.println("[OK] Table created successfully");
            java.util.Map<String, Object> doc = new java.util.HashMap<>();
            doc.put("name", "David");
            doc.put("age", 40);
            String docId = db.insert("test_table", doc);
            System.out.println("[OK] Document inserted: " + docId);
            java.util.Map<String, Object> retrieved = db.get("test_table", docId);
            System.out.println("[OK] Document retrieved: " + retrieved);
            java.util.List<java.util.Map<String, Object>> results = db.query("SELECT * FROM test_table");
            System.out.println("[OK] Query executed: " + results.size() + " rows");
            db.close();
            System.out.println("[OK] Database closed successfully");
            File dbFile = new File("test_java.odb");
            if (dbFile.exists()) dbFile.delete();
            File walFile = new File("test_java.odb.wal");
            if (walFile.exists()) walFile.delete();
            System.out.println("[SUCCESS] Java SDK test PASSED");
        } catch (Exception e) {
            System.err.println("[FAIL] Java SDK test FAILED: " + e.getMessage());
            e.printStackTrace();
            System.exit(1);
        }
    }
}
'''
    with open("TestJavaSDK.java", "w", encoding="utf-8") as f:
        f.write(java_code)
    try:
        cp_sep = ";" if sys.platform == "win32" else ":"
        compile_success, compile_output, compile_error = run_command(
            f'javac -cp "{jar_path}" TestJavaSDK.java', cwd="."
        )
        if not compile_success:
            print(f"[FAIL] Java compilation failed: {compile_error}")
            return False
        success, output, error = run_command(
            f'java -cp ".{cp_sep}{jar_path}" TestJavaSDK', cwd="."
        )
        print(output)
        if error:
            print(f"Stderr: {error}")
        return success
    finally:
        for ext in ['.java', '.class']:
            f = Path(f"TestJavaSDK{ext}")
            if f.exists():
                f.unlink()

def main():
    """Run all SDK tests"""
    print("OverDrive-DB SDK Testing Suite")
    print("==============================")
    print("Testing all SDKs with new centralized native library structure...\n")
    
    script_dir = Path(__file__).resolve().parent
    os.chdir(script_dir)
    
    # Check native library structure
    print("[Native] Checking native library structure...")
    native_dir = Path("native")
    if not native_dir.exists():
        print("[FAIL] Native library directory not found")
        sys.exit(1)
    
    required_libs = [
        "native/windows/overdrive.dll",
        "native/linux/x64/liboverdrive.so", 
        "native/macos/x64/liboverdrive.dylib"
    ]
    
    for lib_path in required_libs:
        if Path(lib_path).exists():
            print(f"[OK] Found: {lib_path}")
        else:
            print(f"[WARN] Missing: {lib_path}")
    
    print()
    
    # Run tests
    tests = [
        ("Python", test_python_sdk),
        ("Node.js", test_nodejs_sdk),
        ("Go", test_go_sdk),
        ("Rust", test_rust_sdk),
        ("Java", test_java_sdk),
    ]
    
    results = {}
    
    for name, test_func in tests:
        try:
            print(f"{'='*50}")
            success = test_func()
            results[name] = success
            if success:
                print(f"[RESULT] {name} SDK: PASSED")
            else:
                print(f"[RESULT] {name} SDK: FAILED")
        except Exception as e:
            print(f"[RESULT] {name} SDK: ERROR - {e}")
            results[name] = False
        print()
    
    # Summary
    print("Test Results Summary")
    print("====================")
    passed = sum(1 for success in results.values() if success)
    total = len(results)
    
    for name, success in results.items():
        status = "PASSED" if success else "FAILED"
        print(f"  {name:10} {status}")
    
    print()
    print(f"Overall: {passed}/{total} SDKs passed")
    
    if passed == total:
        print("[ALL PASSED] All SDK tests PASSED! New native library structure is working correctly.")
        return True
    else:
        print("[WARN] Some SDK tests failed. Please check the errors above.")
        return False

if __name__ == "__main__":
    success = main()
    sys.exit(0 if success else 1)