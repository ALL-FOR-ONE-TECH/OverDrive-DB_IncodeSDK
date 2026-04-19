/**
 * Postinstall script — downloads the correct native binary from GitHub Releases.
 * Runs automatically after `npm install overdrive-db`.
 *
 * Always downloads from the authoritative release — no stale bundled binaries.
 */

'use strict';

const https = require('https');
const fs = require('fs');
const path = require('path');
const os = require('os');

// Always download from the official IncodeSDK repo releases
const REPO = 'ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK';
const VERSION = 'v1.4.3';

function getBinaryInfo() {
    const platform = os.platform();
    const arch = os.arch();

    if (platform === 'win32') {
        return { name: 'overdrive.dll', local: 'overdrive.dll' };
    } else if (platform === 'darwin') {
        const assetName = arch === 'arm64'
            ? 'liboverdrive-macos-arm64.dylib'
            : 'liboverdrive-macos-x64.dylib';
        return { name: assetName, local: 'liboverdrive.dylib' };
    } else {
        // Linux
        const assetName = arch === 'arm64'
            ? 'liboverdrive-linux-arm64.so'
            : 'liboverdrive-linux-x64.so';
        return { name: assetName, local: 'liboverdrive.so' };
    }
}

function download(url, dest) {
    return new Promise((resolve, reject) => {
        const req = https.get(url, { headers: { 'User-Agent': 'overdrive-db-npm' } }, (res) => {
            // Follow redirects (GitHub releases use 302)
            if (res.statusCode === 301 || res.statusCode === 302) {
                return download(res.headers.location, dest).then(resolve).catch(reject);
            }
            if (res.statusCode !== 200) {
                reject(new Error(`Download failed: HTTP ${res.statusCode} for ${url}`));
                return;
            }
            const file = fs.createWriteStream(dest);
            res.pipe(file);
            file.on('finish', () => file.close(resolve));
            file.on('error', (err) => { fs.unlink(dest, () => {}); reject(err); });
        });
        req.on('error', reject);
        req.setTimeout(60000, () => { req.destroy(); reject(new Error('Download timeout')); });
    });
}

async function main() {
    const { name, local } = getBinaryInfo();
    const libDir = path.join(__dirname, '..', 'lib');
    const destPath = path.join(libDir, local);

    // Skip if valid binary already exists (> 1MB = real binary, not placeholder)
    if (fs.existsSync(destPath) && fs.statSync(destPath).size > 1_000_000) {
        console.log(`✓ overdrive-db: Native binary already present (${local})`);
        return;
    }

    console.log(`⬇ overdrive-db: Downloading ${name} from ${VERSION}...`);

    const url = `https://github.com/${REPO}/releases/download/${VERSION}/${name}`;

    try {
        if (!fs.existsSync(libDir)) {
            fs.mkdirSync(libDir, { recursive: true });
        }

        await download(url, destPath);

        // Make executable on Unix
        if (os.platform() !== 'win32') {
            fs.chmodSync(destPath, 0o755);
        }

        const size = fs.statSync(destPath).size;
        console.log(`✓ overdrive-db: Downloaded ${local} (${(size / 1024 / 1024).toFixed(1)} MB)`);
    } catch (err) {
        console.warn(`⚠ overdrive-db: Auto-download failed: ${err.message}`);
        console.warn(`  Download manually from: https://github.com/${REPO}/releases/tag/${VERSION}`);
        console.warn(`  Place '${local}' in: ${libDir}/`);
        // Don't fail the install — user can download manually
    }
}

main();
