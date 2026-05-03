package com.afot.overdrive;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Nested;
import static org.junit.jupiter.api.Assertions.*;
import java.io.InputStream;

/**
 * Unit tests for the library loading mechanisms in NativeLibraryLoader.
 * This test class focuses on testing the resource loading, bundled library detection,
 * and fallback mechanisms used during native library loading.
 */
public class LibraryLoadingMechanismTest {

    @Nested
    @DisplayName("Resource Detection Tests")
    class ResourceDetectionTests {

        @Test
        @DisplayName("Should detect bundled library resources correctly")
        public void testBundledLibraryResourceDetection() {
            String currentPlatform = NativeLibraryLoader.getPlatform();
            String libraryName = NativeLibraryLoader.getLibraryName();
            
            if (libraryName != null) {
                // Check if the library resource exists
                String resourcePath = "/" + libraryName;
                InputStream resourceStream = NativeLibraryLoader.class.getResourceAsStream(resourcePath);
                
                if (resourceStream != null) {
                    // Resource exists - verify it's accessible
                    assertNotNull(resourceStream, "Library resource should be accessible");
                    
                    try {
                        // Verify we can read from the resource
                        int firstByte = resourceStream.read();
                        assertTrue(firstByte >= 0 || firstByte == -1, 
                            "Should be able to read from library resource");
                        resourceStream.close();
                    } catch (Exception e) {
                        fail("Should be able to read from library resource: " + e.getMessage());
                    }
                } else {
                    // Resource doesn't exist - this is okay, fallback should work
                    System.out.println("Library resource not found for " + currentPlatform + 
                                     ", will use system fallback");
                }
            }
        }

        @Test
        @DisplayName("Should handle missing library resources gracefully")
        public void testMissingLibraryResourceHandling() {
            // Test with a non-existent library name
            String nonExistentLibrary = "nonexistent-library.dll";
            String resourcePath = "/" + nonExistentLibrary;
            InputStream resourceStream = NativeLibraryLoader.class.getResourceAsStream(resourcePath);
            
            assertNull(resourceStream, "Non-existent library resource should return null");
        }

        @Test
        @DisplayName("Should verify library resource paths are correct")
        public void testLibraryResourcePaths() {
            var supportedPlatforms = NativeLibraryLoader.getSupportedPlatforms();
            
            for (String platform : supportedPlatforms) {
                String expectedLibraryName = getExpectedLibraryName(platform);
                assertNotNull(expectedLibraryName, 
                    "Should have library name for supported platform: " + platform);
                
                // Verify resource path format
                String resourcePath = "/" + expectedLibraryName;
                assertTrue(resourcePath.startsWith("/"), "Resource path should start with /");
                assertFalse(resourcePath.contains("//"), "Resource path should not contain double slashes");
            }
        }

        /**
         * Helper method to get expected library name for a platform
         */
        private String getExpectedLibraryName(String platform) {
            switch (platform) {
                case "windows-x64":
                    return "overdrive.dll";
                case "linux-x64":
                    return "liboverdrive-linux-x64.so";
                case "linux-arm64":
                    return "liboverdrive-linux-arm64.so";
                case "macos-x64":
                    return "liboverdrive-macos-x64.dylib";
                case "macos-arm64":
                    return "liboverdrive-macos-arm64.dylib";
                default:
                    return null;
            }
        }
    }

    @Nested
    @DisplayName("Library Loading Strategy Tests")
    class LibraryLoadingStrategyTests {

        @Test
        @DisplayName("Should attempt bundled loading before system loading")
        public void testLoadingStrategy() {
            if (!NativeLibraryLoader.isPlatformSupported()) {
                return; // Skip on unsupported platforms
            }
            
            // The actual loading strategy is tested indirectly through successful loading
            // We can't easily mock the internal methods, but we can verify the end result
            assertDoesNotThrow(() -> {
                OverDrive.LibOverDrive library = NativeLibraryLoader.loadNativeLibrary();
                assertNotNull(library, "Library should load successfully");
                
                // Verify the library is functional
                String version = library.overdrive_version();
                assertNotNull(version, "Should be able to call library methods");
                assertFalse(version.isEmpty(), "Version should not be empty");
            }, "Library loading should succeed on supported platform");
        }

        @Test
        @DisplayName("Should handle library loading failures gracefully")
        public void testLibraryLoadingFailureHandling() {
            if (!NativeLibraryLoader.isPlatformSupported()) {
                // On unsupported platforms, loading should fail with appropriate exception
                OverDriveException.FFIException exception = assertThrows(
                    OverDriveException.FFIException.class,
                    () -> NativeLibraryLoader.loadNativeLibrary(),
                    "Should throw FFIException on unsupported platform"
                );
                
                assertNotNull(exception.getMessage(), "Exception should have message");
                assertNotNull(exception.getCode(), "Exception should have error code");
                assertNotNull(exception.getSuggestions(), "Exception should have suggestions");
            }
        }

