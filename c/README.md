# OverDrive InCode SDK — C/C++

**Embeddable document database — like SQLite for JSON.**

## Install

1. Download the native library from [GitHub Releases](https://github.com/ALL-FOR-ONE-TECH/OverDrive-DB_SDK/releases/latest)
2. Copy `overdrive.h` to your include path
3. Link against the library

### Compile example (GCC/Clang)

```bash
# Linux
gcc -o myapp myapp.c -I./include -L./lib -loverdrive -lm -ldl -lpthread

# macOS
clang -o myapp myapp.c -I./include -L./lib -loverdrive

# Windows (MSVC)
cl /I include myapp.c /link /LIBPATH:lib overdrive.lib
```

### CMake

```cmake
cmake_minimum_required(VERSION 3.16)
project(myapp C)

add_executable(myapp main.c)
target_include_directories(myapp PRIVATE ${CMAKE_SOURCE_DIR}/include)
target_link_directories(myapp PRIVATE ${CMAKE_SOURCE_DIR}/lib)
target_link_libraries(myapp overdrive)
```

## Quick Start

```c
#include "overdrive.h"
#include <stdio.h>

int main() {
    ODB* db = overdrive_open("myapp.odb");
    if (!db) {
        printf("Error: %s\n", overdrive_last_error());
        return 1;
    }

    overdrive_create_table(db, "users");

    char* id = overdrive_insert(db, "users",
        "{\"name\":\"Alice\",\"age\":30}");
    printf("Inserted: %s\n", id);
    overdrive_free_string(id);

    char* result = overdrive_query(db,
        "SELECT * FROM users WHERE age > 25");
    printf("Results: %s\n", result);
    overdrive_free_string(result);

    overdrive_close(db);
    return 0;
}
```

## Memory Rules

- Every `char*` returned by `overdrive_*` functions **must** be freed with `overdrive_free_string()`
- The `ODB*` handle **must** be closed with `overdrive_close()`
- `overdrive_last_error()` and `overdrive_version()` return static strings — do **NOT** free them

## License

MIT / Apache-2.0
