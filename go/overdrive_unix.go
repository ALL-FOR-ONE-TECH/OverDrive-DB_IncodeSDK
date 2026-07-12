//go:build !windows

package overdrive

// BEFORE: this file did not exist. The only native-symbol implementation
// in the whole module was `overdrive_windows.go`, and Go's build system
// automatically restricts any `_windows.go` file to `GOOS=windows` by
// filename convention. `overdrive.go` (the cross-platform façade) calls
// `nativeOpen`, `nativeClose`, `nativeInsert`, etc. unconditionally — but
// on Linux and macOS those functions were defined NOWHERE, so:
//
//     $ GOOS=linux go build ./...
//     ./overdrive.go:75:20: undefined: nativeOpen
//     ./overdrive.go:97:10: undefined: nativeClose
//     ... (one compile error per native* call)
//
// i.e. the published Go SDK — despite bundling lib/linux-x64/liboverdrive.so,
// lib/linux-arm64/, lib/macos-x64/, lib/macos-arm64/, and despite go.mod /
// README claiming Linux + macOS support — could not compile on Linux or
// macOS at all. This is the single most severe bug found across both repos:
// the package is unusable on 3 of its 4 advertised platforms.
//
// AFTER: this file supplies the same `native*` functions via cgo
// dlopen/dlsym trampolines (one small typed C wrapper per C signature the
// native library exports), gated to non-Windows targets. No extra Go
// dependency is introduced, matching the zero-dependency go.mod.
//
// Also fixes the Windows side's design flaw while we're at it: the
// Windows file resolves symbols in `init()` and PANICS on failure, so
// merely `import`-ing the package crashes any program on a machine
// without the native lib, before main() runs — uncatchable by the
// importer. Here, loading is lazy (first native call) and failures come
// back as a normal Go `error`.

/*
#cgo LDFLAGS: -ldl
#include <dlfcn.h>
#include <stdlib.h>
#include <stdint.h>

typedef void* (*fn_no_args_ptr)();
typedef void* (*fn_str_ptr)(const char*);
typedef void* (*fn_str3_ptr)(const char*, const char*, const char*);
typedef void  (*fn_handle_void)(void*);
typedef int32_t (*fn_handle_str_int)(void*, const char*);
typedef void* (*fn_handle_ptr)(void*);
typedef void* (*fn_handle_str_ptr)(void*, const char*);
typedef void* (*fn_handle_str2_ptr)(void*, const char*, const char*);
typedef int32_t (*fn_handle_str3_int)(void*, const char*, const char*, const char*);
typedef int32_t (*fn_handle_str2_int)(void*, const char*, const char*);
typedef uint64_t (*fn_handle_i32_u64)(void*, int32_t);
typedef int32_t (*fn_handle_u64_int)(void*, uint64_t);
typedef void (*fn_handle_u64_void)(void*, uint64_t);

static void*    odb_dlopen(const char *path)               { return dlopen(path, RTLD_NOW | RTLD_LOCAL); }
static void*    odb_dlsym(void *h, const char *name)        { return dlsym(h, name); }

static void*    call_no_args_ptr(void *f)                                          { return ((fn_no_args_ptr)f)(); }
static void*    call_str_ptr(void *f, const char *a)                               { return ((fn_str_ptr)f)(a); }
static void*    call_str3_ptr(void *f, const char *a, const char *b, const char *c) { return ((fn_str3_ptr)f)(a, b, c); }
static void     call_handle_void(void *f, void *h)                                 { ((fn_handle_void)f)(h); }
static int32_t  call_handle_str_int(void *f, void *h, const char *a)               { return ((fn_handle_str_int)f)(h, a); }
static void*    call_handle_ptr(void *f, void *h)                                  { return ((fn_handle_ptr)f)(h); }
static void*    call_handle_str_ptr(void *f, void *h, const char *a)               { return ((fn_handle_str_ptr)f)(h, a); }
static void*    call_handle_str2_ptr(void *f, void *h, const char *a, const char *b){ return ((fn_handle_str2_ptr)f)(h, a, b); }
static int32_t  call_handle_str3_int(void *f, void *h, const char *a, const char *b, const char *c) { return ((fn_handle_str3_int)f)(h, a, b, c); }
static int32_t  call_handle_str2_int(void *f, void *h, const char *a, const char *b){ return ((fn_handle_str2_int)f)(h, a, b); }
static uint64_t call_handle_i32_u64(void *f, void *h, int32_t iso)                 { return ((fn_handle_i32_u64)f)(h, iso); }
static int32_t  call_handle_u64_int(void *f, void *h, uint64_t id)                 { return ((fn_handle_u64_int)f)(h, id); }
static void     call_handle_u64_void(void *f, void *h, uint64_t id)                { ((fn_handle_u64_void)f)(h, id); }
*/
import "C"

import (
	"fmt"
	"sync"
	"unsafe"
)

