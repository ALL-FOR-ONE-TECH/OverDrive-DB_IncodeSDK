package com.afot.overdrive;

import org.junit.jupiter.api.*;
import java.io.*;
import java.nio.file.*;
import java.util.*;

import static org.junit.jupiter.api.Assertions.*;

/**
 * OverDrive-DB Java SDK — Unit Tests (v1.4)
 * Task 45: Tests for all v1.4 features
 *
 * Run: mvn test
 */
@TestMethodOrder(MethodOrderer.OrderAnnotation.class)
class OverDriveV14Test {

    private static final String TEST_DIR = "test_data_java_v14";

    @BeforeAll
    static void setup() {
        new File(TEST_DIR).mkdirs();
    }

    @AfterAll
    static void cleanup() throws IOException {
        Path dir = Paths.get(TEST_DIR);
        if (Files.exists(dir)) {
            Files.walk(dir)
                .sorted(java.util.Comparator.reverseOrder())
                .map(Path::toFile)
                .forEach(File::delete);
        }
    }

    private String testPath(String name) {
        return TEST_DIR + "/" + name;
    }

    // ── Task 45.1: open() with options ─────────────────

    @Test @Order(1)
    void testOpenDefault() {
        try (OverDrive db = OverDrive.open(testPath("default.odb"))) {
            assertNotNull(db);
            assertEquals(testPath("default.odb"), db.getPath());
        }
    }

    @Test @Order(2)
    void testOpenWithOptions() {
        try (OverDrive db = OverDrive.open(testPath("opts.odb"), new OverDrive.OpenOptions())) {
            assertNotNull(db);
        }
    }

    @Test @Order(3)
    void testOpenRejectsInvalidEngine() {
        assertThrows(OverDriveException.class, () ->
            OverDrive.open(testPath("bad.odb"), new OverDrive.OpenOptions().engine("InvalidEngine"))
        );
    }

    @Test @Order(4)
    void testOpenRejectsShortPassword() {
        OverDriveException ex = assertThrows(OverDriveException.class, () ->
            OverDrive.open(testPath("short.odb"), new OverDrive.OpenOptions().password("abc"))
        );
        assertTrue(ex.getMessage().contains("8 characters"));
    }

    @Test @Order(5)
    void testOpenWithPassword() {
        try (OverDrive db = OverDrive.open(testPath("enc.odb"),
                new OverDrive.OpenOptions().password("my-secret-password-123"))) {
            assertNotNull(db);
        }
    }

    @Test @Order(6)
    void testAutoCreateTables() {
        try (OverDrive db = OverDrive.open(testPath("auto.odb"))) {
            db.insert("auto_table", Map.of("key", "value"));
            assertTrue(db.tableExists("auto_table"));
        }
    }

    // ── Task 45.2: RAM engine methods ──────────────────

    @Test @Order(10)
    void testCreateTableWithEngine() {
        try (OverDrive db = OverDrive.open(testPath("hybrid.odb"))) {
            db.createTable("ram_table", new OverDrive.TableOptions().engine("RAM"));
            db.insert("ram_table", Map.of("key", "fast"));
            assertEquals(1, db.count("ram_table"));
        }
    }

    @Test @Order(11)
    void testSnapshotAndRestore() {
        try (OverDrive db = OverDrive.open(testPath("ram_snap.odb"),
                new OverDrive.OpenOptions().engine("RAM"))) {
            db.insert("cache", Map.of("key", "session"));
            String snapPath = testPath("snap.odb");
            db.snapshot(snapPath);
            assertTrue(new File(snapPath).exists());
            db.restore(snapPath);
        }
    }

    @Test @Order(12)
    void testMemoryUsage() {
        try (OverDrive db = OverDrive.open(testPath("ram_mem.odb"),
                new OverDrive.OpenOptions().engine("RAM"))) {
            db.insert("data", Map.of("key", "value"));
            OverDrive.MemoryUsage usage = db.memoryUsage();
            assertNotNull(usage);
            assertTrue(usage.getBytes() >= 0);
            assertTrue(usage.getMb() >= 0);
        }
    }

    // ── Task 45.3: watchdog function ───────────────────

    @Test @Order(20)
    void testWatchdogValidDatabase() {
        String dbPath = testPath("watchdog.odb");
        try (OverDrive db = OverDrive.open(dbPath)) {
            db.insert("data", Map.of("key", "value"));
        }

        OverDrive.WatchdogReport report = OverDrive.watchdog(dbPath);
        assertNotNull(report);
        assertEquals(dbPath, report.getFilePath());
        assertTrue(report.getFileSizeBytes() > 0);
        assertEquals("valid", report.getIntegrityStatus());
        assertTrue(report.isMagicValid());
    }

    @Test @Order(21)
    void testWatchdogMissingFile() {
        OverDrive.WatchdogReport report = OverDrive.watchdog(testPath("nonexistent.odb"));
        assertEquals("missing", report.getIntegrityStatus());
    }

    // ── Task 45.4: transaction callback ────────────────

