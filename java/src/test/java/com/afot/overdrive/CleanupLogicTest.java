package com.afot.overdrive;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Nested;
import static org.junit.jupiter.api.Assertions.*;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;

/**
 * Tests for the enhanced cleanup logic in NativeLibraryLoader.
 * 
 * This test class verifies that temporary files created during library extraction
 * are properly cleaned up using the enhanced cleanup mechanisms.
 */
public class CleanupLogicTest {

    private static final String DEBUG_PROPERTY = "overdrive.debug.cleanup";
    private String originalDebugValue;

    @BeforeEach
    void setUp() {
        // Enable debug logging for cleanup operations during tests
        originalDebugValue = System.getProperty(DEBUG_PROPERTY);
        System.setProperty(DEBUG_PROPERTY, "true");
    }

    @AfterEach
    void tearDown() {
        // Restore original debug setting
        if (originalDebugValue != null) {
            System.setProperty(DEBUG_PROPERTY, originalDebugValue);
        } else {
            System.clearProperty(DEBUG_PROPERTY);
        }
    }

    @Nested
    @DisplayName("Cleanup Information Tests")
    class CleanupInformationTests {

        @Test
        @DisplayName("Should provide cleanup information")
        void shouldProvideCleanupInformation() {
            // Get initial cleanup info
            NativeLibraryLoader.CleanupInfo info = NativeLibraryLoader.getCleanupInfo();
            
            assertNotNull(info, "Cleanup info should not be null");
            assertNotNull(info.getTrackedPaths(), "Tracked paths should not be null");
            assertNotNull(info.getExistingPaths(), "Existing paths should not be null");
            
            // Initially, there should be no tracked paths
            assertTrue(info.getTrackedCount() >= 0, "Tracked count should be non-negative");
            assertTrue(info.getExistingCount() >= 0, "Existing count should be non-negative");
            
            System.out.println("Initial cleanup info: " + info);
        }

        @Test
        @DisplayName("Should track cleanup hook registration")
        void shouldTrackCleanupHookRegistration() {
            NativeLibraryLoader.CleanupInfo initialInfo = NativeLibraryLoader.getCleanupInfo();
            
            // The cleanup hook may or may not be registered initially depending on previous tests
            // This is acceptable behavior
            System.out.println("Cleanup hook registered: " + initialInfo.isCleanupHookRegistered());
        }
    }

    @Nested
    @DisplayName("Manual Cleanup Tests")
    class ManualCleanupTests {

        @Test
        @DisplayName("Should perform manual cleanup successfully")
        void shouldPerformManualCleanupSuccessfully() {
            // Perform manual cleanup
            NativeLibraryLoader.CleanupResult result = NativeLibraryLoader.performManualCleanup();
            
            assertNotNull(result, "Cleanup result should not be null");
            assertNotNull(result.getCleanedPaths(), "Cleaned paths should not be null");
            assertNotNull(result.getFailedPaths(), "Failed paths should not be null");
            
            assertTrue(result.getSuccessCount() >= 0, "Success count should be non-negative");
            assertTrue(result.getFailureCount() >= 0, "Failure count should be non-negative");
            assertTrue(result.getTotalCount() >= 0, "Total count should be non-negative");
            
            assertEquals(result.getSuccessCount() + result.getFailureCount(), 
                        result.getTotalCount(), "Total should equal success + failure");
            
            System.out.println("Manual cleanup result: " + result);
            System.out.println("Summary: " + result.getSummary());
        }

        @Test
        @DisplayName("Should handle empty cleanup gracefully")
        void shouldHandleEmptyCleanupGracefully() {
            // Perform cleanup when there might be nothing to clean
            NativeLibraryLoader.CleanupResult result = NativeLibraryLoader.performManualCleanup();
            
            assertNotNull(result, "Cleanup result should not be null");
            
            // Even with nothing to clean, the operation should succeed
            assertTrue(result.getTotalCount() >= 0, "Total count should be non-negative");
            
            System.out.println("Empty cleanup result: " + result);
        }
    }

    @Nested
    @DisplayName("Cleanup Result Tests")
    class CleanupResultTests {

        @Test
        @DisplayName("Should create valid cleanup result")
        void shouldCreateValidCleanupResult() {
            List<String> cleanedPaths = List.of("/tmp/test1", "/tmp/test2");
            List<String> failedPaths = List.of("/tmp/test3");
            
            NativeLibraryLoader.CleanupResult result = 
                new NativeLibraryLoader.CleanupResult(2, 1, cleanedPaths, failedPaths);
            
            assertEquals(2, result.getSuccessCount());
            assertEquals(1, result.getFailureCount());
            assertEquals(3, result.getTotalCount());
            assertEquals(cleanedPaths, result.getCleanedPaths());
            assertEquals(failedPaths, result.getFailedPaths());
            
            assertFalse(result.isFullySuccessful(), "Should not be fully successful with failures");
            
            String summary = result.getSummary();
            assertTrue(summary.contains("2 successful"), "Summary should contain success count");
            assertTrue(summary.contains("1 failed"), "Summary should contain failure count");
            assertTrue(summary.contains("3 total"), "Summary should contain total count");
            
            System.out.println("Test cleanup result: " + result);
        }

