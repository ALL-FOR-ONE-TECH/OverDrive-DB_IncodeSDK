package com.afot.overdrive;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Nested;
import static org.junit.jupiter.api.Assertions.*;

/**
 * Unit tests for error handling and exception scenarios in NativeLibraryLoader.
 * This test class focuses on testing error conditions, exception handling,
 * and edge cases that might occur during native library loading.
 */
public class NativeLibraryLoaderErrorTest {

    @Nested
    @DisplayName("Exception Handling Tests")
    class ExceptionHandlingTests {

        @Test
        @DisplayName("Should throw FFIException for unsupported platform during library loading")
        public void testUnsupportedPlatformException() {
            // This test can only be run if the current platform is actually unsupported
            // For supported platforms, we'll simulate the behavior
            
            String currentPlatform = NativeLibraryLoader.getPlatform();
            boolean isSupported = NativeLibraryLoader.isPlatformSupported();
            
            if (!isSupported) {
                // If current platform is unsupported, loading should throw FFIException
                OverDriveException.FFIException exception = assertThrows(
                    OverDriveException.FFIException.class,
                    () -> NativeLibraryLoader.loadNativeLibrary(),
                    "Loading native library should throw FFIException on unsupported platform"
                );
                
                // Verify exception details
                assertNotNull(exception.getMessage(), "Exception message should not be null");
                assertTrue(exception.getMessage().contains("Unsupported platform"), 
                    "Exception message should mention unsupported platform");
                assertEquals("ODB-FFI-002", exception.getCode(), 
                    "Error code should be ODB-FFI-002 for unsupported platform");
                assertNotNull(exception.getContext(), "Exception context should not be null");
                assertNotNull(exception.getSuggestions(), "Exception suggestions should not be null");
                assertFalse(exception.getSuggestions().isEmpty(), 
                    "Exception suggestions should not be empty");
                assertNotNull(exception.getDocLink(), 
                    "Exception documentation URL should not be null");
            } else {
                // For supported platforms, we can't easily test this scenario
                // but we can verify the platform is indeed supported
                assertTrue(isSupported, "Current platform should be supported");
                assertDoesNotThrow(() -> {
                    OverDrive.LibOverDrive library = NativeLibraryLoader.loadNativeLibrary();
                    assertNotNull(library, "Library should load successfully on supported platform");
                }, "Library loading should not throw exception on supported platform");
            }
        }

        @Test
        @DisplayName("Should provide meaningful error messages")
        public void testErrorMessageQuality() {
            String currentPlatform = NativeLibraryLoader.getPlatform();
            boolean isSupported = NativeLibraryLoader.isPlatformSupported();
            
            if (!isSupported) {
                try {
                    NativeLibraryLoader.loadNativeLibrary();
                    fail("Should have thrown FFIException for unsupported platform");
                } catch (OverDriveException.FFIException e) {
                    // Verify error message contains useful information
                    String message = e.getMessage();
                    assertTrue(message.contains(currentPlatform) || message.contains("platform"), 
                        "Error message should mention platform information");
                    
                    // Verify context contains system information
                    String context = e.getContext();
                    assertTrue(context.contains("Platform:") || context.contains("OS:"), 
                        "Context should contain platform or OS information");
                    
                    // Verify suggestions are helpful
                    var suggestions = e.getSuggestions();
                    assertTrue(suggestions.size() > 0, "Should provide at least one suggestion");
                    
                    boolean hasUsefulSuggestion = suggestions.stream()
                        .anyMatch(s -> s.contains("supported") || s.contains("platform") || s.contains("version"));
                    assertTrue(hasUsefulSuggestion, "Should provide useful suggestions");
                }
            }
        }
    }

    @Nested
    @DisplayName("Library Loading Edge Cases")
    class LibraryLoadingEdgeCases {

        @Test
        @DisplayName("Should handle library loading caching correctly")
        public void testLibraryLoadingCaching() {
            if (NativeLibraryLoader.isPlatformSupported()) {
                // First load
                OverDrive.LibOverDrive library1 = NativeLibraryLoader.loadNativeLibrary();
                assertNotNull(library1, "First library load should succeed");
                
                // Second load should return cached instance
                OverDrive.LibOverDrive library2 = NativeLibraryLoader.loadNativeLibrary();
                assertNotNull(library2, "Second library load should succeed");
                
                // Should be the same instance (cached)
                assertSame(library1, library2, "Subsequent loads should return cached instance");
                
                // Third load to verify caching is persistent
                OverDrive.LibOverDrive library3 = NativeLibraryLoader.loadNativeLibrary();
                assertSame(library1, library3, "Third load should also return cached instance");
            }
        }

