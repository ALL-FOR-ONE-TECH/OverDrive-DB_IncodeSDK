package com.afot.overdrive;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

/**
 * Unit tests for the NativeLibraryLoader utility class.
 */
public class NativeLibraryLoaderTest {

    @Test
    public void testOSDetection() {
        String os = NativeLibraryLoader.detectOS();
        assertNotNull(os, "OS detection should not return null");
        assertFalse(os.isEmpty(), "OS detection should not return empty string");
        
        assertTrue(os.equals("windows") || os.equals("linux") || os.equals("macos") || os.equals("unknown"),
                "OS should be one of: windows, linux, macos, unknown");
    }

    @Test
    public void testArchitectureDetection() {
        String arch = NativeLibraryLoader.detectArchitecture();
        assertNotNull(arch, "Architecture detection should not return null");
        assertFalse(arch.isEmpty(), "Architecture detection should not return empty string");
        
        assertTrue(arch.equals("x64") || arch.equals("arm64"),
                "Architecture should be one of: x64, arm64");
    }

    @Test
    public void testOSSpecificMethods() {
        String detectedOS = NativeLibraryLoader.detectOS();
        
        // Test that exactly one OS method returns true
        int trueCount = 0;
        if (NativeLibraryLoader.isWindows()) trueCount++;
        if (NativeLibraryLoader.isLinux()) trueCount++;
        if (NativeLibraryLoader.isMacOS()) trueCount++;
        
        if (detectedOS.equals("unknown")) {
            assertEquals(0, trueCount, "No OS-specific method should return true for unknown OS");
        } else {
            assertEquals(1, trueCount, "Exactly one OS-specific method should return true");
            
            // Test that the correct method returns true
            switch (detectedOS) {
                case "windows":
                    assertTrue(NativeLibraryLoader.isWindows(), "isWindows() should return true for Windows OS");
                    assertFalse(NativeLibraryLoader.isLinux(), "isLinux() should return false for Windows OS");
                    assertFalse(NativeLibraryLoader.isMacOS(), "isMacOS() should return false for Windows OS");
                    break;
                case "linux":
                    assertFalse(NativeLibraryLoader.isWindows(), "isWindows() should return false for Linux OS");
                    assertTrue(NativeLibraryLoader.isLinux(), "isLinux() should return true for Linux OS");
                    assertFalse(NativeLibraryLoader.isMacOS(), "isMacOS() should return false for Linux OS");
                    break;
                case "macos":
                    assertFalse(NativeLibraryLoader.isWindows(), "isWindows() should return false for macOS");
                    assertFalse(NativeLibraryLoader.isLinux(), "isLinux() should return false for macOS");
                    assertTrue(NativeLibraryLoader.isMacOS(), "isMacOS() should return true for macOS");
                    break;
            }
        }
    }

    @Test
    public void testPlatformDetection() {
        String platform = NativeLibraryLoader.detectPlatform();
        assertNotNull(platform, "Platform detection should not return null");
        assertFalse(platform.isEmpty(), "Platform detection should not return empty string");
        assertTrue(platform.contains("-"), "Platform should be in format 'os-arch'");
        
        // Platform should be one of the supported formats
        String[] parts = platform.split("-");
        assertEquals(2, parts.length, "Platform should have exactly two parts separated by '-'");
        
        String os = parts[0];
        String arch = parts[1];
        
        assertTrue(os.equals("windows") || os.equals("linux") || os.equals("macos") || os.equals("unknown"),
                "OS should be one of: windows, linux, macos, unknown");
        assertTrue(arch.equals("x64") || arch.equals("arm64"),
                "Architecture should be one of: x64, arm64");
    }

    @Test
    public void testGetPlatform() {
        String platform1 = NativeLibraryLoader.getPlatform();
        String platform2 = NativeLibraryLoader.detectPlatform();
        assertEquals(platform1, platform2, "getPlatform() should return same result as detectPlatform()");
    }

    @Test
    public void testGetLibraryName() {
        String platform = NativeLibraryLoader.getPlatform();
        String libraryName = NativeLibraryLoader.getLibraryName();
        
        if (NativeLibraryLoader.isPlatformSupported()) {
            assertNotNull(libraryName, "Library name should not be null for supported platform");
            assertFalse(libraryName.isEmpty(), "Library name should not be empty for supported platform");
            
            // Verify library name matches expected pattern for platform
            if (platform.startsWith("windows")) {
                assertTrue(libraryName.endsWith(".dll"), "Windows library should end with .dll");
            } else if (platform.startsWith("linux")) {
                assertTrue(libraryName.endsWith(".so"), "Linux library should end with .so");
            } else if (platform.startsWith("macos")) {
                assertTrue(libraryName.endsWith(".dylib"), "macOS library should end with .dylib");
            }
        } else {
            assertNull(libraryName, "Library name should be null for unsupported platform");
        }
    }

    @Test
    public void testGetSupportedPlatforms() {
        var supportedPlatforms = NativeLibraryLoader.getSupportedPlatforms();
        assertNotNull(supportedPlatforms, "Supported platforms should not be null");
        assertFalse(supportedPlatforms.isEmpty(), "Supported platforms should not be empty");
        
        // Verify expected platforms are supported
        assertTrue(supportedPlatforms.contains("windows-x64"), "Should support windows-x64");
        assertTrue(supportedPlatforms.contains("linux-x64"), "Should support linux-x64");
        assertTrue(supportedPlatforms.contains("linux-arm64"), "Should support linux-arm64");
        assertTrue(supportedPlatforms.contains("macos-x64"), "Should support macos-x64");
        assertTrue(supportedPlatforms.contains("macos-arm64"), "Should support macos-arm64");
        
        assertEquals(5, supportedPlatforms.size(), "Should support exactly 5 platforms");
    }

    @Test
    public void testIsPlatformSupported() {
        String currentPlatform = NativeLibraryLoader.getPlatform();
        boolean isSupported = NativeLibraryLoader.isPlatformSupported();
        
        var supportedPlatforms = NativeLibraryLoader.getSupportedPlatforms();
        assertEquals(supportedPlatforms.contains(currentPlatform), isSupported,
                "isPlatformSupported() should match whether current platform is in supported list");
    }

    @Test
    public void testLoadNativeLibrary() {
        // This test verifies that the library can be loaded successfully
        // If the current platform is supported, loading should succeed
        if (NativeLibraryLoader.isPlatformSupported()) {
            assertDoesNotThrow(() -> {
                OverDrive.LibOverDrive library = NativeLibraryLoader.loadNativeLibrary();
                assertNotNull(library, "Loaded library should not be null");
                
                // Test that we can call a basic method
                String version = library.overdrive_version();
                assertNotNull(version, "Version should not be null");
                assertFalse(version.isEmpty(), "Version should not be empty");
            }, "Loading native library should not throw exception on supported platform");
        } else {
            // On unsupported platforms, loading should throw an FFI exception
            assertThrows(OverDriveException.FFIException.class, () -> {
                NativeLibraryLoader.loadNativeLibrary();
            }, "Loading native library should throw FFIException on unsupported platform");
        }
    }

    @Test
    public void testLibraryLoadingIsCached() {
        if (NativeLibraryLoader.isPlatformSupported()) {
            OverDrive.LibOverDrive library1 = NativeLibraryLoader.loadNativeLibrary();
            OverDrive.LibOverDrive library2 = NativeLibraryLoader.loadNativeLibrary();
            
            // Should return the same instance (cached)
            assertSame(library1, library2, "Subsequent calls should return cached library instance");
        }
    }
}