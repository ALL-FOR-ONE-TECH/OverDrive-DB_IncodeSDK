"""
overdrive — CLI entry point

Run with:
    python -m overdrive            # auto-download native library
    python -m overdrive --force    # re-download even if already present
    python -m overdrive --version  # print version info
"""

import sys


def main():
    if "--version" in sys.argv or "-v" in sys.argv:
        try:
            from overdrive import OverDrive
            print(f"OverDrive SDK version: {OverDrive.version()}")
        except Exception as e:
            print(f"overdrive: Native library not loaded — {e}")
            print("Run:  python -m overdrive  to download the native library")
        return

    from overdrive.download import ensure_binary
    force = "--force" in sys.argv
    path = ensure_binary(force=force)
    if path:
        print(f"\n✅ OverDrive native library ready: {path}")
        print("\nUsage:")
        print("  from overdrive import OverDrive")
        print('  db = OverDrive.open("myapp.odb")')
        print('  db.insert("users", {"name": "Alice", "age": 30})')
        print('  print(db.query("SELECT * FROM users"))')
        print("  db.close()")
    else:
        print("\n❌ Failed to download native library.")
        print("  Download manually from:")
        print("  https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest")
        sys.exit(1)


if __name__ == "__main__":
    main()
