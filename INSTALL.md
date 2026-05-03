# OverDrive-DB InCode SDK — Install Guide

**Version:** v1.4.3  
**Repo:** [ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK)

---

## Quick Install (All Languages)

| Language | Command |
|----------|---------|
| Python | `pip install overdrive-db` |
| Node.js | `npm install overdrive-db` |
| Rust | `cargo add overdrive-db` |
| Go | `go get github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/go@latest` |
| Java | See [Java section](#java) below |
| C/C++ | Download header + native lib manually |

---

## Python

### Install
```bash
pip install overdrive-db
```

The native library (`overdrive.dll` / `liboverdrive.so` / `liboverdrive.dylib`) is bundled inside the wheel and automatically placed when you install. **No extra steps needed.**

### Verify
```python
from overdrive import OverDrive
print(OverDrive.version())  # e.g. "1.4.3"
```

### Auto-download (if not bundled)
If your platform isn't covered by the wheel, the SDK auto-downloads from GitHub Releases on first use. You can also trigger it manually:
```bash
python -m overdrive        # download native lib if missing
python -m overdrive --version  # check version
```

### Usage
```python
from overdrive import OverDrive

db = OverDrive.open("myapp.odb")
db.insert("users", {"name": "Alice", "age": 30})  # table auto-created
results = db.query("SELECT * FROM users WHERE age > 25")
print(results)
db.close()
```

---

## Node.js

### Install
```bash
npm install overdrive-db
```

The `postinstall` script automatically downloads the correct native library from GitHub Releases. **No extra steps needed.**

> **Offline/firewall:** If auto-download fails, download the binary manually from [Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest) and place it in `node_modules/overdrive-db/lib/`.

### Usage
```javascript
const { OverDrive } = require('overdrive-db');

const db = OverDrive.open('myapp.odb');
db.insert('users', { name: 'Alice', age: 30 });
console.log(db.query('SELECT * FROM users'));
db.close();
```

---

## Rust

### Install
Add to `Cargo.toml`:
```toml
[dependencies]
overdrive-db = "1.4.3"
```

Or:
```bash
cargo add overdrive-db
```

The crate includes the prebuilt native library in its `lib/` directory and `build.rs` automatically copies it to your target directory. If not found, it auto-downloads from GitHub Releases on first run.

> **Firewall/offline:** Download the native library manually from [Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest) and place it in `lib/` next to your `Cargo.toml`, or in the same directory as your compiled binary.

### Usage
```rust
use overdrive::OverDriveDB;

fn main() {
    let mut db = OverDriveDB::open("myapp.odb").unwrap();
    db.create_table("users").unwrap();
    let id = db.insert("users", &serde_json::json!({
        "name": "Alice", "age": 30
    })).unwrap();
    let result = db.query("SELECT * FROM users").unwrap();
    println!("{} users", result.rows.len());
    db.close().unwrap();
}
```

---

## Go

### Install
```bash
go get github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/go@latest
```

### Windows — no GCC needed! ✅
The Go SDK uses Windows' built-in `syscall.LoadDLL` — **no MinGW, no GCC, no CGo required** on Windows.

```bash
# Works on Windows without any C compiler:
go get github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/go@latest
go run .
```

The native `overdrive.dll` is auto-downloaded from GitHub Releases on first use.

### Linux / macOS — CGo required
On Linux and macOS, the SDK uses CGo for `dlopen`. GCC/clang is required (standard on Linux/macOS):

```bash
# Linux: install build-essential if needed
sudo apt-get install build-essential   # Ubuntu/Debian
# macOS: install Xcode CLT if needed
xcode-select --install                 # macOS

go get github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/go@latest
CGO_ENABLED=1 go run .
```

### Usage
```go
import "github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/go"

db, err := overdrive.Open("myapp.odb")
if err != nil { log.Fatal(err) }
defer db.Close()

id, _ := db.Insert("users", map[string]any{"name": "Alice", "age": 30})
result, _ := db.Query("SELECT * FROM users WHERE age > 25")
fmt.Printf("%d users found\n", len(result.Rows))
```

---

## Java

Java is distributed via **GitHub Packages** — this requires authentication. Unlike Maven Central, GitHub Packages needs a Personal Access Token (PAT) to download.

### Step 1: Create a GitHub PAT
1. Go to [GitHub → Settings → Developer settings → Personal access tokens](https://github.com/settings/tokens)
2. Create a token with `read:packages` scope
3. Copy the token

### Step 2: Add to `~/.m2/settings.xml`
```xml
<settings>
  <servers>
    <server>
      <id>github-overdrive</id>
      <username>YOUR_GITHUB_USERNAME</username>
      <password>YOUR_GITHUB_TOKEN</password>
    </server>
  </servers>
</settings>
```

### Step 3: Add to your `pom.xml`
```xml
<!-- Add repository -->
<repositories>
  <repository>
    <id>github-overdrive</id>
    <url>https://maven.pkg.github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK</url>
  </repository>
</repositories>

<!-- Add dependency -->
<dependency>
  <groupId>com.afot</groupId>
  <artifactId>overdrive-db</artifactId>
  <version>1.4.3</version>
</dependency>
```

### Step 4: Build
```bash
mvn compile
```

The JAR bundles the native library automatically — JNA extracts and loads it at runtime.

### Usage
```java
import com.afot.overdrive.OverDrive;
import java.util.Map;

try (OverDrive db = OverDrive.open("myapp.odb")) {
    db.insert("users", Map.of("name", "Alice", "age", 30));
    System.out.println(db.query("SELECT * FROM users"));
}
```

---

## C / C++

### Step 1: Download the header and native library
```
https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest
```

Download:
- `c/include/overdrive.h` (from this repo)
- `overdrive.dll` (Windows) / `liboverdrive.so` (Linux) / `liboverdrive.dylib` (macOS)

### Step 2: Project layout
```
myproject/
├── include/
│   └── overdrive.h
├── lib/
│   └── overdrive.dll  (or liboverdrive.so)
└── src/
    └── main.c
```

### Step 3: Compile

**Windows (MSVC):**
```cmd
cl /I include src\main.c lib\overdrive.dll /link /OUT:myapp.exe
```

**Windows (MinGW/GCC):**
```bash
gcc -Iinclude src/main.c -Llib -loverdrive -o myapp.exe
```

**Linux/macOS:**
```bash
gcc -Iinclude src/main.c -Llib -loverdrive -Wl,-rpath,./lib -o myapp
```

**CMake:**
```cmake
cmake_minimum_required(VERSION 3.15)
project(myapp)

find_library(OVERDRIVE_LIB overdrive HINTS ${CMAKE_SOURCE_DIR}/lib)
include_directories(${CMAKE_SOURCE_DIR}/include)
add_executable(myapp src/main.c)
target_link_libraries(myapp ${OVERDRIVE_LIB})
```

### Usage
```c
#include "overdrive.h"
#include <stdio.h>

int main(void) {
    ODB* db = overdrive_open("myapp.odb");
    overdrive_create_table(db, "users");
    
    char* id = overdrive_insert(db, "users", 
        "{\"name\":\"Alice\",\"age\":30}");
    overdrive_free_string(id);
    
    char* result = overdrive_query(db, "SELECT * FROM users");
    printf("%s\n", result);
    overdrive_free_string(result);
    
    overdrive_close(db);
    return 0;
}
```

---

## Troubleshooting

### Python: `OSError: Cannot load overdrive.dll`
```bash
python -m overdrive   # auto-downloads the native library
```

### Rust: `Native library not found!`
```bash
# Option 1: let it auto-download (needs internet)
cargo run

# Option 2: manual placement  
# Download overdrive.dll from GitHub Releases
# Place it in lib/ next to Cargo.toml
```

### Go (Windows): Build errors about C files
```
This should not happen — Windows build uses no CGo.
Open an issue: https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/issues
```

### Go (Linux): `cgo: C compiler not found`
```bash
sudo apt-get install build-essential   # Ubuntu
sudo yum install gcc                   # CentOS/RHEL
```

### Java: `Could not resolve artifact`
You likely skipped the GitHub Packages auth setup. Follow [Step 1–2 above](#java).

### C/C++: Linker error: `cannot find -loverdrive`
```bash
# Ensure lib/ directory has the correct file:
ls lib/
# Should show: overdrive.dll / liboverdrive.so / liboverdrive.dylib
```

---

## Downloads

| Platform | File | Download |
|----------|------|----------|
| Windows x64 | `overdrive.dll` | [Latest Release](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest) |
| Linux x64 | `liboverdrive-linux-x64.so` | [Latest Release](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest) |
| Linux ARM64 | `liboverdrive-linux-arm64.so` | [Latest Release](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest) |
| macOS ARM64 | `liboverdrive-macos-arm64.dylib` | [Latest Release](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest) |
| macOS x64 | `liboverdrive-macos-x64.dylib` | [Latest Release](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_IncodeSDK/releases/latest) |
