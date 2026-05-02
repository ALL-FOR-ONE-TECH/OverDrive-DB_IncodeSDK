package com.afot.overdrive;

import org.junit.jupiter.api.*;
import java.io.File;
import java.nio.file.*;
import java.util.*;

import static org.junit.jupiter.api.Assertions.*;

/**
 * OverDrive-DB Java SDK — E2E Tests v2.2.0
 * Instance name: odb (convention throughout)
 * Run: mvn test -Dtest=OverdriveDbE2ETest
 */
@TestMethodOrder(MethodOrderer.OrderAnnotation.class)
public class OverdriveDbE2ETest {

    private static final Path TMP_DIR = Path.of(System.getProperty("java.io.tmpdir"), "overdrive_sdk_e2e_java");

    @BeforeAll
    static void setup() throws Exception {
        Files.createDirectories(TMP_DIR);
    }

    private static String dbPath(String name) {
        return TMP_DIR.resolve(name + ".odb").toString();
    }

    private static void cleanup(String path) {
        try { Files.deleteIfExists(Path.of(path)); } catch (Exception ignored) {}
        try { Files.deleteIfExists(Path.of(path + ".wal")); } catch (Exception ignored) {}
    }

    // ── TEST 1 ────────────────────────────────────────────────────────────────
    @Test @Order(1)
    void test_01_open_creates_odb_file() {
        String path = dbPath("t01_open");
        cleanup(path);
        try (OverdriveDb odb = OverdriveDb.open(path)) {
            // just open and close
        }
        assertTrue(new File(path).exists(), "❌ .odb not created");
        assertTrue(new File(path).length() > 0, "❌ .odb is 0 bytes");
        System.out.println("  ✅ TEST 1 — .odb created (" + new File(path).length() + " bytes)");
        cleanup(path);
    }

    // ── TEST 2 ────────────────────────────────────────────────────────────────
    @Test @Order(2)
    void test_02_insert_get_roundtrip() {
        String path = dbPath("t02_crud");
        cleanup(path);
        try (OverdriveDb odb = OverdriveDb.open(path)) {
            odb.createTable("users");
            String id = odb.insert("users", Map.of("name", "Karthikeyan", "role", "engineer", "age", 28));
            assertNotNull(id);
            assertFalse(id.isEmpty());
            Map<String, Object> doc = odb.get("users", id);
            assertNotNull(doc, "❌ doc not found");
            assertEquals("Karthikeyan", doc.get("name"));
            assertEquals("engineer",   doc.get("role"));
            System.out.println("  ✅ TEST 2 — odb.insert+get: _id=" + id + ", name=" + doc.get("name"));
        }
        cleanup(path);
    }

    // ── TEST 3 ────────────────────────────────────────────────────────────────
    @Test @Order(3)
    void test_03_count_accurate() {
        String path = dbPath("t03_count");
        cleanup(path);
        try (OverdriveDb odb = OverdriveDb.open(path)) {
            odb.createTable("items");
            assertEquals(0, odb.count("items"));
            odb.insert("items", Map.of("n", "A"));
            odb.insert("items", Map.of("n", "B"));
            odb.insert("items", Map.of("n", "C"));
            assertEquals(3, odb.count("items"), "❌ count must be 3");
            System.out.println("  ✅ TEST 3 — odb.count=3");
        }
        cleanup(path);
    }

    // ── TEST 4 ────────────────────────────────────────────────────────────────
    @Test @Order(4)
    void test_04_multi_get_fields() {
        String path = dbPath("t04_multi");
        cleanup(path);
        try (OverdriveDb odb = OverdriveDb.open(path)) {
            odb.createTable("products");
            String id1 = odb.insert("products", Map.of("name", "Apple",  "price", 10));
            String id2 = odb.insert("products", Map.of("name", "Banana", "price", 5));
            String id3 = odb.insert("products", Map.of("name", "Cherry", "price", 25));
            assertEquals("Apple",  odb.get("products", id1).get("name"));
            assertEquals("Banana", odb.get("products", id2).get("name"));
            assertEquals("Cherry", odb.get("products", id3).get("name"));
            System.out.println("  ✅ TEST 4 — odb.get per-doc fields correct");
        }
        cleanup(path);
    }