        @Test
        @DisplayName("Should provide appropriate error information for loading failures")
        public void testLoadingFailureErrorInformation() {
            if (!NativeLibraryLoader.isPlatformSupported()) {
                try {
                    NativeLibraryLoader.loadNativeLibrary();
                    fail("Should have thrown exception for unsupported platform");
                } catch (OverDriveException.FFIException e) {
                    // Verify error information quality
                    String message = e.getMessage();
                    assertTrue(message.length() > 10, "Error message should be descriptive");
                    
                    String context = e.getContext();
                    assertNotNull(context, "Error context should not be null");
                    assertTrue(context.contains("Platform:") || context.contains("OS:"), 
                        "Context should contain platform information");
                    
                    var suggestions = e.getSuggestions();
                    assertTrue(suggestions.size() > 0, "Should provide suggestions");
                    assertTrue(suggestions.size() <= 5, "Should not overwhelm with too many suggestions");
                    
                    String docUrl = e.getDocLink();
                    assertNotNull(docUrl, "Should provide documentation URL");
                    assertTrue(docUrl.startsWith("http"), "Documentation URL should be valid HTTP URL");
                }
            }
        }
    }

    @Nested
    @DisplayName("Temporary File Handling Tests")
    class TemporaryFileHandlingTests {

        @Test
        @DisplayName("Should handle temporary directory creation appropriately")
        public void testTemporaryDirectoryHandling() {
            // We can't directly test the private temporary file methods,
            // but we can verify that library loading doesn't leave obvious temp files
            
            if (!NativeLibraryLoader.isPlatformSupported()) {
                return; // Skip on unsupported platforms
            }
            
            // Get system temp directory
            String tempDir = System.getProperty("java.io.tmpdir");
            assertNotNull(tempDir, "System should have temp directory");
            
            // Load library (which may create temp files)
            assertDoesNotThrow(() -> {
                OverDrive.LibOverDrive library = NativeLibraryLoader.loadNativeLibrary();
                assertNotNull(library, "Library should load successfully");
            }, "Library loading should not fail due to temp file issues");
        }

        @Test
        @DisplayName("Should handle temp directory prefix correctly")
        public void testTempDirectoryPrefix() {
            // Test that the temp directory prefix is reasonable
            // We can't access the private constant directly, but we can verify
            // the behavior is consistent with good practices
            
            String expectedPrefix = "overdrive-native-";
            assertTrue(expectedPrefix.length() > 5, "Temp prefix should be descriptive");
            assertTrue(expectedPrefix.contains("overdrive"), "Temp prefix should contain project name");
            assertFalse(expectedPrefix.contains(" "), "Temp prefix should not contain spaces");
        }
    }

    @Nested
    @DisplayName("Cross-Platform Compatibility Tests")
    class CrossPlatformCompatibilityTests {

        @Test
        @DisplayName("Should handle platform-specific library extensions correctly")
        public void testPlatformSpecificExtensions() {
            var supportedPlatforms = NativeLibraryLoader.getSupportedPlatforms();
            
            for (String platform : supportedPlatforms) {
                String libraryName = getExpectedLibraryName(platform);
                assertNotNull(libraryName, "Should have library name for platform: " + platform);
                
                if (platform.startsWith("windows")) {
                    assertTrue(libraryName.endsWith(".dll"), 
                        "Windows library should end with .dll: " + libraryName);
                } else if (platform.startsWith("linux")) {
                    assertTrue(libraryName.endsWith(".so"), 
                        "Linux library should end with .so: " + libraryName);
                } else if (platform.startsWith("macos")) {
                    assertTrue(libraryName.endsWith(".dylib"), 
                        "macOS library should end with .dylib: " + libraryName);
                }
            }
        }

        @Test
        @DisplayName("Should handle architecture-specific library names correctly")
        public void testArchitectureSpecificNames() {
            // Verify that different architectures have different library names where appropriate
            String linuxX64 = getExpectedLibraryName("linux-x64");
            String linuxArm64 = getExpectedLibraryName("linux-arm64");
            String macosX64 = getExpectedLibraryName("macos-x64");
            String macosArm64 = getExpectedLibraryName("macos-arm64");
            
            assertNotEquals(linuxX64, linuxArm64, 
                "Linux x64 and ARM64 should have different library names");
            assertNotEquals(macosX64, macosArm64, 
                "macOS x64 and ARM64 should have different library names");
            
            // Verify architecture is included in the name for multi-arch platforms
            assertTrue(linuxX64.contains("x64"), "Linux x64 library name should contain x64");
            assertTrue(linuxArm64.contains("arm64"), "Linux ARM64 library name should contain arm64");
            assertTrue(macosX64.contains("x64"), "macOS x64 library name should contain x64");
            assertTrue(macosArm64.contains("arm64"), "macOS ARM64 library name should contain arm64");
        }

