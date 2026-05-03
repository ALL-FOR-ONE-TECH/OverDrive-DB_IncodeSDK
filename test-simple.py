#!/usr/bin/env python3
"""
Simple OverDrive-DB SDK Test
Tests Python SDK with new native library structure
"""

import sys
import os
from pathlib import Path

# Add SDK to path
sdk_path = Path(__file__).parent / "sdks" / "python"
sys.path.insert(0, str(sdk_path))

def test_python_sdk():
    """Test Python SDK"""
    print("Testing Python SDK...")
    
    try:
        from overdrive import OverDrive
        print("[OK] Python SDK imported successfully")
        
        # Test version
        version = OverDrive.version()
        print(f"[OK] Version: {version}")
        
        # Test database operations
        db = OverDrive.open("test_simple.odb")
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
        if os.path.exists("test_simple.odb"):
            os.remove("test_simple.odb")
        
        print("[SUCCESS] Python SDK test PASSED")
        return True
        
    except Exception as e:
        print(f"[FAIL] Python SDK test FAILED: {e}")
        import traceback
        traceback.print_exc()
        return False

if __name__ == "__main__":
    success = test_python_sdk()
    sys.exit(0 if success else 1)