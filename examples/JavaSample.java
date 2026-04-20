/**
 * OverDrive-DB Java SDK — Sample Code (v1.4.3)
 * Add to pom.xml:
 *   <dependency>
 *     <groupId>com.afot</groupId>
 *     <artifactId>overdrive-db</artifactId>
 *     <version>1.4.3</version>
 *   </dependency>
 */
import com.afot.overdrive.OverDrive;

public class JavaSample {
    public static void main(String[] args) throws Exception {
        // ── 1. Open database ──────────────────────────
        OverDrive db = OverDrive.open("myapp.odb");

        // ── 2. Insert documents (table auto-created) ──
        db.insert("users", "{\"name\":\"Alice\",\"age\":30,\"role\":\"admin\"}");
        db.insert("users", "{\"name\":\"Bob\",\"age\":25,\"role\":\"user\"}");
        db.insert("users", "{\"name\":\"Carol\",\"age\":35,\"role\":\"admin\"}");

        // ── 3. Query with SQL ─────────────────────────
        String results = db.query("SELECT * FROM users WHERE age > 28");
        System.out.println("Query results: " + results);

        // ── 4. Helper methods ─────────────────────────
        String alice = db.findOne("users", "name = 'Alice'");
        System.out.println("Found: " + alice);

        String admins = db.findAll("users", "role = 'admin'", null, 0);
        System.out.println("Admins: " + admins);

        int count = db.countWhere("users", "age > 25");
        System.out.println("Users older than 25: " + count);

        // ── 5. Transactions ───────────────────────────
        long txnId = db.beginTransaction(1);
        db.insert("users", "{\"name\":\"Dave\",\"age\":28,\"role\":\"user\"}");
        db.commitTransaction(txnId);
        System.out.println("Transaction committed");

        // ── 6. Version ────────────────────────────────
        System.out.println("SDK version: " + OverDrive.version());

        // ── 7. Cleanup ────────────────────────────────
        db.close();
        System.out.println("\n✅ Done!");
    }
}
