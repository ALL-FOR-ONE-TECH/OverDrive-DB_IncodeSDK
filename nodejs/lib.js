'use strict';
/**
 * Native library loader — resolves overdrive.dll/liboverdrive.so/dylib
 * using platform-arch directories with a 4-tier fallback chain.
 */

const path = require('path');
const fs   = require('fs');

function libName() {
    switch (process.platform) {
        case 'win32':  return 'overdrive.dll';
        case 'darwin': return 'liboverdrive.dylib';
        default:       return 'liboverdrive.so';
    }
}

function platform() {
    const os   = process.platform === 'win32' ? 'windows' : process.platform === 'darwin' ? 'macos' : 'linux';
    const arch  = process.arch === 'x64' ? 'x64' : process.arch === 'arm64' ? 'arm64' : process.arch;
    return `${os}-${arch}`;
}

function findLib() {
    const name = libName();

    // 1. Env override
    if (process.env.OVERDRIVE_LIB_PATH) {
        if (fs.existsSync(process.env.OVERDRIVE_LIB_PATH)) return process.env.OVERDRIVE_LIB_PATH;
    }

    // 2. Bundled lib/{os}-{arch}/ — CI copies lib/ into nodejs/lib/ before publish
    const bundled = path.join(__dirname, 'lib', platform(), name);
    if (fs.existsSync(bundled)) return bundled;

    // 3. Same dir as index.js
    const local = path.join(__dirname, name);
    if (fs.existsSync(local)) return local;

    // 4. System PATH (let koffi try)
    return name;
}

let _lib = null;
function getLib() {
    if (!_lib) {
        const libPath = findLib();
        const koffi = require('koffi');
        _lib = koffi.load(libPath);
    }
    return _lib;
}

module.exports = { getLib, findLib };
