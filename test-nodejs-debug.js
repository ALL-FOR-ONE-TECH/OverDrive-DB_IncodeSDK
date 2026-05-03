// Debug Node.js SDK library loading
const path = require('path');
const fs = require('fs');
const os = require('os');

// Simulate the findLibrary function
function findLibrary() {
    const platform = os.platform();
    const arch = os.arch();
    let libName;
    let platformDir;
    
    // Determine library name and platform-specific path
    if (platform === 'win32') {
        libName = 'overdrive.dll';
        platformDir = 'windows';
    } else if (platform === 'darwin') {
        libName = 'liboverdrive.dylib';
        if (arch === 'arm64') {
            platformDir = 'macos/arm64';
        } else {
            platformDir = 'macos/x64';
        }
    } else {
        libName = 'liboverdrive.so';
        if (arch === 'arm64') {
            platformDir = 'linux/arm64';
        } else {
            platformDir = 'linux/x64';
        }
    }

    console.log(`Platform: ${platform}, Arch: ${arch}`);
    console.log(`Library name: ${libName}, Platform dir: ${platformDir}`);

    // NEW: Primary search paths using centralized native/ directory
    const searchPaths = [
        // 1. NEW STRUCTURE: Centralized native libraries
        path.join(__dirname, 'sdks', 'nodejs', '..', '..', '..', 'native', platformDir, libName),
        path.join(__dirname, 'sdks', 'nodejs', '..', '..', '..', 'native', 'windows', libName), // Windows fallback
        
        // 2. BACKWARD COMPATIBILITY: Old locations (for transition period)
        path.join(__dirname, 'sdks', 'nodejs', libName),
        path.join(__dirname, 'sdks', 'nodejs', 'lib', libName),
        path.join(__dirname, 'sdks', 'nodejs', '..', 'target', 'release', libName),
        path.join(__dirname, 'sdks', 'nodejs', '..', '..', 'target', 'release', libName),
    ];

    console.log('\nSearch paths:');
    for (let i = 0; i < searchPaths.length; i++) {
        const p = path.resolve(searchPaths[i]);
        const exists = fs.existsSync(p);
        const size = exists ? fs.statSync(p).size : 0;
        console.log(`  ${i + 1}. ${p} - ${exists ? `EXISTS (${size} bytes)` : 'NOT FOUND'}`);
        
        if (exists) {
            console.log(`     -> This would be selected!`);
            return p;
        }
    }
    
    console.log('\nNo library found, would fall back to system path');
    return libName; // Fall back to system path
}

const foundPath = findLibrary();