var (
	unixLib     unsafe.Pointer
	unixLibOnce sync.Once
	unixLibErr  error

	pOpen, pOpenWithEngine, pClose, pSync, pVersion, pLastError,
	pCreateTable, pDropTable, pListTables, pTableExists, pInsert, pGet,
	pUpdate, pDelete, pCount, pQuery, pSearch, pFreeString, pBeginTxn,
	pCommitTxn, pAbortTxn unsafe.Pointer
)

func ensureUnixLoaded() error {
	unixLibOnce.Do(func() {
		path := libPath()
		cpath := C.CString(path)
		defer C.free(unsafe.Pointer(cpath))

		h := C.odb_dlopen(cpath)
		if h == nil {
			unixLibErr = fmt.Errorf("[overdrive-sdk] failed to load native library from %s", path)
			return
		}
		unixLib = h

		must := func(name string) (unsafe.Pointer, error) {
			cname := C.CString(name)
			defer C.free(unsafe.Pointer(cname))
			p := C.odb_dlsym(h, cname)
			if p == nil {
				return nil, fmt.Errorf("[overdrive-sdk] symbol not found: %s", name)
			}
			return p, nil
		}

		targets := []struct {
			name string
			dst  *unsafe.Pointer
		}{
			{"overdrive_open", &pOpen},
			{"overdrive_open_with_engine", &pOpenWithEngine},
			{"overdrive_close", &pClose},
			{"overdrive_sync", &pSync},
			{"overdrive_version", &pVersion},
			{"overdrive_last_error", &pLastError},
			{"overdrive_create_table", &pCreateTable},
			{"overdrive_drop_table", &pDropTable},
			{"overdrive_list_tables", &pListTables},
			{"overdrive_table_exists", &pTableExists},
			{"overdrive_insert", &pInsert},
			{"overdrive_get", &pGet},
			{"overdrive_update", &pUpdate},
			{"overdrive_delete", &pDelete},
			{"overdrive_count", &pCount},
			{"overdrive_query", &pQuery},
			{"overdrive_search", &pSearch},
			{"overdrive_free_string", &pFreeString},
			{"overdrive_begin_transaction", &pBeginTxn},
			{"overdrive_commit_transaction", &pCommitTxn},
			{"overdrive_abort_transaction", &pAbortTxn},
		}
		for _, t := range targets {
			p, err := must(t.name)
			if err != nil {
				unixLibErr = err
				return
			}
			*t.dst = p
		}
	})
	return unixLibErr
}

func ugostrFreed(p unsafe.Pointer) string {
	if p == nil {
		return ""
	}
	s := C.GoString((*C.char)(p))
	C.call_handle_void(pFreeString, p)
	return s
}

func ugostrStatic(p unsafe.Pointer) string {
	if p == nil {
		return ""
	}
	return C.GoString((*C.char)(p))
}

func nativeVersion() string {
	if ensureUnixLoaded() != nil {
		return ""
	}
	return ugostrStatic(C.call_no_args_ptr(pVersion))
}

func nativeOpen(path string) (uintptr, error) {
	if err := ensureUnixLoaded(); err != nil {
		return 0, err
	}
	cp := C.CString(path)
	defer C.free(unsafe.Pointer(cp))
	r := C.call_str_ptr(pOpen, cp)
	if r == nil {
		return 0, fmt.Errorf("%s", nativeLastError())
	}
	return uintptr(r), nil
}

func nativeOpenWithEngine(path, engine, opts string) (uintptr, error) {
	if err := ensureUnixLoaded(); err != nil {
		return 0, err
	}
	cp, ce, co := C.CString(path), C.CString(engine), C.CString(opts)
	defer C.free(unsafe.Pointer(cp))
	defer C.free(unsafe.Pointer(ce))
	defer C.free(unsafe.Pointer(co))
	r := C.call_str3_ptr(pOpenWithEngine, cp, ce, co)
	if r == nil {
		return 0, fmt.Errorf("%s", nativeLastError())
	}
	return uintptr(r), nil
}

func nativeClose(h uintptr) {
	if ensureUnixLoaded() != nil {
		return
	}
	C.call_handle_void(pClose, unsafe.Pointer(h))
}

func nativeSync(h uintptr) {
	if ensureUnixLoaded() != nil {
		return
	}
	C.call_handle_void(pSync, unsafe.Pointer(h))
}

func nativeLastError() string {
	if ensureUnixLoaded() != nil {
		return ""
	}
	return ugostrStatic(C.call_no_args_ptr(pLastError))
}

func nativeCreateTable(h uintptr, name string) int32 {
	if ensureUnixLoaded() != nil {
		return -1
	}
	cn := C.CString(name)
	defer C.free(unsafe.Pointer(cn))
	return int32(C.call_handle_str_int(pCreateTable, unsafe.Pointer(h), cn))
}

