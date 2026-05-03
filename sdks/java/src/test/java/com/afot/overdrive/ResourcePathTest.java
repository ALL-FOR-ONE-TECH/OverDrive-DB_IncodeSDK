package com.afot.overdrive;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.DisplayName;
import static org.junit.jupiter.api.Assertions.*;
import java.io.InputStream;

/**
 * Tests to verify that resource paths are correctly constructed for the organized directory structure.
 */
public class ResourcePathTest {

    @Test
    @DisplayName("Should find native libraries in organized directory structure")
    public void testOrganizedResourcePaths() {
        // Test each supported platform's resource path
        String[][] platformTests = {
            {"windows", "x64", "overdrive.dll"},
            {"linux", "x64", "liboverdrive.so"},
            {"linux", "arm64", "liboverdrive.so"},
            {"macos", "x64", "liboverdrive.dylib"},
            {"macos", "arm64", "liboverdrive.dylib"}
        };
        
        for (String[] test : platformTests) {
            String os = test[0];
            String arch = test[1];
            String libraryName = test[2];
            
            // Construct the organized resource path
            String resourcePath = "/native/" + os + "/" + arch + "/" + libraryName;
            
            // Try to find the resource
            InputStream resourceStream = NativeLibraryLoader.class.getResourceAsStream(resourcePath);
            
            assertNotNull(resourceStream, 
                "Should find library at organized path: " + resourcePath);
            
            try {
                resourceStream.close();
            } catch (Exception e) {
                // Ignore close errors
            }
        }
    }
    
    @Test
    @DisplayName("Should handle current platform resource path correctly")
    public void testCurrentPlatformResourcePath() {
        String platform = NativeLibraryLoader.detectPlatform();
        String libraryName = NativeLibraryLoader.getLibraryName();
        
        assertNotNull(platform, "Platform should be detected");
        assertNotNull(libraryName, "Library name should be available");
        
        // Parse platform to get OS and architecture
        String[] platformParts = platform.split("-");
        assertEquals(2, platformParts.length, "Platform should be in format 'os-arch'");
        
        String os = platformParts[0];
        String arch = platformParts[1];
        
        // Construct the organized resource path
        String resourcePath = "/native/" + os + "/" + arch + "/" + libraryName;
        
        // Try to find the resource
        InputStream resourceStream = NativeLibraryLoader.class.getResourceAsStream(resourcePath);
        
        assertNotNull(resourceStream, 
            "Should find library for current platform at: " + resourcePath);
        
        try {
            resourceStream.close();
        } catch (Exception e) {
            // Ignore close errors
        }
    }
    
    @Test
    @DisplayName("Should verify legacy resource paths still work as fallback")
    public void testLegacyResourcePaths() {
        // Test that legacy paths (in root of resources) still exist for fallback
        String[] legacyLibraries = {
            "overdrive.dll",
            "liboverdrive-linux-x64.so",
            "liboverdrive-linux-arm64.so",
            "liboverdrive-macos-x64.dylib",
            "liboverdrive-macos-arm64.dylib"
        };
        
        for (String libraryName : legacyLibraries) {
            String legacyResourcePath = "/" + libraryName;
            InputStream resourceStream = NativeLibraryLoader.class.getResourceAsStream(legacyResourcePath);
            
            assertNotNull(resourceStream, 
                "Should find legacy library at root path: " + legacyResourcePath);
            
            try {
                resourceStream.close();
            } catch (Exception e) {
                // Ignore close errors
            }
        }
    }
}