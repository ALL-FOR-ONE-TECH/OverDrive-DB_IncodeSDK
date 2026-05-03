package com.afot.overdrive;

/**
 * Demo class to test OS detection functionality.
 */
public class OSDetectionDemo {
    public static void main(String[] args) {
        System.out.println("=== OS Detection Demo ===");
        System.out.println("Detected OS: " + NativeLibraryLoader.detectOS());
        System.out.println("Detected Architecture: " + NativeLibraryLoader.detectArchitecture());
        System.out.println("Detected Platform: " + NativeLibraryLoader.detectPlatform());
        System.out.println();
        
        System.out.println("=== OS-Specific Methods ===");
        System.out.println("Is Windows: " + NativeLibraryLoader.isWindows());
        System.out.println("Is Linux: " + NativeLibraryLoader.isLinux());
        System.out.println("Is macOS: " + NativeLibraryLoader.isMacOS());
        System.out.println();
        
        System.out.println("=== Platform Support ===");
        System.out.println("Is Platform Supported: " + NativeLibraryLoader.isPlatformSupported());
        System.out.println("Library Name: " + NativeLibraryLoader.getLibraryName());
        System.out.println("Supported Platforms: " + NativeLibraryLoader.getSupportedPlatforms());
        System.out.println();
        
        System.out.println("=== System Properties ===");
        System.out.println("os.name: " + System.getProperty("os.name"));
        System.out.println("os.arch: " + System.getProperty("os.arch"));
    }
}