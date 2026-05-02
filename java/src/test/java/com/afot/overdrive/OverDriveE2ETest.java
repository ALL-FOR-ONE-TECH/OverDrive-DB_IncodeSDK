package com.afot.overdrive;

import org.junit.jupiter.api.*;
import org.junit.jupiter.api.io.TempDir;

import java.io.File;
import java.nio.file.Path;
import java.util.List;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.*;

/**
 * OverDrive-DB Java SDK — Real End-to-End Tests
 *
 * These tests ACTUALLY create .odb files using the native library.
 * A passing test proves the native lib loaded and the full stack works.
 *
 * Run: mvn test -Dtest=OverDriveE2ETest
 */
@DisplayName("OverDrive-DB Java SDK — End-to-End Tests")
@TestMethodOrder(MethodOrderer.OrderAnnotation.class)
class OverDriveE2ETest {

    @TempDir
    static Path tempDir;

    private String dbPath(String name) {
        return tempDir.resolve(name + ".odb").toAbsolutePath().toString();
    }

    // ─────────────────────────────────────────────────────────────
    // TEST 1: open() creates a real .odb file on disk
    // ─────────────────────────────────────────────────────────────
    @Test
    @Order(1)
    @DisplayName("open() creates a real .odb file on disk")
    void testOpenCreatesOdbFile() {
        String path = dbPath("open_creates");

        OverDrive db = assertDoesNotThrow(
            () -> OverDrive.open(path),
            "open() must succeed — native library must be loaded"
        );
        db.close();

        File f = new File(path);
        assertTrue(f.exists(), "❌ .odb file was NOT created at: " + path);
        assertTrue(f.length() > 0, "❌ .odb file is 0 bytes — engine did not initialize");
        System.out.println("✅ .odb file created (" + f.length() + " bytes): " + path);
    }

    // ─────────────────────────────────────────────────────────────
    // TEST 2: insert() + get() roundtrip
    // ─────────────────────────────────────────────────────────────
    @Test
    @Order(2)
    @DisplayName("insert() + get() roundtrip")
    void testInsertAndGetRoundtrip() throws Exception {
        String path = dbPath("crud_roundtrip");
        OverDrive db = OverDrive.open(path);

        db.createTable("users");

        String id = db.insert("users", Map.of(
            "name", "Karthikeyan",
            "role", "engineer",
            "age",  28
        ));

        assertNotNull(id, "❌ insert must return a non-null _id");
        assertFalse(id.isEmpty(), "❌ insert must return a non-empty _id");

        Map<String, Object> doc = db.get("users", id);
        assertNotNull(doc, "❌ document must exist after insert");
        assertEquals("Karthikeyan", doc.get("name"), "❌ name mismatch");
        assertEquals("engineer",    doc.get("role"), "❌ role mismatch");

        System.out.println("✅ Inserted _id=" + id + ", name=" + doc.get("name"));
        db.close();
    }

    // ─────────────────────────────────────────────────────────────
    // TEST 3: count() is accurate
    // ─────────────────────────────────────────────────────────────
    @Test
    @Order(3)
    @DisplayName("count() returns correct number")
    void testCountIsAccurate() throws Exception {
        String path = dbPath("count_check");
        OverDrive db = OverDrive.open(path);

        db.createTable("items");

        long before = db.count("items");
        assertEquals(0, before, "❌ empty table count must be 0");

        db.insert("items", Map.of("name", "A"));
        db.insert("items", Map.of("name", "B"));
        db.insert("items", Map.of("name", "C"));

        long after = db.count("items");
        assertEquals(3, after, "❌ count must be 3 after 3 inserts");
        System.out.println("✅ count=" + after);
        db.close();
    }

    // ─────────────────────────────────────────────────────────────
    // TEST 4: SQL query with WHERE returns filtered results
    // ─────────────────────────────────────────────────────────────
    @Test
    @Order(4)
    @DisplayName("query() with WHERE clause filters correctly")
    void testSQLQueryWithWhere() throws Exception {
        String path = dbPath("sql_query");
        OverDrive db = OverDrive.open(path);

        db.createTable("products");
        db.insert("products", Map.of("name", "Apple",  "price", 10));
        db.insert("products", Map.of("name", "Banana", "price", 5));
        db.insert("products", Map.of("name", "Cherry", "price", 25));

        List<Map<String, Object>> rows = db.query("SELECT * FROM products WHERE price > 8");

        assertFalse(rows.isEmpty(), "❌ query with WHERE price>8 must return rows");

        boolean hasApple  = rows.stream().anyMatch(r -> "Apple".equals(r.get("name")));
        boolean hasCherry = rows.stream().anyMatch(r -> "Cherry".equals(r.get("name")));
        assertTrue(hasApple,  "❌ Apple (price=10) must be in results");
        assertTrue(hasCherry, "❌ Cherry (price=25) must be in results");

        System.out.printf("✅ %d rows returned%n", rows.size());
        db.close();
    }

