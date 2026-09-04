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
    const plat = platform();
    const osName = process.platform === 'win32' ? 'windows' : process.platform === 'darwin' ? 'macos' : 'linux';
    const archName = process.arch === 'x64' ? 'x64' : process.arch === 'arm64' ? 'arm64' : process.arch;

    // 1. Env override
    if (process.env.OVERDRIVE_LIB_PATH) {
        if (fs.existsSync(process.env.OVERDRIVE_LIB_PATH)) return process.env.OVERDRIVE_LIB_PATH;
    }

    // 2. Search local and parent directories up to repo root
    let dir = __dirname;
    for (let i = 0; i < 4; i++) {
        const candidates = [
            path.join(dir, 'lib', plat, name),
            path.join(dir, 'native', osName, name),
            path.join(dir, 'native', osName, archName, name),
            path.join(dir, name),
        ];
        for (const candidate of candidates) {
            if (fs.existsSync(candidate)) return candidate;
        }
        const parent = path.dirname(dir);
        if (parent === dir) break;
        dir = parent;
    }

    // 3. System PATH (let koffi try)
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
