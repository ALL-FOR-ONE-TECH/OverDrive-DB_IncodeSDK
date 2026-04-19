//go:build !windows

package overdrive

/*
#include <dlfcn.h>
#include <stdlib.h>

static void* odb_dlopen(const char* path) {
    return dlopen(path, RTLD_NOW | RTLD_GLOBAL);
}

static void* odb_dlsym(void* handle, const char* name) {
    return dlsym(handle, name);
}

static void odb_dlclose(void* handle) {
    dlclose(handle);
}
*/
import "C"

import (
	"fmt"
	"unsafe"
)


// unixLib wraps a dlopen handle via CGo.
// CGo is only used on Linux/macOS where gcc/clang is always available.
// Windows users do NOT need CGo (see overdrive_windows.go).
type unixLib struct {
	handle unsafe.Pointer
}

func openLib(path string) (libHandle, error) {
	cpath := C.CString(path)
	defer C.free(unsafe.Pointer(cpath))
	h := C.odb_dlopen(cpath)
	if h == nil {
		return nil, fmt.Errorf("dlopen(%s) failed", path)
	}
	return &unixLib{handle: h}, nil
}

func (u *unixLib) sym(name string) (unsafe.Pointer, error) {
	cname := C.CString(name)
	defer C.free(unsafe.Pointer(cname))
	ptr := C.odb_dlsym(u.handle, cname)
	if ptr == nil {
		return nil, fmt.Errorf("dlsym(%s) not found", name)
	}
	return ptr, nil
}

func (u *unixLib) close() {
	if u.handle != nil {
		C.odb_dlclose(u.handle)
		u.handle = nil
	}
}

// ─── Call helpers (Unix — direct function pointer calls) ──────

func callFn0(sym unsafe.Pointer) uintptr {
	f := (*func() uintptr)(unsafe.Pointer(&sym))
	return (*f)()
}

func callFn0v(sym unsafe.Pointer) {
	f := (*func())(unsafe.Pointer(&sym))
	(*f)()
}

func callFn1(sym unsafe.Pointer, a1 uintptr) uintptr {
	f := (*func(uintptr) uintptr)(unsafe.Pointer(&sym))
	return (*f)(a1)
}

func callFn1v(sym unsafe.Pointer, a1 uintptr) {
	f := (*func(uintptr))(unsafe.Pointer(&sym))
	(*f)(a1)
}

func callFn2(sym unsafe.Pointer, a1, a2 uintptr) uintptr {
	f := (*func(uintptr, uintptr) uintptr)(unsafe.Pointer(&sym))
	return (*f)(a1, a2)
}

func callFn2v(sym unsafe.Pointer, a1, a2 uintptr) {
	f := (*func(uintptr, uintptr))(unsafe.Pointer(&sym))
	(*f)(a1, a2)
}

func callFn3(sym unsafe.Pointer, a1, a2, a3 uintptr) uintptr {
	f := (*func(uintptr, uintptr, uintptr) uintptr)(unsafe.Pointer(&sym))
	return (*f)(a1, a2, a3)
}

func callFn4(sym unsafe.Pointer, a1, a2, a3, a4 uintptr) uintptr {
	f := (*func(uintptr, uintptr, uintptr, uintptr) uintptr)(unsafe.Pointer(&sym))
	return (*f)(a1, a2, a3, a4)
}

func callFn1i2(sym unsafe.Pointer, a1 uintptr, a2 int32) uintptr {
	f := (*func(uintptr, int32) uintptr)(unsafe.Pointer(&sym))
	return (*f)(a1, a2)
}

func callFn1u64(sym unsafe.Pointer, a1 uintptr, a2 uint64) uintptr {
	f := (*func(uintptr, uint64) uintptr)(unsafe.Pointer(&sym))
	return (*f)(a1, a2)
}



