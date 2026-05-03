// Test Node.js SDK path resolution from within the SDK directory
const path = require('path');
const fs = require('fs');

// Change to the SDK directory to simulate running from there
process.chdir('IncodeSDK/sdks/nodejs');

// Now require the SDK
const { OverDrive } = require('./index.js');

console.log("[OK] Node.js SDK loaded successfully");
console.log(`[OK] Version: ${OverDrive.version()}`);

// Test basic functionality
const db = OverDrive.open("test.odb");
console.log("[OK] Database opened");
db.close();
console.log("[OK] Database closed");

// Cleanup
if (fs.existsSync("test.odb")) {
    fs.unlinkSync("test.odb");
}

console.log("[SUCCESS] Node.js SDK test PASSED");