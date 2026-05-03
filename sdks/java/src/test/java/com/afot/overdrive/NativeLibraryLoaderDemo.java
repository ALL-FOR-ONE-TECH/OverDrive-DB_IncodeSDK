package com.afot.overdrive;

/**
 * Demonstration of the NativeLibraryLoader functionality.
 * This class shows how the new cross-platform native library loading works.
 */
public class NativeLibraryLoaderDemo {
    
    public static void main(String[] args) {
        System.out.println("=== OverDrive-DB Native Library Loader Demo ===");
        System.out.println();
        
        // Show platform detection
        System.out.println("Platform Detection:");
        System.out.println("  Detected Platform: " + NativeLibraryLoader.getPlatform());
        System.out.println("  Library Name: " + NativeLibraryLoader.getLibraryName());
        System.out.println("  Platform Supported: " + NativeLibraryLoader.isPlatformSupported());
        System.out.println();
        
        // Show all supported platforms
        System.out.println("Supported Platforms:");
        for (String platform : NativeLibraryLoader.getSupportedPlatforms()) {
            System.out.println("  - " + platform);
        }
        System.out.println();
        
        // Show system properties
        System.out.println("System Properties:");
        System.out.println("  os.name: " + System.getProperty("os.name"));
        System.out.println("  os.arch: " + System.getProperty("os.arch"));
        System.out.println("  java.version: " + System.getProperty("java.version"));
        System.out.println();
        
        // Test library loading
        System.out.println("Library Loading Test:");
        try {
            OverDrive.LibOverDrive library = NativeLibraryLoader.loadNativeLibrary();
            System.out.println("  ✓ Native library loaded successfully");
            
            String version = library.overdrive_version();
            System.out.println("  ✓ Library version: " + version);
            
            // Test basic OverDrive functionality
            System.out.println();
            System.out.println("Basic OverDrive Test:");
            OverDrive db = OverDrive.open("demo.odb");
            System.out.println("  ✓ Database opened successfully");
            
            db.createTable("demo_table");
            System.out.println("  ✓ Table created successfully");
            
            String id = db.insert("demo_table", java.util.Map.of("name", "Alice", "age", 30));
            System.out.println("  ✓ Document inserted with ID: " + id);
            
            var doc = db.get("demo_table", id);
            System.out.println("  ✓ Document retrieved: " + doc);
            
            db.close();
            System.out.println("  ✓ Database closed successfully");
            
        } catch (OverDriveException.FFIException e) {
            System.out.println("  ✗ Failed to load native library:");
            System.out.println("    Error Code: " + e.getCode());
            System.out.println("    Message: " + e.getMessage());
            System.out.println("    Suggestions:");
            for (String suggestion : e.getSuggestions()) {
                System.out.println("      - " + suggestion);
            }
        } catch (Exception e) {
            System.out.println("  ✗ Unexpected error: " + e.getMessage());
            e.printStackTrace();
        }
        
        System.out.println();
        System.out.println("=== Demo Complete ===");
    }
}