    @Test @Order(30)
    void testTransactionAutoCommit() {
        try (OverDrive db = OverDrive.open(testPath("txn_ok.odb"))) {
            db.createTable("users");

            String result = db.transaction(txn -> {
                db.insert("users", Map.of("name", "Alice"));
                return "committed";
            });

            assertEquals("committed", result);
            assertEquals(1, db.count("users"));
        }
    }

    @Test @Order(31)
    void testTransactionAutoRollback() {
        try (OverDrive db = OverDrive.open(testPath("txn_fail.odb"))) {
            db.createTable("users");

            assertThrows(RuntimeException.class, () -> {
                db.transaction(txn -> {
                    db.insert("users", Map.of("name", "Alice"));
                    throw new RuntimeException("test rollback");
                });
            });
        }
    }

    @Test @Order(32)
    void testTransactionReturnsValue() {
        try (OverDrive db = OverDrive.open(testPath("txn_val.odb"))) {
            db.createTable("data");
            Integer result = db.transaction(txn -> 42);
            assertEquals(42, result);
        }
    }

    // ── Task 45.5: helper methods ──────────────────────

    @Test @Order(40)
    void testFindOne() {
        try (OverDrive db = OverDrive.open(testPath("helpers.odb"))) {
            db.insert("items", Map.of("name", "Apple", "category", "fruit"));
            db.insert("items", Map.of("name", "Carrot", "category", "vegetable"));

            Map<String, Object> item = db.findOne("items", "category = 'fruit'");
            assertNotNull(item);
            assertEquals("fruit", item.get("category"));
        }
    }

    @Test @Order(41)
    void testFindOneReturnsNull() {
        try (OverDrive db = OverDrive.open(testPath("helpers2.odb"))) {
            db.createTable("empty");
            Map<String, Object> item = db.findOne("empty", "category = 'meat'");
            assertNull(item);
        }
    }

    @Test @Order(42)
    void testFindAll() {
        try (OverDrive db = OverDrive.open(testPath("helpers3.odb"))) {
            db.insert("items", Map.of("name", "Apple", "category", "fruit"));
            db.insert("items", Map.of("name", "Banana", "category", "fruit"));
            db.insert("items", Map.of("name", "Carrot", "category", "vegetable"));

            List<Map<String, Object>> items = db.findAll("items", "category = 'fruit'", null, 0);
            assertEquals(2, items.size());
        }
    }

    @Test @Order(43)
    void testCountWhere() {
        try (OverDrive db = OverDrive.open(testPath("helpers4.odb"))) {
            db.insert("items", Map.of("name", "Apple", "category", "fruit"));
            db.insert("items", Map.of("name", "Banana", "category", "fruit"));

            int count = db.countWhere("items", "category = 'fruit'");
            assertEquals(2, count);
        }
    }

    @Test @Order(44)
    void testExists() {
        try (OverDrive db = OverDrive.open(testPath("helpers5.odb"))) {
            String id = db.insert("items", Map.of("name", "Apple"));
            assertTrue(db.exists("items", id));
            assertFalse(db.exists("items", "nonexistent_999"));
        }
    }

    // ── Task 45.6: error handling ──────────────────────

    @Test @Order(50)
    void testErrorHierarchy() {
        OverDriveException base = new OverDriveException("test");
        assertInstanceOf(RuntimeException.class, base);

        OverDriveException.AuthenticationException auth =
            new OverDriveException.AuthenticationException("bad", "ODB-AUTH-001", "", List.of(), "");
        assertInstanceOf(OverDriveException.class, auth);
        assertEquals("ODB-AUTH-001", auth.getCode());
    }

    @Test @Order(51)
    void testErrorIncludesSuggestions() {
        OverDriveException ex = new OverDriveException("test", "ODB-AUTH-001", "ctx",
            List.of("Check password", "Try again"), "https://docs.example.com");
        assertFalse(ex.getSuggestions().isEmpty());
        assertEquals(2, ex.getSuggestions().size());
        assertFalse(ex.getDocLink().isEmpty());
    }

    // ── Task 45.7: backward compatibility ──────────────

    @Test @Order(60)
    void testLegacyOpenStillWorks() {
        try (OverDrive db = OverDrive.open(testPath("legacy.odb"))) {
            assertNotNull(db);
        }
    }

    @Test @Order(61)
    void testExplicitCreateTableStillWorks() {
        try (OverDrive db = OverDrive.open(testPath("legacy_tbl.odb"))) {
            db.createTable("explicit");
            assertTrue(db.tableExists("explicit"));
        }
    }

    @Test @Order(62)
    void testManualTransactionsStillWork() {
        try (OverDrive db = OverDrive.open(testPath("legacy_txn.odb"))) {
            db.createTable("txn_test");
            long txnId = db.beginTransaction();
            db.insert("txn_test", Map.of("key", "value"));
            db.commitTransaction(txnId);
            assertEquals(1, db.count("txn_test"));
        }
    }

    @Test @Order(63)
    void testVersionReturnsString() {
        String version = OverDrive.version();
        assertNotNull(version);
        assertFalse(version.isEmpty());
    }
}