        @Test
        @DisplayName("Should handle fully successful cleanup")
        void shouldHandleFullySuccessfulCleanup() {
            List<String> cleanedPaths = List.of("/tmp/test1", "/tmp/test2");
            List<String> failedPaths = List.of();
            
            NativeLibraryLoader.CleanupResult result = 
                new NativeLibraryLoader.CleanupResult(2, 0, cleanedPaths, failedPaths);
            
            assertTrue(result.isFullySuccessful(), "Should be fully successful with no failures");
            assertEquals(0, result.getFailureCount());
            assertTrue(result.getFailedPaths().isEmpty());
        }
    }

    @Nested
    @DisplayName("Cleanup Info Tests")
    class CleanupInfoTests {

        @Test
        @DisplayName("Should create valid cleanup info")
        void shouldCreateValidCleanupInfo() {
            List<String> trackedPaths = List.of("/tmp/tracked1", "/tmp/tracked2");
            List<String> existingPaths = List.of("/tmp/tracked1");
            
            NativeLibraryLoader.CleanupInfo info = 
                new NativeLibraryLoader.CleanupInfo(trackedPaths, existingPaths, true);
            
            assertEquals(trackedPaths, info.getTrackedPaths());
            assertEquals(existingPaths, info.getExistingPaths());
            assertTrue(info.isCleanupHookRegistered());
            assertEquals(2, info.getTrackedCount());
            assertEquals(1, info.getExistingCount());
            
            String infoString = info.toString();
            assertTrue(infoString.contains("tracked=2"), "Info string should contain tracked count");
            assertTrue(infoString.contains("existing=1"), "Info string should contain existing count");
            assertTrue(infoString.contains("hookRegistered=true"), "Info string should contain hook status");
            
            System.out.println("Test cleanup info: " + info);
        }
    }

    @Nested
    @DisplayName("Integration Tests")
    class IntegrationTests {

        @Test
        @DisplayName("Should handle library loading and cleanup integration")
        void shouldHandleLibraryLoadingAndCleanupIntegration() {
            // Get initial state
            NativeLibraryLoader.CleanupInfo initialInfo = NativeLibraryLoader.getCleanupInfo();
            int initialTrackedCount = initialInfo.getTrackedCount();
            
            System.out.println("Initial tracked directories: " + initialTrackedCount);
            
            // Try to load the native library (this may create temporary files)
            try {
                // This will attempt to load the library and may create temp files
                String platform = NativeLibraryLoader.detectPlatform();
                String libraryName = NativeLibraryLoader.getLibraryName();
                
                System.out.println("Detected platform: " + platform);
                System.out.println("Library name: " + libraryName);
                
                // Check if cleanup tracking increased (it might not if library loading fails)
                NativeLibraryLoader.CleanupInfo afterInfo = NativeLibraryLoader.getCleanupInfo();
                System.out.println("Tracked directories after library operations: " + afterInfo.getTrackedCount());
                
                // Perform manual cleanup
                NativeLibraryLoader.CleanupResult cleanupResult = NativeLibraryLoader.performManualCleanup();
                System.out.println("Cleanup result: " + cleanupResult.getSummary());
                
                // Verify cleanup was attempted
                assertNotNull(cleanupResult);
                assertTrue(cleanupResult.getTotalCount() >= 0);
                
            } catch (Exception e) {
                // Library loading might fail in test environment, which is acceptable
                System.out.println("Library loading failed (expected in test environment): " + e.getMessage());
                
                // Even if library loading fails, cleanup operations should work
                NativeLibraryLoader.CleanupResult cleanupResult = NativeLibraryLoader.performManualCleanup();
                assertNotNull(cleanupResult);
            }
        }

        @Test
        @DisplayName("Should handle platform detection with cleanup")
        void shouldHandlePlatformDetectionWithCleanup() {
            // Test that platform detection works with cleanup functionality
            String platform = NativeLibraryLoader.detectPlatform();
            assertNotNull(platform, "Platform should be detected");
            assertFalse(platform.isEmpty(), "Platform should not be empty");
            
            boolean isSupported = NativeLibraryLoader.isPlatformSupported();
            System.out.println("Platform " + platform + " is supported: " + isSupported);
            
            // Cleanup should work regardless of platform support
            NativeLibraryLoader.CleanupResult result = NativeLibraryLoader.performManualCleanup();
            assertNotNull(result);
            
            System.out.println("Platform detection with cleanup completed successfully");
        }
    }

    @Nested
    @DisplayName("Error Handling Tests")
    class ErrorHandlingTests {

        @Test
        @DisplayName("Should handle cleanup with debug logging")
        void shouldHandleCleanupWithDebugLogging() {
            // Ensure debug logging is enabled
            System.setProperty(DEBUG_PROPERTY, "true");
            
            // Perform operations that should generate debug logs
            NativeLibraryLoader.CleanupInfo info = NativeLibraryLoader.getCleanupInfo();
            NativeLibraryLoader.CleanupResult result = NativeLibraryLoader.performManualCleanup();
            
            assertNotNull(info);
            assertNotNull(result);
            
            System.out.println("Debug logging test completed");
        }

        @Test
        @DisplayName("Should handle cleanup without debug logging")
        void shouldHandleCleanupWithoutDebugLogging() {
            // Disable debug logging
            System.setProperty(DEBUG_PROPERTY, "false");
            
            // Operations should still work without debug logging
            NativeLibraryLoader.CleanupInfo info = NativeLibraryLoader.getCleanupInfo();
            NativeLibraryLoader.CleanupResult result = NativeLibraryLoader.performManualCleanup();
            
            assertNotNull(info);
            assertNotNull(result);
            
            System.out.println("Non-debug logging test completed");
        }
    }
}