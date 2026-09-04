//go:build !windows && !cgo

package overdrive

import (
	"errors"
)

var errNoCgo = errors.New("overdrive: cgo is required for loading native library on unix platforms; compile with CGO_ENABLED=1")

func nativeVersion() string {
	return "unknown (nocgo)"
}

func nativeOpen(path string) (uintptr, error) {
	return 0, errNoCgo
}

func nativeOpenWithEngine(path, engine, opts string) (uintptr, error) {
	return 0, errNoCgo
}

func nativeClose(h uintptr) {}

func nativeSync(h uintptr) {}

func nativeLastError() string {
	return errNoCgo.Error()
}

func nativeCreateTable(h uintptr, name string) int32 {
	return -1
}

func nativeDropTable(h uintptr, name string) int32 {
	return -1
}

func nativeListTables(h uintptr) (string, error) {
	return "", errNoCgo
}

func nativeTableExists(h uintptr, name string) int32 {
	return 0
}

func nativeInsert(h uintptr, table, json string) (string, error) {
	return "", errNoCgo
}

func nativeGet(h uintptr, table, id string) (string, error) {
	return "", errNoCgo
}

func nativeUpdate(h uintptr, table, id, json string) int32 {
	return -1
}

func nativeDelete(h uintptr, table, id string) int32 {
	return -1
}

func nativeCount(h uintptr, table string) int32 {
	return -1
}

func nativeQuery(h uintptr, sql string) (string, error) {
	return "", errNoCgo
}

func nativeSearch(h uintptr, table, text string) (string, error) {
	return "", errNoCgo
}

func nativeBeginTransaction(h uintptr, iso int32) uint64 {
	return 0
}

func nativeCommitTransaction(h uintptr, txnID uint64) int32 {
	return -1
}

func nativeAbortTransaction(h uintptr, txnID uint64) {}
