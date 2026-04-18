"""
Helper to download the OverDrive native binary from GitHub Releases.

The native library is downloaded from the official GitHub release on first use.
This ensures the correct version is always used — no stale bundled binaries.

Usage:
    python -m overdrive.download

Or in code:
    from overdrive.download import ensure_binary
    ensure_binary()
"""

import os
import platform
import sys
import urllib.request
from pathlib import Path

# Always download from the main OverDrive-DB repo releases
# This is the authoritative source for the native library
REPO = "karthikeyanV2K/OverDrive-DB"
VERSION = "v1.4.2"

# Platform → (release asset name, local file name)
PLATFORM_MAP = {
    "Windows": {
        "x86_64": ("overdrive.dll", "overdrive.dll"),
    },
    "Linux": {
        "x86_64": ("liboverdrive-linux-x64.so", "liboverdrive.so"),
        "aarch64": ("liboverdrive-linux-arm64.so", "liboverdrive.so"),
    },
    "Darwin": {
        "x86_64": ("liboverdrive-macos-x64.dylib", "liboverdrive.dylib"),
        "arm64": ("liboverdrive-macos-arm64.dylib", "liboverdrive.dylib"),
    },
}


def get_binary_info():
    """Get the correct binary name for the current platform and architecture."""
    import platform as _platform
    system = _platform.system()
    machine = _platform.machine().lower()

    # Normalize architecture names
    arch = "x86_64"
    if machine in ("arm64", "aarch64"):
        arch = "arm64" if system == "Darwin" else "aarch64"

    platform_binaries = PLATFORM_MAP.get(system, {})
    binary_info = platform_binaries.get(arch) or platform_binaries.get("x86_64")

    if not binary_info:
        raise RuntimeError(
            f"Unsupported platform: {system} {machine}. "
            f"Download manually from: https://github.com/{REPO}/releases/tag/{VERSION}"
        )

    return binary_info  # (remote_name, local_name)


def ensure_binary(target_dir: str = None, force: bool = False) -> str:
    """
    Download the native binary if not already present.

    Args:
        target_dir: Directory to place the binary. Defaults to package directory.
        force: Re-download even if binary already exists.

    Returns:
        Path to the binary.
    """
    remote_name, local_name = get_binary_info()

    if target_dir is None:
        target_dir = str(Path(__file__).parent)

    dest = os.path.join(target_dir, local_name)

    if os.path.exists(dest) and not force:
        # Verify the file is not empty/corrupt
        if os.path.getsize(dest) > 100_000:  # > 100KB = valid binary
            return dest
        # File is too small — likely corrupt, re-download
        os.remove(dest)

    url = f"https://github.com/{REPO}/releases/download/{VERSION}/{remote_name}"
    print(f"⬇ overdrive: Downloading {remote_name} from {VERSION}...")

    try:
        urllib.request.urlretrieve(url, dest)
        size_mb = os.path.getsize(dest) / (1024 * 1024)
        print(f"✓ overdrive: Downloaded {local_name} ({size_mb:.1f} MB)")
        return dest
    except Exception as e:
        print(f"⚠ overdrive: Download failed: {e}")
        print(f"  Download manually from: https://github.com/{REPO}/releases/tag/{VERSION}")
        print(f"  Place '{local_name}' in: {target_dir}/")
        return ""


if __name__ == "__main__":
    path = ensure_binary(force="--force" in sys.argv)
    if path:
        print(f"\nBinary ready at: {path}")
    else:
        sys.exit(1)