        @Test
        @DisplayName("Should handle concurrent library loading correctly")
        public void testConcurrentLibraryLoading() throws InterruptedException {
            if (!NativeLibraryLoader.isPlatformSupported()) {
                return; // Skip test on unsupported platforms
            }
            
            final int threadCount = 5;
            final OverDrive.LibOverDrive[] results = new OverDrive.LibOverDrive[threadCount];
            final Exception[] exceptions = new Exception[threadCount];
            final Thread[] threads = new Thread[threadCount];
            
            // Create multiple threads that try to load the library simultaneously
            for (int i = 0; i < threadCount; i++) {
                final int index = i;
                threads[i] = new Thread(() -> {
                    try {
                        results[index] = NativeLibraryLoader.loadNativeLibrary();
                    } catch (Exception e) {
                        exceptions[index] = e;
                    }
                });
            }
            
            // Start all threads
            for (Thread thread : threads) {
                thread.start();
            }
            
            // Wait for all threads to complete
            for (Thread thread : threads) {
                thread.join(5000); // 5 second timeout
            }
            
            // Verify results
            for (int i = 0; i < threadCount; i++) {
                assertNull(exceptions[i], "Thread " + i + " should not have thrown exception");
                assertNotNull(results[i], "Thread " + i + " should have loaded library");
            }
            
            // All results should be the same instance (cached)
            for (int i = 1; i < threadCount; i++) {
                assertSame(results[0], results[i], 
                    "All threads should get the same cached library instance");
            }
        }
    }

    @Nested
    @DisplayName("Platform Detection Robustness Tests")
    class PlatformDetectionRobustnessTests {

        @Test
        @DisplayName("Should handle system property edge cases")
        public void testSystemPropertyEdgeCases() {
            // Test that platform detection works even with unusual system properties
            // Note: We can't actually modify system properties in tests, but we can
            // verify the current detection is robust
            
            String osName = System.getProperty("os.name");
            String osArch = System.getProperty("os.arch");
            
            assertNotNull(osName, "os.name system property should not be null");
            assertNotNull(osArch, "os.arch system property should not be null");
            
            // Verify detection methods handle current system properties correctly
            String detectedOS = NativeLibraryLoader.detectOS();
            String detectedArch = NativeLibraryLoader.detectArchitecture();
            String detectedPlatform = NativeLibraryLoader.detectPlatform();
            
            assertNotNull(detectedOS, "OS detection should not return null");
            assertNotNull(detectedArch, "Architecture detection should not return null");
            assertNotNull(detectedPlatform, "Platform detection should not return null");
            
            assertFalse(detectedOS.isEmpty(), "OS detection should not return empty string");
            assertFalse(detectedArch.isEmpty(), "Architecture detection should not return empty string");
            assertFalse(detectedPlatform.isEmpty(), "Platform detection should not return empty string");
        }

        @Test
        @DisplayName("Should provide consistent results across multiple calls")
        public void testDetectionConsistency() {
            // Test that detection methods return consistent results across multiple calls
            final int iterations = 10;
            
            String firstOS = NativeLibraryLoader.detectOS();
            String firstArch = NativeLibraryLoader.detectArchitecture();
            String firstPlatform = NativeLibraryLoader.detectPlatform();
            
            for (int i = 0; i < iterations; i++) {
                assertEquals(firstOS, NativeLibraryLoader.detectOS(), 
                    "OS detection should be consistent across calls");
                assertEquals(firstArch, NativeLibraryLoader.detectArchitecture(), 
                    "Architecture detection should be consistent across calls");
                assertEquals(firstPlatform, NativeLibraryLoader.detectPlatform(), 
                    "Platform detection should be consistent across calls");
            }
        }

        @Test
        @DisplayName("Should handle platform detection in different thread contexts")
        public void testPlatformDetectionThreadSafety() throws InterruptedException {
            final int threadCount = 5;
            final String[] osResults = new String[threadCount];
            final String[] archResults = new String[threadCount];
            final String[] platformResults = new String[threadCount];
            final Thread[] threads = new Thread[threadCount];
            
            // Create threads that perform platform detection
            for (int i = 0; i < threadCount; i++) {
                final int index = i;
                threads[i] = new Thread(() -> {
                    osResults[index] = NativeLibraryLoader.detectOS();
                    archResults[index] = NativeLibraryLoader.detectArchitecture();
                    platformResults[index] = NativeLibraryLoader.detectPlatform();
                });
            }
            
            // Start all threads
            for (Thread thread : threads) {
                thread.start();
            }
            
            // Wait for completion
            for (Thread thread : threads) {
                thread.join(5000);
            }
            
            // Verify all threads got the same results
            for (int i = 1; i < threadCount; i++) {
                assertEquals(osResults[0], osResults[i], 
                    "OS detection should be consistent across threads");
                assertEquals(archResults[0], archResults[i], 
                    "Architecture detection should be consistent across threads");
                assertEquals(platformResults[0], platformResults[i], 
                    "Platform detection should be consistent across threads");
            }
        }
    }