    // ─────────────────────────────────────────────────────────────
    // TEST 5: update() changes a field — verified by get()
    // ─────────────────────────────────────────────────────────────
    @Test
    @Order(5)
    @DisplayName("update() changes a field")
    void testUpdateChangesField() throws Exception {
        String path = dbPath("update_check");
        OverDrive db = OverDrive.open(path);

        db.createTable("config");
        String id = db.insert("config", Map.of("key", "theme", "value", "light"));

        boolean updated = db.update("config", id, Map.of("value", "dark"));
        assertTrue(updated, "❌ update must return true for existing doc");

        Map<String, Object> doc = db.get("config", id);
        assertEquals("dark", doc.get("value"), "❌ value must be 'dark' after update");

        System.out.println("✅ theme updated: light → " + doc.get("value"));
        db.close();
    }

    // ─────────────────────────────────────────────────────────────
    // TEST 6: delete() removes document — count drops by 1
    // ─────────────────────────────────────────────────────────────
    @Test
    @Order(6)
    @DisplayName("delete() removes document")
    void testDeleteRemovesDocument() throws Exception {
        String path = dbPath("delete_check");
        OverDrive db = OverDrive.open(path);

        db.createTable("logs");
        String id1 = db.insert("logs", Map.of("msg", "event1"));
        String id2 = db.insert("logs", Map.of("msg", "event2"));
        assertEquals(2, db.count("logs"));

        boolean deleted = db.delete("logs", id1);
        assertTrue(deleted, "❌ delete must return true for existing doc");
        assertEquals(1, db.count("logs"), "❌ count must be 1 after delete");
        assertNull(db.get("logs", id1), "❌ deleted doc must return null");
        assertNotNull(db.get("logs", id2), "❌ other doc must still exist");

        System.out.println("✅ id1 deleted, id2 still present");
        db.close();
    }

    // ─────────────────────────────────────────────────────────────
    // TEST 7: Data persists after close() + open()
    // ─────────────────────────────────────────────────────────────
    @Test
    @Order(7)
    @DisplayName("data persists after close() + open()")
    void testDataPersistsAfterReopen() throws Exception {
        String path = dbPath("persistence");

        // Write phase
        {
            OverDrive db = OverDrive.open(path);
            db.createTable("sessions");
            db.insert("sessions", Map.of("token", "abc123", "user", "afot_admin"));
            db.sync();
            db.close();
        }

        // Read phase — fresh open
        {
            OverDrive db = OverDrive.open(path);
            long count = db.count("sessions");
            assertEquals(1, count, "❌ data must persist after close+reopen. count=" + count);

            List<Map<String, Object>> rows = db.query("SELECT * FROM sessions");
            assertEquals(1, rows.size(), "❌ must have 1 row after reopen");
            assertEquals("abc123", rows.get(0).get("token"), "❌ token must persist");

            System.out.println("✅ persisted doc: token=" + rows.get(0).get("token"));
            db.close();
        }
    }

    // ─────────────────────────────────────────────────────────────
    // TEST 8: tableExists() returns correct bool
    // ─────────────────────────────────────────────────────────────
    @Test
    @Order(8)
    @DisplayName("tableExists() returns correct bool")
    void testTableExists() throws Exception {
        String path = dbPath("table_exists");
        OverDrive db = OverDrive.open(path);

        assertFalse(db.tableExists("ghost_table"), "❌ non-existent table must return false");
        db.createTable("real_table");
        assertTrue(db.tableExists("real_table"), "❌ created table must return true");

        System.out.println("✅ ghost=false, real_table=true");
        db.close();
    }

    // ─────────────────────────────────────────────────────────────
    // TEST 9: listTables() returns created tables
    // ─────────────────────────────────────────────────────────────
    @Test
    @Order(9)
    @DisplayName("listTables() returns all created tables")
    void testListTables() throws Exception {
        String path = dbPath("list_tables");
        OverDrive db = OverDrive.open(path);

        db.createTable("alpha");
        db.createTable("beta");
        db.createTable("gamma");

        List<String> tables = db.listTables();
        assertTrue(tables.contains("alpha"), "❌ 'alpha' must appear in listTables");
        assertTrue(tables.contains("beta"),  "❌ 'beta' must appear in listTables");
        assertTrue(tables.contains("gamma"), "❌ 'gamma' must appear in listTables");

        System.out.println("✅ listTables: " + tables);
        db.close();
    }

    // ─────────────────────────────────────────────────────────────
    // TEST 10: version() is a valid string from native lib
    // ─────────────────────────────────────────────────────────────
    @Test
    @Order(10)
    @DisplayName("version() returns a valid string from native lib")
    void testVersionIsValid() {
        String v = OverDrive.version();
        assertNotNull(v, "❌ version() must not return null");
        assertFalse(v.isEmpty(), "❌ version() must return a non-empty string");
        assertNotEquals("unknown", v, "❌ version() returned 'unknown' — native lib may not have loaded");
        System.out.println("✅ SDK version: " + v);
    }
}
