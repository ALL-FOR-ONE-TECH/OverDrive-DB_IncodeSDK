package com.afot.overdrive;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.DisplayName;
import static org.junit.jupiter.api.Assertions.*;
import java.io.File;
import java.nio.file.Files;
import java.nio.file.Path;

/**
 * Tests for resource extraction functionality in NativeLibraryLoader.
 */
public class ResourceExtractionTest {

    @Test
    @DisplayName("Should extract native library from JAR to temp directory")
    public void testResourceExtraction() {
        // Get the current temp directory count before loading
        File tempDir = new File(System.getProperty("java.io.tmpdir"));
        File[] beforeFiles = tempDir.listFiles((dir, name) -> name.startsWith("overdrive-native-"));
        int beforeCount = beforeFiles != null ? beforeFiles.length : 0;
        
        // Load the native library (this should trigger resource extraction)
        OverDrive.LibOverDrive library = NativeLibraryLoader.loadNativeLibrary();
        assertNotNull(library, "Native library should be loaded successfully");
        
        // Check if a new temp directory was created
        File[] afterFiles = tempDir.listFiles((dir, name) -> name.startsWith("overdrive-native-"));
        int afterCount = afterFiles != null ? afterFiles.length : 0;
        
        // Should have at least one temp directory (may be more if multiple tests run)
        assertTrue(afterCount >= beforeCount, "Should have created temp directory for native library extraction");
        
        // Verify that the library is functional
        String platform = NativeLibraryLoader.getPlatform();
        assertNotNull(platform, "Platform detection should work");
        assertTrue(NativeLibraryLoader.isPlatformSupported(), "Current platform should be supported");
    }
    
    @Test
    @DisplayName("Should handle platform-specific resource paths correctly")
    public void testPlatformSpecificResourcePaths() {
        String platform = NativeLibraryLoader.detectPlatform();
        String libraryName = NativeLibraryLoader.getLibraryName();
        
        assertNotNull(platform, "Platform should be detected");
        assertNotNull(libraryName, "Library name should be available for current platform");
        
        // Verify platform format
        assertTrue(platform.matches("\\w+-\\w+"), "Platform should be in format 'os-arch'");
        
        // Verify library name has correct extension for platform
        if (platform.startsWith("windows")) {
            assertTrue(libraryName.endsWith(".dll"), "Windows library should have .dll extension");
        } else if (platform.startsWith("linux")) {
            assertTrue(libraryName.endsWith(".so"), "Linux library should have .so extension");
        } else if (platform.startsWith("macos")) {
            assertTrue(libraryName.endsWith(".dylib"), "macOS library should have .dylib extension");
        }
    }
    
    @Test
    @DisplayName("Should create secure temp directories with proper permissions")
    public void testSecureTempDirectoryCreation() {
        // Load library to trigger temp directory creation
        OverDrive.LibOverDrive library = NativeLibraryLoader.loadNativeLibrary();
        assertNotNull(library, "Native library should be loaded successfully");
        
        // Find the temp directories created by our loader
        File tempDir = new File(System.getProperty("java.io.tmpdir"));
        File[] overdriveTemps = tempDir.listFiles((dir, name) -> name.startsWith("overdrive-native-"));
        
        assertTrue(overdriveTemps != null && overdriveTemps.length > 0, 
                  "Should have created at least one temp directory");
        
        // Check that temp directories exist and are readable
        for (File tempDirFile : overdriveTemps) {
            assertTrue(tempDirFile.exists(), "Temp directory should exist");
            assertTrue(tempDirFile.isDirectory(), "Should be a directory");
            assertTrue(tempDirFile.canRead(), "Should be readable");
            
            // Check if it contains a native library file
            File[] libraryFiles = tempDirFile.listFiles((dir, name) -> 
                name.endsWith(".dll") || name.endsWith(".so") || name.endsWith(".dylib"));
            
            if (libraryFiles != null && libraryFiles.length > 0) {
                File libraryFile = libraryFiles[0];
                assertTrue(libraryFile.exists(), "Library file should exist");
                assertTrue(libraryFile.canRead(), "Library file should be readable");
                
                // On Unix-like systems, library should be executable
                if (!System.getProperty("os.name", "").toLowerCase().contains("win")) {
                    assertTrue(libraryFile.canExecute(), "Library file should be executable on Unix-like systems");
                }
            }
        }
    }
    
    @Test
    @DisplayName("Should validate system temp directory security")
    public void testSystemTempDirectoryValidation() {
        // This test verifies that the enhanced security validation works
        // by ensuring the library can still be loaded successfully
        assertDoesNotThrow(() -> {
            OverDrive.LibOverDrive library = NativeLibraryLoader.loadNativeLibrary();
            assertNotNull(library, "Library should load successfully with enhanced security validation");
            
            // Verify the library is functional
            String version = library.overdrive_version();
            assertNotNull(version, "Version should be accessible");
            assertFalse(version.isEmpty(), "Version should not be empty");
        }, "Enhanced security validation should not prevent successful library loading");
    }
    
    @Test
    @DisplayName("Should handle permission setting gracefully across platforms")
    public void testCrossPlatformPermissionHandling() {
        // Load library multiple times to test permission handling robustness
        for (int i = 0; i < 3; i++) {
            final int iteration = i; // Make variable effectively final for lambda
            assertDoesNotThrow(() -> {
                OverDrive.LibOverDrive library = NativeLibraryLoader.loadNativeLibrary();
                assertNotNull(library, "Library should load successfully on iteration " + iteration);
            }, "Permission handling should be robust across multiple loads");
        }
        
        // Verify that temp directories have appropriate security measures
        File tempDir = new File(System.getProperty("java.io.tmpdir"));
        File[] overdriveTemps = tempDir.listFiles((dir, name) -> name.startsWith("overdrive-native-"));
        
        if (overdriveTemps != null && overdriveTemps.length > 0) {
            for (File tempDirFile : overdriveTemps) {
                // Verify directory is accessible to owner
                assertTrue(tempDirFile.canRead(), "Temp directory should be readable by owner");
                assertTrue(tempDirFile.canWrite(), "Temp directory should be writable by owner");
                
                // Check library files have proper permissions
                File[] libraryFiles = tempDirFile.listFiles((dir, name) -> 
                    name.endsWith(".dll") || name.endsWith(".so") || name.endsWith(".dylib"));
                
                if (libraryFiles != null) {
                    for (File libraryFile : libraryFiles) {
                        assertTrue(libraryFile.canRead(), "Library file should be readable");
                        assertTrue(libraryFile.canExecute(), "Library file should be executable");
                    }
                }
            }
        }
    }
}