func nativeDropTable(h uintptr, name string) int32 {
	if ensureUnixLoaded() != nil {
		return -1
	}
	cn := C.CString(name)
	defer C.free(unsafe.Pointer(cn))
	return int32(C.call_handle_str_int(pDropTable, unsafe.Pointer(h), cn))
}

func nativeListTables(h uintptr) (string, error) {
	if err := ensureUnixLoaded(); err != nil {
		return "", err
	}
	r := C.call_handle_ptr(pListTables, unsafe.Pointer(h))
	if r == nil {
		return "", fmt.Errorf("%s", nativeLastError())
	}
	return ugostrFreed(r), nil
}

func nativeTableExists(h uintptr, name string) int32 {
	if ensureUnixLoaded() != nil {
		return 0
	}
	cn := C.CString(name)
	defer C.free(unsafe.Pointer(cn))
	return int32(C.call_handle_str_int(pTableExists, unsafe.Pointer(h), cn))
}

func nativeInsert(h uintptr, table, json string) (string, error) {
	if err := ensureUnixLoaded(); err != nil {
		return "", err
	}
	ct, cj := C.CString(table), C.CString(json)
	defer C.free(unsafe.Pointer(ct))
	defer C.free(unsafe.Pointer(cj))
	r := C.call_handle_str2_ptr(pInsert, unsafe.Pointer(h), ct, cj)
	if r == nil {
		return "", fmt.Errorf("%s", nativeLastError())
	}
	return ugostrFreed(r), nil
}

func nativeGet(h uintptr, table, id string) (string, error) {
	if err := ensureUnixLoaded(); err != nil {
		return "", err
	}
	ct, ci := C.CString(table), C.CString(id)
	defer C.free(unsafe.Pointer(ct))
	defer C.free(unsafe.Pointer(ci))
	r := C.call_handle_str2_ptr(pGet, unsafe.Pointer(h), ct, ci)
	if r == nil {
		return "", nil
	}
	return ugostrFreed(r), nil
}

func nativeUpdate(h uintptr, table, id, json string) int32 {
	if ensureUnixLoaded() != nil {
		return -1
	}
	ct, ci, cj := C.CString(table), C.CString(id), C.CString(json)
	defer C.free(unsafe.Pointer(ct))
	defer C.free(unsafe.Pointer(ci))
	defer C.free(unsafe.Pointer(cj))
	return int32(C.call_handle_str3_int(pUpdate, unsafe.Pointer(h), ct, ci, cj))
}

func nativeDelete(h uintptr, table, id string) int32 {
	if ensureUnixLoaded() != nil {
		return -1
	}
	ct, ci := C.CString(table), C.CString(id)
	defer C.free(unsafe.Pointer(ct))
	defer C.free(unsafe.Pointer(ci))
	return int32(C.call_handle_str2_int(pDelete, unsafe.Pointer(h), ct, ci))
}

func nativeCount(h uintptr, table string) int32 {
	if ensureUnixLoaded() != nil {
		return -1
	}
	ct := C.CString(table)
	defer C.free(unsafe.Pointer(ct))
	return int32(C.call_handle_str_int(pCount, unsafe.Pointer(h), ct))
}

func nativeQuery(h uintptr, sql string) (string, error) {
	if err := ensureUnixLoaded(); err != nil {
		return "", err
	}
	cs := C.CString(sql)
	defer C.free(unsafe.Pointer(cs))
	r := C.call_handle_str_ptr(pQuery, unsafe.Pointer(h), cs)
	if r == nil {
		return "", fmt.Errorf("%s", nativeLastError())
	}
	return ugostrFreed(r), nil
}

func nativeSearch(h uintptr, table, text string) (string, error) {
	if err := ensureUnixLoaded(); err != nil {
		return "", err
	}
	ct, cx := C.CString(table), C.CString(text)
	defer C.free(unsafe.Pointer(ct))
	defer C.free(unsafe.Pointer(cx))
	r := C.call_handle_str2_ptr(pSearch, unsafe.Pointer(h), ct, cx)
	if r == nil {
		return "", nil
	}
	return ugostrFreed(r), nil
}

func nativeBeginTransaction(h uintptr, iso int32) uint64 {
	if ensureUnixLoaded() != nil {
		return 0
	}
	return uint64(C.call_handle_i32_u64(pBeginTxn, unsafe.Pointer(h), C.int32_t(iso)))
}

func nativeCommitTransaction(h uintptr, txnID uint64) int32 {
	if ensureUnixLoaded() != nil {
		return -1
	}
	return int32(C.call_handle_u64_int(pCommitTxn, unsafe.Pointer(h), C.uint64_t(txnID)))
}

func nativeAbortTransaction(h uintptr, txnID uint64) {
	if ensureUnixLoaded() != nil {
		return
	}
	C.call_handle_u64_void(pAbortTxn, unsafe.Pointer(h), C.uint64_t(txnID))
}
