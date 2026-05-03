// Simple Node.js SDK test
const path = require('path');
const fs = require('fs');

try {
    // Load the SDK
    const { OverDrive } = require('./sdks/nodejs');
    
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