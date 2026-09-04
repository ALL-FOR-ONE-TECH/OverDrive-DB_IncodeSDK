package overdrive

import (
	"fmt"
	"unsafe"
	"syscall"
)

var (
	dll               *syscall.DLL
	procOpen          *syscall.Proc
	procOpenWithEngine *syscall.Proc
	procClose         *syscall.Proc
	procSync          *syscall.Proc
	procVersion       *syscall.Proc
	procLastError     *syscall.Proc
	procCreateTable   *syscall.Proc
	procDropTable     *syscall.Proc
	procListTables    *syscall.Proc
	procTableExists   *syscall.Proc
	procInsert        *syscall.Proc
	procGet           *syscall.Proc
	procUpdate        *syscall.Proc
	procDelete        *syscall.Proc
	procCount         *syscall.Proc
	procQuery         *syscall.Proc
	procSearch        *syscall.Proc
	procFreeString    *syscall.Proc
	procBeginTxn      *syscall.Proc
	procCommitTxn     *syscall.Proc
	procAbortTxn      *syscall.Proc
)

func init() {
	path := libPath()
	var err error
	dll, err = syscall.LoadDLL(path)
	if err != nil {
		panic(fmt.Sprintf("[overdrive-sdk] Failed to load native library from %s: %v", path, err))
	}
	must := func(name string) *syscall.Proc {
		p, e := dll.FindProc(name)
		if e != nil { panic(fmt.Sprintf("[overdrive-sdk] symbol not found: %s", name)) }
		return p
	}
	procOpen          = must("overdrive_open")
	procOpenWithEngine = must("overdrive_open_with_engine")
	procClose         = must("overdrive_close")
	procSync          = must("overdrive_sync")
	procVersion       = must("overdrive_version")
	procLastError     = must("overdrive_last_error")
	procCreateTable   = must("overdrive_create_table")
	procDropTable     = must("overdrive_drop_table")
	procListTables    = must("overdrive_list_tables")
	procTableExists   = must("overdrive_table_exists")
	procInsert        = must("overdrive_insert")
	procGet           = must("overdrive_get")
	procUpdate        = must("overdrive_update")
	procDelete        = must("overdrive_delete")
	procCount         = must("overdrive_count")
	procQuery         = must("overdrive_query")
	procSearch        = must("overdrive_search")
	procFreeString    = must("overdrive_free_string")
	procBeginTxn      = must("overdrive_begin_transaction")
	procCommitTxn     = must("overdrive_commit_transaction")
	procAbortTxn      = must("overdrive_abort_transaction")
}

func cstr(s string) uintptr {
	b := append([]byte(s), 0)
	return uintptr(unsafe.Pointer(&b[0]))
}

func gostr(ptr uintptr) string {
	if ptr == 0 { return "" }
	p := *(**[1 << 20]byte)(unsafe.Pointer(&ptr))
	if p == nil { return "" }
	n := 0
	for p[n] != 0 { n++ }
	s := string(p[:n])
	procFreeString.Call(ptr)
	return s
}

func gostrStatic(ptr uintptr) string {
	if ptr == 0 { return "" }
	p := *(**[1 << 20]byte)(unsafe.Pointer(&ptr))
	if p == nil { return "" }
	n := 0
	for p[n] != 0 { n++ }
	return string(p[:n])
}

func nativeVersion() string {
	r, _, _ := procVersion.Call()
	return gostrStatic(r)
}

func nativeOpen(path string) (uintptr, error) {
	r, _, _ := procOpen.Call(cstr(path))
	if r == 0 { return 0, fmt.Errorf("%s", nativeLastError()) }
	return r, nil
}

func nativeOpenWithEngine(path, engine, opts string) (uintptr, error) {
	r, _, _ := procOpenWithEngine.Call(cstr(path), cstr(engine), cstr(opts))
	if r == 0 { return 0, fmt.Errorf("%s", nativeLastError()) }
	return r, nil
}

func nativeClose(h uintptr)  { procClose.Call(h) }
func nativeSync(h uintptr)   { procSync.Call(h) }

func nativeLastError() string {
	r, _, _ := procLastError.Call()
	return gostrStatic(r)
}

func nativeCreateTable(h uintptr, name string) int32 {
	r, _, _ := procCreateTable.Call(h, cstr(name))
	return int32(r)
}

func nativeDropTable(h uintptr, name string) int32 {
	r, _, _ := procDropTable.Call(h, cstr(name))
	return int32(r)
}

func nativeListTables(h uintptr) (string, error) {
	r, _, _ := procListTables.Call(h)
	if r == 0 { return "", fmt.Errorf("%s", nativeLastError()) }
	return gostr(r), nil
}

func nativeTableExists(h uintptr, name string) int32 {
	r, _, _ := procTableExists.Call(h, cstr(name))
	return int32(r)
}

func nativeInsert(h uintptr, table, json string) (string, error) {
	r, _, _ := procInsert.Call(h, cstr(table), cstr(json))
	if r == 0 { return "", fmt.Errorf("%s", nativeLastError()) }
	return gostr(r), nil
}

func nativeGet(h uintptr, table, id string) (string, error) {
	r, _, _ := procGet.Call(h, cstr(table), cstr(id))
	if r == 0 { return "", nil }
	return gostr(r), nil
}

func nativeUpdate(h uintptr, table, id, json string) int32 {
	r, _, _ := procUpdate.Call(h, cstr(table), cstr(id), cstr(json))
	return int32(r)
}

func nativeDelete(h uintptr, table, id string) int32 {
	r, _, _ := procDelete.Call(h, cstr(table), cstr(id))
	return int32(r)
}

func nativeCount(h uintptr, table string) int32 {
	r, _, _ := procCount.Call(h, cstr(table))
	return int32(r)
}

func nativeQuery(h uintptr, sql string) (string, error) {
	r, _, _ := procQuery.Call(h, cstr(sql))
	if r == 0 { return "", fmt.Errorf("%s", nativeLastError()) }
	return gostr(r), nil
}

func nativeSearch(h uintptr, table, text string) (string, error) {
	r, _, _ := procSearch.Call(h, cstr(table), cstr(text))
	if r == 0 { return "", nil }
	return gostr(r), nil
}

func nativeBeginTransaction(h uintptr, iso int32) uint64 {
	r, _, _ := procBeginTxn.Call(h, uintptr(iso))
	return uint64(r)
}

func nativeCommitTransaction(h uintptr, txnID uint64) int32 {
	r, _, _ := procCommitTxn.Call(h, uintptr(txnID))
	return int32(r)
}

func nativeAbortTransaction(h uintptr, txnID uint64) {
	procAbortTxn.Call(h, uintptr(txnID))
}