    @Nested
    @DisplayName("Utility Method Edge Cases")
    class UtilityMethodEdgeCases {

        @Test
        @DisplayName("Should handle getSupportedPlatforms correctly")
        public void testGetSupportedPlatformsEdgeCases() {
            var supportedPlatforms = NativeLibraryLoader.getSupportedPlatforms();
            
            assertNotNull(supportedPlatforms, "Supported platforms should not be null");
            assertFalse(supportedPlatforms.isEmpty(), "Supported platforms should not be empty");
            
            // Verify immutability
            assertThrows(UnsupportedOperationException.class, () -> {
                supportedPlatforms.clear();
            }, "Supported platforms set should be immutable");
            
            assertThrows(UnsupportedOperationException.class, () -> {
                supportedPlatforms.add("test-platform");
            }, "Should not be able to add to supported platforms set");
            
            assertThrows(UnsupportedOperationException.class, () -> {
                supportedPlatforms.remove("windows-x64");
            }, "Should not be able to remove from supported platforms set");
        }

        @Test
        @DisplayName("Should handle getLibraryName edge cases")
        public void testGetLibraryNameEdgeCases() {
            String libraryName = NativeLibraryLoader.getLibraryName();
            boolean isSupported = NativeLibraryLoader.isPlatformSupported();
            
            if (isSupported) {
                assertNotNull(libraryName, "Library name should not be null for supported platform");
                assertFalse(libraryName.isEmpty(), "Library name should not be empty for supported platform");
                assertTrue(libraryName.contains("."), "Library name should have file extension");
                
                // Verify library name format
                assertTrue(libraryName.endsWith(".dll") || 
                          libraryName.endsWith(".so") || 
                          libraryName.endsWith(".dylib"),
                    "Library name should have valid native library extension");
            } else {
                assertNull(libraryName, "Library name should be null for unsupported platform");
            }
        }

        @Test
        @DisplayName("Should handle isPlatformSupported edge cases")
        public void testIsPlatformSupportedEdgeCases() {
            boolean isSupported = NativeLibraryLoader.isPlatformSupported();
            String currentPlatform = NativeLibraryLoader.getPlatform();
            var supportedPlatforms = NativeLibraryLoader.getSupportedPlatforms();
            
            // Verify consistency
            assertEquals(supportedPlatforms.contains(currentPlatform), isSupported,
                "isPlatformSupported should match whether current platform is in supported set");
            
            // Multiple calls should return same result
            for (int i = 0; i < 5; i++) {
                assertEquals(isSupported, NativeLibraryLoader.isPlatformSupported(),
                    "isPlatformSupported should return consistent results");
            }
        }
    }

    @Nested
    @DisplayName("Error Code and Documentation Tests")
    class ErrorCodeTests {

        @Test
        @DisplayName("Should provide proper error codes for different scenarios")
        public void testErrorCodes() {
            if (!NativeLibraryLoader.isPlatformSupported()) {
                try {
                    NativeLibraryLoader.loadNativeLibrary();
                    fail("Should have thrown exception for unsupported platform");
                } catch (OverDriveException.FFIException e) {
                    assertEquals("ODB-FFI-002", e.getCode(),
                        "Unsupported platform should use error code ODB-FFI-002");
                    
                    assertNotNull(e.getDocLink(),
                        "Should provide documentation URL");
                    assertTrue(e.getDocLink().contains("ODB-FFI-002"),
                        "Documentation URL should reference the error code");
                }
            }
        }

        @Test
        @DisplayName("Should provide helpful suggestions for error scenarios")
        public void testErrorSuggestions() {
            if (!NativeLibraryLoader.isPlatformSupported()) {
                try {
                    NativeLibraryLoader.loadNativeLibrary();
                    fail("Should have thrown exception for unsupported platform");
                } catch (OverDriveException.FFIException e) {
                    var suggestions = e.getSuggestions();
                    
                    assertNotNull(suggestions, "Suggestions should not be null");
                    assertFalse(suggestions.isEmpty(), "Should provide at least one suggestion");
                    
                    // Verify suggestions are meaningful
                    boolean hasRelevantSuggestion = suggestions.stream()
                        .anyMatch(s -> s.toLowerCase().contains("platform") || 
                                      s.toLowerCase().contains("support") ||
                                      s.toLowerCase().contains("version"));
                    
                    assertTrue(hasRelevantSuggestion, 
                        "Should provide relevant suggestions about platform support");
                }
            }
        }
    }
}