    // ── TEST 5 ────────────────────────────────────────────────────────────────
    @Test @Order(5)
    void test_05_update_changes_field() {
        String path = dbPath("t05_update");
        cleanup(path);
        try (OverdriveDb odb = OverdriveDb.open(path)) {
            odb.createTable("cfg");
            String id = odb.insert("cfg", Map.of("key", "theme", "val", "light"));
            assertTrue(odb.update("cfg", id, Map.of("val", "dark")));
            assertEquals("dark", odb.get("cfg", id).get("val"));
            System.out.println("  ✅ TEST 5 — odb.update: light->dark");
        }
        cleanup(path);
    }

    // ── TEST 6 ────────────────────────────────────────────────────────────────
    @Test @Order(6)
    void test_06_delete_removes_doc() {
        String path = dbPath("t06_delete");
        cleanup(path);
        try (OverdriveDb odb = OverdriveDb.open(path)) {
            odb.createTable("logs");
            String id1 = odb.insert("logs", Map.of("msg", "e1"));
            String id2 = odb.insert("logs", Map.of("msg", "e2"));
            assertTrue(odb.delete("logs", id1));
            assertEquals(1, odb.count("logs"));
            assertNull(odb.get("logs", id1));
            assertNotNull(odb.get("logs", id2));
            System.out.println("  ✅ TEST 6 — odb.delete: count=1, deleted=null");
        }
        cleanup(path);
    }

    // ── TEST 7 ────────────────────────────────────────────────────────────────
    @Test @Order(7)
    void test_07_persist_after_reopen() {
        String path = dbPath("t07_persist");
        cleanup(path);
        String storedId;
        try (OverdriveDb odb = OverdriveDb.open(path)) {
            odb.createTable("sessions");
            storedId = odb.insert("sessions", Map.of("token", "abc123", "user", "afot"));
            odb.sync();
        }
        try (OverdriveDb odb = OverdriveDb.open(path)) {
            assertEquals(1, odb.count("sessions"), "❌ data must persist");
            Map<String, Object> doc = odb.get("sessions", storedId);
            assertNotNull(doc);
            assertEquals("abc123", doc.get("token"));
            assertEquals("afot",   doc.get("user"));
            System.out.println("  ✅ TEST 7 — persisted: token=" + doc.get("token") + ", _id=" + storedId);
        }
        cleanup(path);
    }

    // ── TEST 8 ────────────────────────────────────────────────────────────────
    @Test @Order(8)
    void test_08_insert_many() {
        String path = dbPath("t08_batch");
        cleanup(path);
        try (OverdriveDb odb = OverdriveDb.open(path)) {
            odb.createTable("orders");
            List<Map<String, Object>> docs = List.of(
                Map.of("order_id", "ORD-001", "amount", 150),
                Map.of("order_id", "ORD-002", "amount", 200),
                Map.of("order_id", "ORD-003", "amount", 75)
            );
            List<String> ids = odb.insertMany("orders", docs);
            assertEquals(3, ids.size());
            assertEquals(3, odb.count("orders"));
            for (String id : ids) assertNotNull(odb.get("orders", id));
            System.out.println("  ✅ TEST 8 — odb.insertMany: " + ids);
        }
        cleanup(path);
    }

    // ── TEST 9 ────────────────────────────────────────────────────────────────
    @Test @Order(9)
    void test_09_table_exists() {
        String path = dbPath("t09_tables");
        cleanup(path);
        try (OverdriveDb odb = OverdriveDb.open(path)) {
            assertFalse(odb.tableExists("ghost"));
            odb.createTable("real");
            assertTrue(odb.tableExists("real"));
            System.out.println("  ✅ TEST 9 — odb.tableExists: ghost=false, real=true");
        }
        cleanup(path);
    }

    // ── TEST 10 ───────────────────────────────────────────────────────────────
    @Test @Order(10)
    void test_10_version() {
        String v = OverdriveDb.version();
        assertNotNull(v);
        assertFalse(v.isEmpty(), "❌ version empty");
        assertNotEquals("unknown", v, "❌ native lib not loaded");
        assertEquals("2.2.0", v, "❌ expected 2.2.0, got " + v);
        System.out.println("  ✅ TEST 10 — odb version: " + v);
    }
}
