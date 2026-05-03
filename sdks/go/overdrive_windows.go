//go:build windows

package overdrive

import (
	"fmt"
	"syscall"
	"unsafe"
)

// windowsLib wraps a Windows HMODULE.
type windowsLib struct {
	dll *syscall.DLL
}

func openLib(path string) (libHandle, error) {
	dll, err := syscall.LoadDLL(path)
	if err != nil {
		return nil, fmt.Errorf("LoadDLL(%s): %w", path, err)
	}
	return &windowsLib{dll: dll}, nil
}

func (w *windowsLib) sym(name string) (unsafe.Pointer, error) {
	proc, err := w.dll.FindProc(name)
	if err != nil {
		return nil, fmt.Errorf("GetProcAddress(%s): %w", name, err)
	}
	return unsafe.Pointer(proc.Addr()), nil
}

func (w *windowsLib) close() {
	_ = w.dll.Release()
}

// ─── Call helpers (Windows — uses SyscallN) ───────────

func callFn0(sym unsafe.Pointer) uintptr {
	r, _, _ := syscall.SyscallN(uintptr(sym))
	return r
}

func callFn0v(sym unsafe.Pointer) {
	syscall.SyscallN(uintptr(sym))
}

func callFn1(sym unsafe.Pointer, a1 uintptr) uintptr {
	r, _, _ := syscall.SyscallN(uintptr(sym), a1)
	return r
}

func callFn1v(sym unsafe.Pointer, a1 uintptr) {
	syscall.SyscallN(uintptr(sym), a1)
}

func callFn2(sym unsafe.Pointer, a1, a2 uintptr) uintptr {
	r, _, _ := syscall.SyscallN(uintptr(sym), a1, a2)
	return r
}

func callFn2v(sym unsafe.Pointer, a1, a2 uintptr) {
	syscall.SyscallN(uintptr(sym), a1, a2)
}

func callFn3(sym unsafe.Pointer, a1, a2, a3 uintptr) uintptr {
	r, _, _ := syscall.SyscallN(uintptr(sym), a1, a2, a3)
	return r
}

func callFn4(sym unsafe.Pointer, a1, a2, a3, a4 uintptr) uintptr {
	r, _, _ := syscall.SyscallN(uintptr(sym), a1, a2, a3, a4)
	return r
}

func callFn1i2(sym unsafe.Pointer, a1 uintptr, a2 int32) uintptr {
	r, _, _ := syscall.SyscallN(uintptr(sym), a1, uintptr(a2))
	return r
}

func callFn1u64(sym unsafe.Pointer, a1 uintptr, a2 uint64) uintptr {
	r, _, _ := syscall.SyscallN(uintptr(sym), a1, uintptr(a2))
	return r
}