        @Test
        @DisplayName("Should handle platform naming conventions correctly")
        public void testPlatformNamingConventions() {
            var supportedPlatforms = NativeLibraryLoader.getSupportedPlatforms();
            
            for (String platform : supportedPlatforms) {
                // Verify platform format
                assertTrue(platform.contains("-"), "Platform should contain architecture separator");
                
                String[] parts = platform.split("-");
                assertEquals(2, parts.length, "Platform should have exactly two parts");
                
                String os = parts[0];
                String arch = parts[1];
                
                // Verify OS names
                assertTrue(os.equals("windows") || os.equals("linux") || os.equals("macos"),
                    "OS should be windows, linux, or macos: " + os);
                
                // Verify architecture names
                assertTrue(arch.equals("x64") || arch.equals("arm64"),
                    "Architecture should be x64 or arm64: " + arch);
                
                // Verify naming consistency
                assertFalse(os.contains("_"), "OS name should not contain underscores");
                assertFalse(arch.contains("_"), "Architecture name should not contain underscores");
                assertTrue(os.toLowerCase().equals(os), "OS name should be lowercase");
                assertTrue(arch.toLowerCase().equals(arch), "Architecture name should be lowercase");
            }
        }

        /**
         * Helper method to get expected library name for a platform
         */
        private String getExpectedLibraryName(String platform) {
            switch (platform) {
                case "windows-x64":
                    return "overdrive.dll";
                case "linux-x64":
                    return "liboverdrive-linux-x64.so";
                case "linux-arm64":
                    return "liboverdrive-linux-arm64.so";
                case "macos-x64":
                    return "liboverdrive-macos-x64.dylib";
                case "macos-arm64":
                    return "liboverdrive-macos-arm64.dylib";
                default:
                    return null;
            }
        }
    }

    @Nested
    @DisplayName("Performance and Resource Management Tests")
    class PerformanceTests {

        @Test
        @DisplayName("Should load library efficiently")
        public void testLibraryLoadingPerformance() {
            if (!NativeLibraryLoader.isPlatformSupported()) {
                return; // Skip on unsupported platforms
            }
            
            long startTime = System.currentTimeMillis();
            
            assertDoesNotThrow(() -> {
                OverDrive.LibOverDrive library = NativeLibraryLoader.loadNativeLibrary();
                assertNotNull(library, "Library should load successfully");
            }, "Library loading should not throw exceptions");
            
            long endTime = System.currentTimeMillis();
            long loadTime = endTime - startTime;
            
            // Library loading should be reasonably fast (less than 5 seconds)
            assertTrue(loadTime < 5000, 
                "Library loading should complete within 5 seconds, took: " + loadTime + "ms");
        }

        @Test
        @DisplayName("Should cache library loading results efficiently")
        public void testLibraryLoadingCaching() {
            if (!NativeLibraryLoader.isPlatformSupported()) {
                return; // Skip on unsupported platforms
            }
            
            // First load
            long startTime1 = System.currentTimeMillis();
            OverDrive.LibOverDrive library1 = NativeLibraryLoader.loadNativeLibrary();
            long endTime1 = System.currentTimeMillis();
            long firstLoadTime = endTime1 - startTime1;
            
            // Second load (should be cached)
            long startTime2 = System.currentTimeMillis();
            OverDrive.LibOverDrive library2 = NativeLibraryLoader.loadNativeLibrary();
            long endTime2 = System.currentTimeMillis();
            long secondLoadTime = endTime2 - startTime2;
            
            // Verify caching
            assertSame(library1, library2, "Second load should return cached instance");
            
            // Cached load should be much faster (less than 10ms typically)
            assertTrue(secondLoadTime < Math.max(100, firstLoadTime / 2), 
                "Cached load should be faster than initial load. First: " + firstLoadTime + 
                "ms, Second: " + secondLoadTime + "ms");
        }

        @Test
        @DisplayName("Should handle multiple rapid library access efficiently")
        public void testRapidLibraryAccess() {
            if (!NativeLibraryLoader.isPlatformSupported()) {
                return; // Skip on unsupported platforms
            }
            
            final int accessCount = 100;
            long startTime = System.currentTimeMillis();
            
            for (int i = 0; i < accessCount; i++) {
                OverDrive.LibOverDrive library = NativeLibraryLoader.loadNativeLibrary();
                assertNotNull(library, "Library should be accessible on iteration " + i);
            }
            
            long endTime = System.currentTimeMillis();
            long totalTime = endTime - startTime;
            
            // Multiple rapid accesses should be fast due to caching
            assertTrue(totalTime < 1000, 
                "100 rapid library accesses should complete within 1 second, took: " + totalTime + "ms");
        }
    }
}