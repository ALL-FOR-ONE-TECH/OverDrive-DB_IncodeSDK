package com.afot.overdrive;

/**
 * Demo class to showcase the enhanced architecture detection capabilities.
 */
public class ArchitectureDetectionDemo {
    
    public static void main(String[] args) {
        System.out.println("=== Enhanced Architecture Detection Demo ===");
        System.out.println();
        
        // Show current system information
        System.out.println("Current System Information:");
        System.out.println("  os.name: " + System.getProperty("os.name"));
        System.out.println("  os.arch: " + System.getProperty("os.arch"));
        System.out.println("  os.version: " + System.getProperty("os.version"));
        System.out.println();
        
        // Show detected values
        System.out.println("Detected Platform Information:");
        System.out.println("  Detected OS: " + NativeLibraryLoader.detectOS());
        System.out.println("  Detected Architecture: " + NativeLibraryLoader.detectArchitecture());
        System.out.println("  Detected Platform: " + NativeLibraryLoader.detectPlatform());
        System.out.println();
        
        // Show platform support
        System.out.println("Platform Support:");
        System.out.println("  Is Platform Supported: " + NativeLibraryLoader.isPlatformSupported());
        System.out.println("  Library Name: " + NativeLibraryLoader.getLibraryName());
        System.out.println();
        
        // Show all supported platforms
        System.out.println("All Supported Platforms:");
        for (String platform : NativeLibraryLoader.getSupportedPlatforms()) {
            System.out.println("  - " + platform);
        }
        System.out.println();
        
        // Demonstrate architecture detection for various inputs
        System.out.println("Architecture Detection Examples:");
        String[] testArchs = {
            "amd64", "x86_64", "aarch64", "arm64", "arm", "ia64", 
            "i386", "x86", "unknown", "ppc", "sparc"
        };
        
        for (String arch : testArchs) {
            String detected = simulateArchDetection(arch);
            System.out.printf("  %-10s -> %s%n", arch, detected);
        }
    }
    
    /**
     * Simulates architecture detection for demonstration purposes.
     * This replicates the logic from NativeLibraryLoader.detectArchitecture().
     */
    private static String simulateArchDetection(String osArch) {
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
}