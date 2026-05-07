"""Native ctypes bindings for overdrive.dll / liboverdrive.so / liboverdrive.dylib"""

import ctypes
import json
import os
import platform
import sys
from pathlib import Path

_lib = None

def _lib_name():
    s = sys.platform
    if s == 'win32':   return 'overdrive.dll'
    if s == 'darwin':  return 'liboverdrive.dylib'
    return 'liboverdrive.so'

def _platform_arch():
    s = sys.platform
    os_name = 'windows' if s == 'win32' else ('macos' if s == 'darwin' else 'linux')
    arch_map = {'AMD64': 'x64', 'x86_64': 'x64', 'arm64': 'arm64', 'aarch64': 'arm64'}
    arch = arch_map.get(platform.machine(), platform.machine())
    return f'{os_name}-{arch}'

def _find_lib():
    # 1. Env override
    env = os.environ.get('OVERDRIVE_LIB_PATH')
    if env and Path(env).exists():
        return env

    # 2. Bundled lib/{os}-{arch}/ — walk up from THIS file to IncodeSDK root
    here = Path(__file__).resolve().parent  # overdrive/
    # python/overdrive/_native.py → python/overdrive/ → python/ → IncodeSDK/
    for _ in range(4):
        candidate = here / 'lib' / _platform_arch() / _lib_name()
        if candidate.exists():
            return str(candidate)
        here = here.parent

    # 3. System
    return _lib_name()

def load():
    global _lib
    if _lib is None:
        path = _find_lib()
        _lib = ctypes.CDLL(path)
        _setup_signatures(_lib)
    return _lib

def _setup_signatures(lib):
    # Handle type (opaque pointer)
    H = ctypes.c_void_p
    S = ctypes.c_char_p
    I = ctypes.c_int
    U64 = ctypes.c_uint64

    lib.overdrive_open.restype          = H;  lib.overdrive_open.argtypes          = [S]
    lib.overdrive_open_with_engine.restype = H; lib.overdrive_open_with_engine.argtypes = [S, S, S]
    lib.overdrive_close.restype         = None; lib.overdrive_close.argtypes         = [H]
    lib.overdrive_sync.restype          = None; lib.overdrive_sync.argtypes          = [H]
    lib.overdrive_version.restype       = S;  lib.overdrive_version.argtypes       = []
    lib.overdrive_free_string.restype   = None; lib.overdrive_free_string.argtypes   = [ctypes.c_void_p]
    lib.overdrive_last_error.restype    = S;  lib.overdrive_last_error.argtypes    = []
    lib.overdrive_last_error_ex.restype = S;  lib.overdrive_last_error_ex.argtypes = [H]
    lib.overdrive_create_table.restype  = I;  lib.overdrive_create_table.argtypes  = [H, S]
    lib.overdrive_drop_table.restype    = I;  lib.overdrive_drop_table.argtypes    = [H, S]
    lib.overdrive_list_tables.restype   = ctypes.c_void_p; lib.overdrive_list_tables.argtypes = [H]
    lib.overdrive_table_exists.restype  = I;  lib.overdrive_table_exists.argtypes  = [H, S]
    lib.overdrive_insert.restype        = ctypes.c_void_p; lib.overdrive_insert.argtypes = [H, S, S]
    lib.overdrive_get.restype           = ctypes.c_void_p; lib.overdrive_get.argtypes    = [H, S, S]
    lib.overdrive_update.restype        = I;  lib.overdrive_update.argtypes        = [H, S, S, S]
    lib.overdrive_delete.restype        = I;  lib.overdrive_delete.argtypes        = [H, S, S]
    lib.overdrive_count.restype         = I;  lib.overdrive_count.argtypes         = [H, S]
    lib.overdrive_get_history.restype   = ctypes.c_void_p; lib.overdrive_get_history.argtypes = [H, S, S]
    lib.overdrive_query_safe.restype    = ctypes.c_void_p; lib.overdrive_query_safe.argtypes  = [H, S, S]
    lib.overdrive_backup.restype        = ctypes.c_void_p; lib.overdrive_backup.argtypes       = [H, S]
    lib.overdrive_cleanup_wal.restype   = I;               lib.overdrive_cleanup_wal.argtypes  = [H]
    lib.overdrive_query.restype         = ctypes.c_void_p; lib.overdrive_query.argtypes  = [H, S]
    lib.overdrive_search.restype        = ctypes.c_void_p; lib.overdrive_search.argtypes = [H, S, S]
    lib.overdrive_begin_transaction.restype  = U64; lib.overdrive_begin_transaction.argtypes  = [H, I]
    lib.overdrive_commit_transaction.restype = I;   lib.overdrive_commit_transaction.argtypes = [H, U64]
    lib.overdrive_abort_transaction.restype  = I;   lib.overdrive_abort_transaction.argtypes  = [H, U64]
    lib.overdrive_verify_integrity.restype   = ctypes.c_void_p; lib.overdrive_verify_integrity.argtypes = [H]

def encode(s: str) -> bytes:
    return s.encode('utf-8')

def read_free(lib, ptr: int) -> str:
    """Read a C string returned by overdrive and free it."""
    if not ptr:
        return ''
    s = ctypes.cast(ptr, ctypes.c_char_p).value
    result = s.decode('utf-8') if s else ''
    lib.overdrive_free_string(ptr)
    return result

def last_error(lib) -> str:
    e = lib.overdrive_last_error()
    return e.decode('utf-8') if e else ''

def last_error_ex(lib, handle) -> str:
    """Thread-safe error reader — reads from the per-handle error field.
    Use this in multithreaded environments instead of last_error()."""
    if not handle:
        return last_error(lib)
    e = lib.overdrive_last_error_ex(handle)
    if e:
        return e.decode('utf-8')
    return last_error(lib)  # fall back to thread-local
