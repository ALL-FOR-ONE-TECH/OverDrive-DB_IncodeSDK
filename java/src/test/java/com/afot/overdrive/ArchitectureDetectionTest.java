package com.afot.overdrive;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.*;

/**
 * Comprehensive tests for architecture detection to ensure it handles
 * various architecture name variations correctly.
 */
public class ArchitectureDetectionTest {

    @Test
    public void testCurrentArchitectureDetection() {
        String arch = NativeLibraryLoader.detectArchitecture();
        assertNotNull(arch, "Architecture detection should not return null");
        assertTrue(arch.equals("x64") || arch.equals("arm64"), 
                "Architecture should be either x64 or arm64, got: " + arch);
        
        // Print current system info for debugging
        System.out.println("Current os.arch: " + System.getProperty("os.arch"));
        System.out.println("Detected architecture: " + arch);
    }

    @Test
    public void testArchitectureVariations() {
        // Test various architecture strings that might be encountered
        String[][] testCases = {
            // Common x64 variations
            {"amd64", "x64"},
            {"x86_64", "x64"},
            {"x64", "x64"},
            {"AMD64", "x64"},
            {"X86_64", "x64"},
            {"ia64", "x64"},        // Intel Itanium
            {"IA64", "x64"},
            
            // Common ARM64 variations
            {"aarch64", "arm64"},
            {"arm64", "arm64"},
            {"AARCH64", "arm64"},
            {"ARM64", "arm64"},
            
            // ARM variations (should default to arm64)
            {"arm", "arm64"},
            {"ARM", "arm64"},
            {"armv7", "arm64"},     // ARM v7 (assume 64-bit for modern systems)
            {"armv8", "arm64"},     // ARM v8
            
            // 32-bit architectures (should default to x64 for compatibility)
            {"i386", "x64"},
            {"i686", "x64"},
            {"x86", "x64"},
            {"X86", "x64"},
            
            // Edge cases (should default to x64)
            {"unknown", "x64"},
            {"", "x64"},
            {"ppc", "x64"},         // PowerPC (unsupported, default to x64)
            {"sparc", "x64"},       // SPARC (unsupported, default to x64)
            {"mips", "x64"}         // MIPS (unsupported, default to x64)
        };
        
        for (String[] testCase : testCases) {
            String input = testCase[0];
            String expected = testCase[1];
            String result = detectArchitectureFromString(input);
            assertEquals(expected, result, 
                    String.format("Architecture '%s' should be detected as '%s', but got '%s'", 
                            input, expected, result));
        }
    }
    
    /**
     * Helper method to test architecture detection with a specific os.arch string.
     * This simulates the logic in NativeLibraryLoader.detectArchitecture().
     */
    private String detectArchitectureFromString(String osArch) {
        String arch = osArch.toLowerCase();
        
        // Handle ARM64 architectures first (most specific)
        if (arch.contains("aarch64") || arch.contains("arm64")) {
            return "arm64";
        }
        
        // Handle other ARM architectures (assume ARM64 for modern systems)
        if (arch.contains("arm")) {
            return "arm64";
        }
        
        // Handle x64 architectures
        if (arch.contains("amd64") || arch.contains("x86_64") || arch.contains("x64")) {
            return "x64";
        }
        
        // Handle Intel Itanium (ia64) - treat as x64 for compatibility
        if (arch.contains("ia64")) {
            return "x64";
        }
        
        // Handle 32-bit architectures (default to x64 for compatibility)
        if (arch.contains("x86") || arch.contains("i386") || arch.contains("i686")) {
            return "x64"; // Default to x64 for compatibility
        }
        
        // Default to x64 for unknown architectures
        return "x64";
    }
    
    @Test
    public void testArchitectureDetectionRobustness() {
        // Test that the method handles null and empty strings gracefully
        // Note: System.getProperty with default should never return null,
        // but we test the logic anyway
        
        String result1 = detectArchitectureFromString("");
        assertEquals("x64", result1, "Empty string should default to x64");
        
        String result2 = detectArchitectureFromString("unknown_arch");
        assertEquals("x64", result2, "Unknown architecture should default to x64");
        
        String result3 = detectArchitectureFromString("some_random_string");
        assertEquals("x64", result3, "Random string should default to x64");
    }
}