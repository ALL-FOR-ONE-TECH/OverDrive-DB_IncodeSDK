package com.afot.overdrive;

import com.sun.jna.Native;
import java.io.*;
import java.nio.file.*;
import java.util.*;
import java.util.EnumSet;
import java.util.stream.Collectors;

/**
 * Utility class for cross-platform native library loading.
 * 
 * <p>This class handles platform detection and loads the appropriate native library
 * for the current operating system and architecture. It supports:</p>
 * 
 * <ul>
 *   <li>Windows x64 (overdrive.dll)</li>
 *   <li>Linux x64 (liboverdrive-linux-x64.so)</li>
 *   <li>Linux ARM64 (liboverdrive-linux-arm64.so)</li>
 *   <li>macOS x64 (liboverdrive-macos-x64.dylib)</li>
 *   <li>macOS ARM64 (liboverdrive-macos-arm64.dylib)</li>
 * </ul>
 * 
 * <p>The loader first attempts to load bundled libraries from the JAR resources.
 * If that fails, it falls back to system library loading.</p>
 * 
 * @since 1.4.3
 */
public class NativeLibraryLoader {
    
    private static final Map<String, String> LIBRARY_MAPPING = createLibraryMapping();
    private static final String TEMP_DIR_PREFIX = "overdrive-native-";
    private static volatile OverDrive.LibOverDrive cachedLibrary = null;
    private static volatile String detectedPlatform = null;
    
    // Cleanup management
    private static final Set<Path> tempDirectoriesToCleanup = Collections.synchronizedSet(new HashSet<>());
    private static volatile boolean cleanupHookRegistered = false;
    
    /**
     * Creates the platform-to-library mapping.
     * Maps platform strings to the actual library file names in the organized structure.
     */
    private static Map<String, String> createLibraryMapping() {
        Map<String, String> mapping = new HashMap<>();
        mapping.put("windows-x64", "overdrive.dll");
        mapping.put("linux-x64", "liboverdrive.so");
        mapping.put("linux-arm64", "liboverdrive.so");
        mapping.put("macos-x64", "liboverdrive.dylib");
        mapping.put("macos-arm64", "liboverdrive.dylib");
        return Collections.unmodifiableMap(mapping);
    }
    
    /**
     * Loads the appropriate native library for the current platform.
     * 
     * <p>This method performs the following steps:</p>
     * <ol>
     *   <li>Detects the current operating system and architecture</li>
     *   <li>Maps the platform to the appropriate library file</li>
     *   <li>Attempts to load the bundled library from JAR resources</li>
     *   <li>Falls back to system library loading if bundled loading fails</li>
     * </ol>
     * 
     * @return The loaded native library interface
     * @throws OverDriveException.FFIException if library loading fails
     */
    public static OverDrive.LibOverDrive loadNativeLibrary() {
        // Return cached library if already loaded
        if (cachedLibrary != null) {
            return cachedLibrary;
        }
        
        synchronized (NativeLibraryLoader.class) {
            // Double-check locking pattern
            if (cachedLibrary != null) {
                return cachedLibrary;
            }
            
            try {
                String platform = detectPlatform();
                String libraryName = LIBRARY_MAPPING.get(platform);
                
                if (libraryName == null) {
                    throw new OverDriveException.FFIException(
                        "Unsupported platform: " + platform,
                        "ODB-FFI-002",
                        "Platform: " + platform + ", OS: " + System.getProperty("os.name") + 
                        ", Arch: " + System.getProperty("os.arch"),
                        Arrays.asList(
                            "Verify your platform is supported (Windows x64, Linux x64/ARM64, macOS x64/ARM64)",
                            "Check if a newer version of the SDK supports your platform",
                            "Contact support if you believe this platform should be supported"
                        ),
                        "https://overdrive-db.com/docs/errors/ODB-FFI-002"
                    );
                }
                
                // Try multiple fallback strategies in order of preference
                OverDrive.LibOverDrive library = tryLoadWithFallbacks(libraryName, platform);
                if (library != null) {
                    cachedLibrary = library;
                    return library;
                }
                
                // If we get here, both attempts failed
                throw new OverDriveException.FFIException(
                    "Failed to load native library for platform: " + platform,
                    "ODB-FFI-001",
                    "Attempted library: " + libraryName + ", Platform: " + platform,
                    Arrays.asList(
                        "Reinstall the package to ensure native libraries are properly bundled",
                        "Verify your platform is supported (Windows x64, Linux x64/ARM64, macOS x64/ARM64)",
                        "Check that the package installation completed successfully",
                        "Ensure the native library is available in your system PATH"
                    ),
                    "https://overdrive-db.com/docs/errors/ODB-FFI-001"
                );
                
            } catch (OverDriveException e) {
                throw e;
            } catch (Exception e) {
                throw new OverDriveException.FFIException(
                    "Unexpected error during native library loading: " + e.getMessage(),
                    "ODB-FFI-003",
                    "Exception: " + e.getClass().getSimpleName(),
                    Arrays.asList(
                        "Check system logs for additional error details",
                        "Verify file system permissions",
                        "Try running with elevated privileges if necessary"
                    ),
                    "https://overdrive-db.com/docs/errors/ODB-FFI-003"
                );
            }
        }
    }
    
    /**
     * Detects the current operating system.
     * 
     * @return OS string ("windows", "linux", "macos", or "unknown")
     */
    public static String detectOS() {
        String osName = System.getProperty("os.name", "").toLowerCase();
        
        if (osName.contains("win")) {
            return "windows";
        } else if (osName.contains("nux") || osName.contains("nix")) {
            return "linux";
        } else if (osName.contains("mac") || osName.contains("darwin")) {
            return "macos";
        } else {
            return "unknown";
        }
    }
    
    /**
     * Checks if the current operating system is Windows.
     * 
     * @return true if running on Windows
     */
    public static boolean isWindows() {
        return "windows".equals(detectOS());
    }
    
    /**
     * Checks if the current operating system is Linux.
     * 
     * @return true if running on Linux
     */
    public static boolean isLinux() {
        return "linux".equals(detectOS());
    }
    
    /**
     * Checks if the current operating system is macOS.
     * 
     * @return true if running on macOS
     */
    public static boolean isMacOS() {
        return "macos".equals(detectOS());
    }
    
    /**
     * Detects the current platform (OS + architecture).
     * 
     * @return Platform string in format "os-arch" (e.g., "windows-x64", "linux-arm64")
     */
    public static String detectPlatform() {
        if (detectedPlatform != null) {
            return detectedPlatform;
        }
        
        String os = detectOS();
        String arch = detectArchitecture();
        
        detectedPlatform = os + "-" + arch;
        return detectedPlatform;
    }
    
    /**
     * Detects the current system architecture.
     * 
     * <p>This method handles various architecture name variations across different platforms:</p>
     * <ul>
     *   <li><strong>x64 architectures:</strong> amd64, x86_64, x64, ia64</li>
     *   <li><strong>ARM64 architectures:</strong> aarch64, arm64, arm (assumes 64-bit)</li>
     *   <li><strong>Default:</strong> x64 for unknown architectures</li>
     * </ul>
     * 
     * @return Architecture string ("x64", "arm64")
     */
    public static String detectArchitecture() {
        String osArch = System.getProperty("os.arch", "").toLowerCase();
        
        // Handle ARM64 architectures first (most specific)
        if (osArch.contains("aarch64") || osArch.contains("arm64")) {
            return "arm64";
        }
        
        // Handle other ARM architectures (assume ARM64 for modern systems)
        if (osArch.contains("arm")) {
            return "arm64";
        }
        
        // Handle x64 architectures
        if (osArch.contains("amd64") || osArch.contains("x86_64") || osArch.contains("x64")) {
            return "x64";
        }
        
        // Handle Intel Itanium (ia64) - treat as x64 for compatibility
        if (osArch.contains("ia64")) {
            return "x64";
        }
        
        // Handle 32-bit architectures (default to x64 for compatibility)
        if (osArch.contains("x86") || osArch.contains("i386") || osArch.contains("i686")) {
            return "x64"; // Default to x64 for compatibility
        }
        
        // Default to x64 for unknown architectures
        return "x64";
    }
    
    /**
     * Attempts to load the native library using multiple fallback strategies.
     * 
     * <p>This method implements a comprehensive fallback strategy that tries multiple
     * approaches to load the native library, in order of preference:</p>
     * 
     * <ol>
     *   <li><strong>Bundled Library (Organized Structure):</strong> Load from organized JAR resources (/native/{os}/{arch}/)</li>
     *   <li><strong>Bundled Library (Legacy Structure):</strong> Load from legacy JAR resources (root level)</li>
     *   <li><strong>System Library (Exact Name):</strong> Load using the platform-specific library name</li>
     *   <li><strong>System Library (Generic Name):</strong> Load using the generic "overdrive" name</li>
     *   <li><strong>Alternative Library Names:</strong> Try common naming variations</li>
     *   <li><strong>Library Path Search:</strong> Search common library installation paths</li>
     *   <li><strong>Environment Variable Paths:</strong> Check custom library paths from environment variables</li>
     *   <li><strong>Classpath Resources:</strong> Try loading from classpath with different naming patterns</li>
     * </ol>
     * 
     * <p>Each fallback strategy is attempted with proper error handling and logging.
     * The method returns the first successfully loaded library interface.</p>
     * 
     * @param libraryName The primary library name for the detected platform
     * @param platform The detected platform string (e.g., "windows-x64")
     * @return The loaded library interface, or null if all fallback strategies failed
     */
    private static OverDrive.LibOverDrive tryLoadWithFallbacks(String libraryName, String platform) {
        List<String> attemptedMethods = new ArrayList<>();
        Exception lastException = null;
        
        // Strategy 1: Try bundled library from organized structure
        try {
            OverDrive.LibOverDrive library = tryLoadBundled(libraryName, platform);
            if (library != null) {
                logSuccessfulLoad("Bundled library (organized structure)", libraryName, platform);
                return library;
            }
            attemptedMethods.add("Bundled library (organized structure)");
        } catch (Exception e) {
            lastException = e;
            attemptedMethods.add("Bundled library (organized structure) - failed: " + e.getMessage());
        }
        
        // Strategy 2: Try bundled library from legacy structure (root level)
        try {
            OverDrive.LibOverDrive library = tryLoadBundledLegacy(libraryName);
            if (library != null) {
                logSuccessfulLoad("Bundled library (legacy structure)", libraryName, platform);
                return library;
            }
            attemptedMethods.add("Bundled library (legacy structure)");
        } catch (Exception e) {
            lastException = e;
            attemptedMethods.add("Bundled library (legacy structure) - failed: " + e.getMessage());
        }
        
        // Strategy 3: Try system library with platform-specific name
        try {
            String systemLibraryName = getSystemLibraryName(libraryName);
            OverDrive.LibOverDrive library = tryLoadSystemLibrary(systemLibraryName);
            if (library != null) {
                logSuccessfulLoad("System library (platform-specific)", systemLibraryName, platform);
                return library;
            }
            attemptedMethods.add("System library (" + systemLibraryName + ")");
        } catch (Exception e) {
            lastException = e;
            attemptedMethods.add("System library (" + getSystemLibraryName(libraryName) + ") - failed: " + e.getMessage());
        }
        
        // Strategy 4: Try system library with generic name
        try {
            OverDrive.LibOverDrive library = tryLoadSystemLibrary("overdrive");
            if (library != null) {
                logSuccessfulLoad("System library (generic)", "overdrive", platform);
                return library;
            }
            attemptedMethods.add("System library (overdrive)");
        } catch (Exception e) {
            lastException = e;
            attemptedMethods.add("System library (overdrive) - failed: " + e.getMessage());
        }
        
        // Strategy 5: Try alternative library names
        try {
            OverDrive.LibOverDrive library = tryAlternativeLibraryNames(platform);
            if (library != null) {
                logSuccessfulLoad("Alternative library names", "various", platform);
                return library;
            }
            attemptedMethods.add("Alternative library names");
        } catch (Exception e) {
            lastException = e;
            attemptedMethods.add("Alternative library names - failed: " + e.getMessage());
        }
        
        // Strategy 6: Try common library paths
        try {
            OverDrive.LibOverDrive library = tryCommonLibraryPaths(libraryName, platform);
            if (library != null) {
                logSuccessfulLoad("Common library paths", libraryName, platform);
                return library;
            }
            attemptedMethods.add("Common library paths");
        } catch (Exception e) {
            lastException = e;
            attemptedMethods.add("Common library paths - failed: " + e.getMessage());
        }
        
        // Strategy 7: Try environment variable paths
        try {
            OverDrive.LibOverDrive library = tryEnvironmentVariablePaths(libraryName, platform);
            if (library != null) {
                logSuccessfulLoad("Environment variable paths", libraryName, platform);
                return library;
            }
            attemptedMethods.add("Environment variable paths");
        } catch (Exception e) {
            lastException = e;
            attemptedMethods.add("Environment variable paths - failed: " + e.getMessage());
        }
        
        // Strategy 8: Try classpath resources with different patterns
        try {
            OverDrive.LibOverDrive library = tryClasspathResourcePatterns(libraryName, platform);
            if (library != null) {
                logSuccessfulLoad("Classpath resource patterns", libraryName, platform);
                return library;
            }
            attemptedMethods.add("Classpath resource patterns");
        } catch (Exception e) {
            lastException = e;
            attemptedMethods.add("Classpath resource patterns - failed: " + e.getMessage());
        }
        
        // Log all attempted methods for debugging
        logFailedAttempts(attemptedMethods, lastException, platform);
        
        return null;
    }
    
    /**
     * Attempts to load a bundled library from the legacy structure (root level in JAR).
     * 
     * <p>This method provides backward compatibility with older library packaging
     * that placed native libraries at the root level of the JAR file.</p>
     * 
     * @param libraryName The name of the library file
     * @return The loaded library interface, or null if loading failed
     */
    private static OverDrive.LibOverDrive tryLoadBundledLegacy(String libraryName) {
        try {
            String resourcePath = "/" + libraryName;
            InputStream resourceStream = NativeLibraryLoader.class.getResourceAsStream(resourcePath);
            
            if (resourceStream == null) {
                return null;
            }
            
            // Extract library to secure temporary directory
            Path tempLibraryPath = extractLibraryToTempDirectory(resourceStream, libraryName, "legacy");
            
            if (tempLibraryPath == null) {
                return null;
            }
            
            // Load the library using the absolute path
            return Native.load(tempLibraryPath.toAbsolutePath().toString(), OverDrive.LibOverDrive.class);
            
        } catch (Exception e) {
            return null;
        }
    }
    
    /**
     * Attempts to load a system library with the specified name.
     * 
     * <p>This method tries to load a library from the system library path
     * using JNA's standard library loading mechanism.</p>
     * 
     * @param libraryName The name of the library to load
     * @return The loaded library interface, or null if loading failed
     */
    private static OverDrive.LibOverDrive tryLoadSystemLibrary(String libraryName) {
        try {
            return Native.load(libraryName, OverDrive.LibOverDrive.class);
        } catch (UnsatisfiedLinkError e) {
            return null;
        } catch (Exception e) {
            return null;
        }
    }
    
    /**
     * Gets the system library name by removing platform-specific prefixes and suffixes.
     * 
     * <p>This method converts platform-specific library names to their base names
     * that might be used in system library paths:</p>
     * 
     * <ul>
     *   <li>liboverdrive-linux-x64.so → overdrive</li>
     *   <li>liboverdrive-macos-x64.dylib → overdrive</li>
     *   <li>overdrive.dll → overdrive</li>
     * </ul>
     * 
     * @param libraryName The platform-specific library name
     * @return The base library name for system loading
     */
    private static String getSystemLibraryName(String libraryName) {
        if (libraryName == null) {
            return "overdrive";
        }
        
        // Remove common prefixes
        String baseName = libraryName;
        if (baseName.startsWith("lib")) {
            baseName = baseName.substring(3);
        }
        
        // Remove platform-specific suffixes
        baseName = baseName.replaceAll("-linux-x64", "");
        baseName = baseName.replaceAll("-linux-arm64", "");
        baseName = baseName.replaceAll("-macos-x64", "");
        baseName = baseName.replaceAll("-macos-arm64", "");
        
        // Remove file extensions
        baseName = baseName.replaceAll("\\.(dll|so|dylib)$", "");
        
        return baseName.isEmpty() ? "overdrive" : baseName;
    }
    
    /**
     * Attempts to load the library using alternative naming conventions.
     * 
     * <p>This method tries various naming conventions that might be used
     * for the OverDrive library in different environments:</p>
     * 
     * <ul>
     *   <li>overdrive-db</li>
     *   <li>overdrivedb</li>
     *   <li>liboverdrive</li>
     *   <li>overdrive_native</li>
     *   <li>Platform-specific variations</li>
     * </ul>
     * 
     * @param platform The detected platform string
     * @return The loaded library interface, or null if all alternatives failed
     */
    private static OverDrive.LibOverDrive tryAlternativeLibraryNames(String platform) {
        String[] alternativeNames = {
            "overdrive-db",
            "overdrivedb", 
            "liboverdrive",
            "overdrive_native",
            "overdrive_lib",
            "odb",
            "odb_native"
        };
        
        for (String altName : alternativeNames) {
            try {
                OverDrive.LibOverDrive library = tryLoadSystemLibrary(altName);
                if (library != null) {
                    return library;
                }
            } catch (Exception e) {
                // Continue to next alternative
            }
        }
        
        // Try platform-specific alternative names
        if (platform != null) {
            String[] platformSpecificNames = getPlatformSpecificAlternatives(platform);
            for (String altName : platformSpecificNames) {
                try {
                    OverDrive.LibOverDrive library = tryLoadSystemLibrary(altName);
                    if (library != null) {
                        return library;
                    }
                } catch (Exception e) {
                    // Continue to next alternative
                }
            }
        }
        
        return null;
    }
    
    /**
     * Gets platform-specific alternative library names.
     * 
     * @param platform The detected platform string
     * @return Array of platform-specific alternative names to try
     */
    private static String[] getPlatformSpecificAlternatives(String platform) {
        if (platform.startsWith("windows")) {
            return new String[] {
                "overdrive64",
                "overdrive_x64",
                "liboverdrive64"
            };
        } else if (platform.startsWith("linux")) {
            return new String[] {
                "overdrive-linux",
                "liboverdrive-linux",
                "overdrive_linux"
            };
        } else if (platform.startsWith("macos")) {
            return new String[] {
                "overdrive-macos",
                "liboverdrive-macos", 
                "overdrive_macos",
                "overdrive-darwin",
                "liboverdrive-darwin"
            };
        }
        
        return new String[0];
    }
    
    /**
     * Attempts to load the library from common installation paths.
     * 
     * <p>This method searches for the library in standard installation locations
     * where native libraries are commonly installed on each platform:</p>
     * 
     * <ul>
     *   <li><strong>Windows:</strong> System32, Program Files, local directories</li>
     *   <li><strong>Linux:</strong> /usr/lib, /usr/local/lib, /lib, LD_LIBRARY_PATH</li>
     *   <li><strong>macOS:</strong> /usr/lib, /usr/local/lib, /opt/local/lib, DYLD_LIBRARY_PATH</li>
     * </ul>
     * 
     * @param libraryName The library name to search for
     * @param platform The detected platform string
     * @return The loaded library interface, or null if not found in common paths
     */
    private static OverDrive.LibOverDrive tryCommonLibraryPaths(String libraryName, String platform) {
        List<String> searchPaths = getCommonLibraryPaths(platform);
        
        for (String searchPath : searchPaths) {
            try {
                Path libraryPath = Paths.get(searchPath, libraryName);
                if (Files.exists(libraryPath) && Files.isReadable(libraryPath)) {
                    OverDrive.LibOverDrive library = Native.load(libraryPath.toAbsolutePath().toString(), OverDrive.LibOverDrive.class);
                    if (library != null) {
                        return library;
                    }
                }
            } catch (Exception e) {
                // Continue to next path
            }
        }
        
        return null;
    }
    
    /**
     * Attempts to load the library from environment variable specified paths.
     * 
     * <p>This method checks for custom library paths specified in environment variables
     * that users might set to override default library locations:</p>
     * 
     * <ul>
     *   <li><strong>OVERDRIVE_LIBRARY_PATH:</strong> Custom path to OverDrive library directory</li>
     *   <li><strong>JAVA_LIBRARY_PATH:</strong> Additional Java library search paths</li>
     *   <li><strong>Platform-specific paths:</strong> LD_LIBRARY_PATH, DYLD_LIBRARY_PATH, PATH</li>
     * </ul>
     * 
     * @param libraryName The library name to search for
     * @param platform The detected platform string
     * @return The loaded library interface, or null if not found in environment paths
     */
    private static OverDrive.LibOverDrive tryEnvironmentVariablePaths(String libraryName, String platform) {
        // Check OVERDRIVE_LIBRARY_PATH first (most specific)
        String overdriveLibPath = System.getenv("OVERDRIVE_LIBRARY_PATH");
        if (overdriveLibPath != null && !overdriveLibPath.trim().isEmpty()) {
            OverDrive.LibOverDrive library = tryLoadFromPath(overdriveLibPath, libraryName);
            if (library != null) {
                return library;
            }
        }
        
        // Check JAVA_LIBRARY_PATH system property
        String javaLibPath = System.getProperty("java.library.path");
        if (javaLibPath != null && !javaLibPath.trim().isEmpty()) {
            String[] paths = javaLibPath.split(System.getProperty("path.separator"));
            for (String path : paths) {
                if (!path.trim().isEmpty()) {
                    OverDrive.LibOverDrive library = tryLoadFromPath(path.trim(), libraryName);
                    if (library != null) {
                        return library;
                    }
                }
            }
        }
        
        // Check platform-specific environment variables
        List<String> envVars = getPlatformSpecificLibraryEnvVars(platform);
        for (String envVar : envVars) {
            String envValue = System.getenv(envVar);
            if (envValue != null && !envValue.trim().isEmpty()) {
                String[] paths = envValue.split(getPlatformPathSeparator(platform));
                for (String path : paths) {
                    if (!path.trim().isEmpty()) {
                        OverDrive.LibOverDrive library = tryLoadFromPath(path.trim(), libraryName);
                        if (library != null) {
                            return library;
                        }
                    }
                }
            }
        }
        
        return null;
    }
    
    /**
     * Attempts to load the library using different classpath resource patterns.
     * 
     * <p>This method tries various resource path patterns that might be used
     * in different packaging scenarios:</p>
     * 
     * <ul>
     *   <li>Platform-specific subdirectories with different naming conventions</li>
     *   <li>Versioned library names</li>
     *   <li>Different resource root paths</li>
     *   <li>Compressed/archived library formats</li>
     * </ul>
     * 
     * @param libraryName The library name to search for
     * @param platform The detected platform string
     * @return The loaded library interface, or null if not found in classpath resources
     */
    private static OverDrive.LibOverDrive tryClasspathResourcePatterns(String libraryName, String platform) {
        String[] platformParts = platform.split("-");
        if (platformParts.length != 2) {
            return null;
        }
        String os = platformParts[0];
        String arch = platformParts[1];
        
        // Try different resource path patterns
        String[] resourcePatterns = {
            // Standard patterns
            "/lib/" + platform + "/" + libraryName,
            "/libs/" + platform + "/" + libraryName,
            "/native-libs/" + platform + "/" + libraryName,
            
            // OS-specific patterns
            "/lib/" + os + "/" + libraryName,
            "/libs/" + os + "/" + libraryName,
            "/native/" + os + "/" + libraryName,
            
            // Architecture-specific patterns
            "/lib/" + arch + "/" + libraryName,
            "/libs/" + arch + "/" + libraryName,
            
            // Versioned patterns
            "/native/v1.4/" + platform + "/" + libraryName,
            "/lib/1.4/" + platform + "/" + libraryName,
            
            // Alternative naming patterns
            "/resources/native/" + platform + "/" + libraryName,
            "/META-INF/native/" + platform + "/" + libraryName,
            
            // Flat structure alternatives
            "/" + platform + "-" + libraryName,
            "/lib-" + platform + "/" + libraryName
        };
        
        for (String resourcePath : resourcePatterns) {
            try {
                InputStream resourceStream = NativeLibraryLoader.class.getResourceAsStream(resourcePath);
                if (resourceStream != null) {
                    Path tempLibraryPath = extractLibraryToTempDirectory(resourceStream, libraryName, platform);
                    if (tempLibraryPath != null) {
                        OverDrive.LibOverDrive library = Native.load(tempLibraryPath.toAbsolutePath().toString(), OverDrive.LibOverDrive.class);
                        if (library != null) {
                            return library;
                        }
                    }
                }
            } catch (Exception e) {
                // Continue to next pattern
            }
        }
        
        return null;
    }
    
    /**
     * Attempts to load a library from a specific directory path.
     * 
     * @param directoryPath The directory path to search in
     * @param libraryName The library file name
     * @return The loaded library interface, or null if loading failed
     */
    private static OverDrive.LibOverDrive tryLoadFromPath(String directoryPath, String libraryName) {
        try {
            Path libraryPath = Paths.get(directoryPath, libraryName);
            if (Files.exists(libraryPath) && Files.isReadable(libraryPath)) {
                return Native.load(libraryPath.toAbsolutePath().toString(), OverDrive.LibOverDrive.class);
            }
        } catch (Exception e) {
            // Loading failed, continue
        }
        return null;
    }
    
    /**
     * Gets platform-specific library environment variables.
     * 
     * @param platform The detected platform string
     * @return List of environment variable names to check
     */
    private static List<String> getPlatformSpecificLibraryEnvVars(String platform) {
        List<String> envVars = new ArrayList<>();
        
        if (platform.startsWith("windows")) {
            envVars.add("PATH");
        } else if (platform.startsWith("linux")) {
            envVars.add("LD_LIBRARY_PATH");
            envVars.add("LD_RUN_PATH");
        } else if (platform.startsWith("macos")) {
            envVars.add("DYLD_LIBRARY_PATH");
            envVars.add("DYLD_FALLBACK_LIBRARY_PATH");
        }
        
        return envVars;
    }
    
    /**
     * Gets the platform-specific path separator.
     * 
     * @param platform The detected platform string
     * @return The path separator character for the platform
     */
    private static String getPlatformPathSeparator(String platform) {
        if (platform.startsWith("windows")) {
            return ";";
        } else {
            return ":";
        }
    }
    
    /**
     * Logs successful library loading for debugging and monitoring.
     * 
     * @param strategy The successful loading strategy
     * @param libraryName The loaded library name
     * @param platform The detected platform
     */
    private static void logSuccessfulLoad(String strategy, String libraryName, String platform) {
        if (Boolean.parseBoolean(System.getProperty("overdrive.debug.library.loading", "false"))) {
            System.out.println("[OverDrive] Successfully loaded native library using: " + strategy + 
                             " (library: " + libraryName + ", platform: " + platform + ")");
        }
    }
    
    /**
     * Logs failed loading attempts for debugging and troubleshooting.
     * 
     * @param attemptedMethods List of attempted loading methods
     * @param lastException The last exception encountered
     * @param platform The detected platform
     */
    private static void logFailedAttempts(List<String> attemptedMethods, Exception lastException, String platform) {
        if (Boolean.parseBoolean(System.getProperty("overdrive.debug.library.loading", "false"))) {
            System.err.println("[OverDrive] All fallback strategies failed for platform: " + platform);
            System.err.println("[OverDrive] Attempted methods:");
            for (String method : attemptedMethods) {
                System.err.println("  - " + method);
            }
            
            if (lastException != null) {
                System.err.println("[OverDrive] Last exception: " + lastException.getMessage());
            }
            
            // Provide troubleshooting suggestions
            System.err.println("[OverDrive] Troubleshooting suggestions:");
            System.err.println("  - Set OVERDRIVE_LIBRARY_PATH environment variable to custom library directory");
            System.err.println("  - Ensure native library is in system PATH or library path");
            System.err.println("  - Verify platform support: " + getSupportedPlatforms());
            System.err.println("  - Enable debug logging: -Doverdrive.debug.library.loading=true");
        }
    }
    /**
     * Gets common library search paths for the specified platform.
     * 
     * @param platform The detected platform string
     * @return List of common library paths to search
     */
    private static List<String> getCommonLibraryPaths(String platform) {
        List<String> paths = new ArrayList<>();
        
        if (platform.startsWith("windows")) {
            // Windows common library paths
            paths.add(System.getenv("WINDIR") + "\\System32");
            paths.add(System.getenv("WINDIR") + "\\SysWOW64");
            paths.add(System.getProperty("user.dir"));
            paths.add(".");
            
            String programFiles = System.getenv("ProgramFiles");
            if (programFiles != null) {
                paths.add(programFiles + "\\OverDrive");
                paths.add(programFiles + "\\OverDrive-DB");
            }
            
            String programFilesX86 = System.getenv("ProgramFiles(x86)");
            if (programFilesX86 != null) {
                paths.add(programFilesX86 + "\\OverDrive");
                paths.add(programFilesX86 + "\\OverDrive-DB");
            }
            
        } else if (platform.startsWith("linux")) {
            // Linux common library paths
            paths.add("/usr/lib");
            paths.add("/usr/local/lib");
            paths.add("/lib");
            paths.add("/usr/lib64");
            paths.add("/usr/local/lib64");
            paths.add("/lib64");
            paths.add(System.getProperty("user.dir"));
            paths.add(".");
            
            // Add LD_LIBRARY_PATH directories
            String ldLibraryPath = System.getenv("LD_LIBRARY_PATH");
            if (ldLibraryPath != null) {
                String[] ldPaths = ldLibraryPath.split(":");
                for (String ldPath : ldPaths) {
                    if (!ldPath.trim().isEmpty()) {
                        paths.add(ldPath.trim());
                    }
                }
            }
            
        } else if (platform.startsWith("macos")) {
            // macOS common library paths
            paths.add("/usr/lib");
            paths.add("/usr/local/lib");
            paths.add("/opt/local/lib");
            paths.add("/System/Library/Frameworks");
            paths.add("/Library/Frameworks");
            paths.add(System.getProperty("user.dir"));
            paths.add(".");
            
            // Add DYLD_LIBRARY_PATH directories
            String dyldLibraryPath = System.getenv("DYLD_LIBRARY_PATH");
            if (dyldLibraryPath != null) {
                String[] dyldPaths = dyldLibraryPath.split(":");
                for (String dyldPath : dyldPaths) {
                    if (!dyldPath.trim().isEmpty()) {
                        paths.add(dyldPath.trim());
                    }
                }
            }
        }
        
        // Filter out null and non-existent paths
        return paths.stream()
            .filter(Objects::nonNull)
            .filter(path -> {
                try {
                    return Files.exists(Paths.get(path)) && Files.isDirectory(Paths.get(path));
                } catch (Exception e) {
                    return false;
                }
            })
            .collect(Collectors.toList());
    }

    /**
     * Attempts to load a bundled native library from JAR resources.
     * 
     * <p>This method extracts the appropriate native library from the organized
     * JAR resource structure (/native/{os}/{arch}/{library}) to a secure temporary
     * directory and loads it using JNA.</p>
     * 
     * <p>The extraction process:</p>
     * <ol>
     *   <li>Creates a secure temporary directory with restricted permissions</li>
     *   <li>Locates the library resource using the organized path structure</li>
     *   <li>Extracts the library to the temporary directory</li>
     *   <li>Sets appropriate file permissions (executable on Unix-like systems)</li>
     *   <li>Registers cleanup hooks for temporary file management</li>
     *   <li>Loads the library using the absolute path</li>
     * </ol>
     * 
     * @param libraryName The name of the library file (e.g., "overdrive.dll")
     * @param platform The detected platform string (e.g., "windows-x64")
     * @return The loaded library interface, or null if loading failed
     */
    private static OverDrive.LibOverDrive tryLoadBundled(String libraryName, String platform) {
        try {
            // Parse platform to get OS and architecture
            String[] platformParts = platform.split("-");
            if (platformParts.length != 2) {
                return null;
            }
            String os = platformParts[0];
            String arch = platformParts[1];
            
            // Construct the organized resource path: /native/{os}/{arch}/{library}
            String resourcePath = "/native/" + os + "/" + arch + "/" + libraryName;
            
            // Try to find the library in resources
            InputStream resourceStream = NativeLibraryLoader.class.getResourceAsStream(resourcePath);
            
            if (resourceStream == null) {
                // Library not found in organized structure, try legacy root location
                String legacyResourcePath = "/" + libraryName;
                resourceStream = NativeLibraryLoader.class.getResourceAsStream(legacyResourcePath);
                
                if (resourceStream == null) {
                    // Library not found in either location
                    return null;
                }
            }
            
            // Extract library to secure temporary directory
            Path tempLibraryPath = extractLibraryToTempDirectory(resourceStream, libraryName, platform);
            
            if (tempLibraryPath == null) {
                return null;
            }
            
            // Load the library using the absolute path
            return Native.load(tempLibraryPath.toAbsolutePath().toString(), OverDrive.LibOverDrive.class);
            
        } catch (Exception e) {
            // Bundled loading failed, will try system loading
            return null;
        }
    }
    
    /**
     * Extracts a native library from JAR resources to a secure temporary directory.
     * 
     * <p>This method handles the secure extraction process:</p>
     * <ul>
     *   <li>Creates a temporary directory with platform-appropriate security</li>
     *   <li>Copies the library resource to the temporary location</li>
     *   <li>Sets proper file permissions for execution</li>
     *   <li>Registers cleanup hooks for resource management</li>
     * </ul>
     * 
     * @param resourceStream The input stream of the library resource
     * @param libraryName The name of the library file
     * @param platform The platform string for error reporting
     * @return The path to the extracted library, or null if extraction failed
     */
    private static Path extractLibraryToTempDirectory(InputStream resourceStream, String libraryName, String platform) {
        try {
            // Create secure temporary directory
            Path tempDir = createSecureTempDirectory();
            
            // Extract library to temp directory
            Path tempLibrary = tempDir.resolve(libraryName);
            
            try {
                Files.copy(resourceStream, tempLibrary, StandardCopyOption.REPLACE_EXISTING);
            } finally {
                // Always close the resource stream
                try {
                    resourceStream.close();
                } catch (IOException e) {
                    // Ignore close errors
                }
            }
            
            // Set executable permissions on the extracted library
            try {
                setLibraryPermissions(tempLibrary);
            } catch (IOException e) {
                // Permission setting failed - this is critical for library loading
                // Clean up the extracted file and directory
                try {
                    Files.deleteIfExists(tempLibrary);
                    Files.deleteIfExists(tempDir);
                } catch (IOException cleanupException) {
                    // Ignore cleanup errors
                }
                return null;
            }
            
            // Register cleanup hook for the temporary directory
            registerCleanupHook(tempDir);
            
            return tempLibrary;
            
        } catch (Exception e) {
            // Extraction failed
            return null;
        }
    }
    
    /**
     * Creates a secure temporary directory for native library extraction.
     * 
     * <p>The directory is created with restricted permissions to prevent
     * unauthorized access to the extracted native libraries. This method
     * implements platform-specific security measures:</p>
     * 
     * <ul>
     *   <li><strong>Unix-like systems:</strong> Sets POSIX permissions to owner-only (700)</li>
     *   <li><strong>Windows:</strong> Uses secure temp directory location and sets file attributes</li>
     *   <li><strong>All platforms:</strong> Validates temp directory location and registers cleanup hooks</li>
     * </ul>
     * 
     * @return Path to the created temporary directory
     * @throws IOException if directory creation fails or security measures cannot be applied
     */
    private static Path createSecureTempDirectory() throws IOException {
        // Get system temp directory and validate it's secure
        Path systemTempDir = validateSystemTempDirectory();
        
        // Create temporary directory with secure prefix
        Path tempDir;
        try {
            tempDir = Files.createTempDirectory(systemTempDir, TEMP_DIR_PREFIX);
        } catch (IOException e) {
            throw new IOException("Failed to create secure temporary directory: " + e.getMessage(), e);
        }
        
        // Apply platform-specific security measures
        applySecurityMeasures(tempDir);
        
        // Ensure directory is deleted on JVM exit
        tempDir.toFile().deleteOnExit();
        
        return tempDir;
    }
    
    /**
     * Validates that the system temporary directory is secure and appropriate for use.
     * 
     * <p>This method performs security checks on the system temp directory to ensure
     * it's safe to create temporary files containing native libraries.</p>
     * 
     * @return Path to the validated system temporary directory
     * @throws IOException if the system temp directory is not secure or accessible
     */
    private static Path validateSystemTempDirectory() throws IOException {
        String tempDirProperty = System.getProperty("java.io.tmpdir");
        if (tempDirProperty == null || tempDirProperty.trim().isEmpty()) {
            throw new IOException("System temporary directory property (java.io.tmpdir) is not set");
        }
        
        Path systemTempDir = Paths.get(tempDirProperty);
        
        // Verify the temp directory exists and is accessible
        if (!Files.exists(systemTempDir)) {
            throw new IOException("System temporary directory does not exist: " + systemTempDir);
        }
        
        if (!Files.isDirectory(systemTempDir)) {
            throw new IOException("System temporary directory is not a directory: " + systemTempDir);
        }
        
        if (!Files.isWritable(systemTempDir)) {
            throw new IOException("System temporary directory is not writable: " + systemTempDir);
        }
        
        // Additional security check: ensure temp directory is not world-writable on Unix-like systems
        if (!isWindows()) {
            try {
                Set<java.nio.file.attribute.PosixFilePermission> permissions = 
                    Files.getPosixFilePermissions(systemTempDir);
                
                // Check if directory is world-writable (security risk)
                if (permissions.contains(java.nio.file.attribute.PosixFilePermission.OTHERS_WRITE)) {
                    // Log warning but don't fail - some systems have world-writable /tmp
                    System.err.println("WARNING: System temporary directory is world-writable: " + systemTempDir);
                }
            } catch (Exception e) {
                // Ignore POSIX permission check errors on non-POSIX systems
            }
        }
        
        return systemTempDir;
    }
    
    /**
     * Applies platform-specific security measures to the temporary directory.
     * 
     * <p>This method implements comprehensive security measures appropriate for each platform:</p>
     * 
     * <ul>
     *   <li><strong>Unix-like systems:</strong> Sets POSIX permissions to 700 (owner read/write/execute only)</li>
     *   <li><strong>Windows:</strong> Sets hidden and system attributes, attempts to restrict access</li>
     *   <li><strong>All platforms:</strong> Validates that security measures were applied successfully</li>
     * </ul>
     * 
     * @param tempDir The temporary directory to secure
     * @throws IOException if critical security measures cannot be applied
     */
    private static void applySecurityMeasures(Path tempDir) throws IOException {
        if (isWindows()) {
            applyWindowsSecurityMeasures(tempDir);
        } else {
            applyUnixSecurityMeasures(tempDir);
        }
        
        // Verify the directory is still accessible after applying security measures
        if (!Files.isReadable(tempDir) || !Files.isWritable(tempDir)) {
            throw new IOException("Temporary directory became inaccessible after applying security measures: " + tempDir);
        }
    }
    
    /**
     * Applies Unix-specific security measures using POSIX file permissions.
     * 
     * <p>Sets directory permissions to 700 (owner read/write/execute only) to prevent
     * unauthorized access from other users on the system.</p>
     * 
     * @param tempDir The temporary directory to secure
     * @throws IOException if POSIX permissions cannot be set and the system supports them
     */
    private static void applyUnixSecurityMeasures(Path tempDir) throws IOException {
        try {
            // Set restrictive POSIX permissions: owner read/write/execute only (700)
            Set<java.nio.file.attribute.PosixFilePermission> permissions = EnumSet.of(
                java.nio.file.attribute.PosixFilePermission.OWNER_READ,
                java.nio.file.attribute.PosixFilePermission.OWNER_WRITE,
                java.nio.file.attribute.PosixFilePermission.OWNER_EXECUTE
            );
            Files.setPosixFilePermissions(tempDir, permissions);
            
            // Verify permissions were set correctly
            Set<java.nio.file.attribute.PosixFilePermission> actualPermissions = 
                Files.getPosixFilePermissions(tempDir);
            
            if (!actualPermissions.equals(permissions)) {
                throw new IOException("Failed to set secure POSIX permissions on temporary directory: " + tempDir);
            }
            
        } catch (UnsupportedOperationException e) {
            // POSIX permissions not supported on this file system
            // Fall back to basic Java file permissions
            File tempDirFile = tempDir.toFile();
            if (!tempDirFile.setReadable(true, true) || 
                !tempDirFile.setWritable(true, true) || 
                !tempDirFile.setExecutable(true, true)) {
                throw new IOException("Failed to set secure file permissions on temporary directory: " + tempDir);
            }
        } catch (IOException e) {
            // Re-throw IOException as-is
            throw e;
        } catch (Exception e) {
            // Handle other exceptions (SecurityException, etc.)
            throw new IOException("Failed to apply Unix security measures to temporary directory: " + e.getMessage(), e);
        }
    }
    
    /**
     * Applies Windows-specific security measures using file attributes and ACLs.
     * 
     * <p>On Windows, this method attempts to:</p>
     * <ul>
     *   <li>Set the directory as hidden to reduce visibility</li>
     *   <li>Apply restrictive access control where possible</li>
     *   <li>Use Windows-specific security features when available</li>
     * </ul>
     * 
     * @param tempDir The temporary directory to secure
     * @throws IOException if critical Windows security measures cannot be applied
     */
    private static void applyWindowsSecurityMeasures(Path tempDir) throws IOException {
        try {
            File tempDirFile = tempDir.toFile();
            
            // Set basic file permissions (owner only)
            if (!tempDirFile.setReadable(true, true) || 
                !tempDirFile.setWritable(true, true) || 
                !tempDirFile.setExecutable(true, true)) {
                throw new IOException("Failed to set basic file permissions on Windows temporary directory: " + tempDir);
            }
            
            // Try to set Windows-specific attributes
            try {
                // Set hidden attribute to reduce visibility
                Files.setAttribute(tempDir, "dos:hidden", true);
            } catch (Exception e) {
                // Hidden attribute is not critical, continue without it
            }
            
            // Try to apply Windows ACL restrictions if available
            try {
                // This is a best-effort attempt to restrict access using Windows ACLs
                // The exact implementation would require Windows-specific libraries
                // For now, we rely on the basic file permissions set above
                applyWindowsACLRestrictions(tempDir);
            } catch (Exception e) {
                // ACL restrictions are not critical for basic security
                // The basic file permissions provide reasonable protection
            }
            
        } catch (IOException e) {
            // Re-throw IOException as-is
            throw e;
        } catch (Exception e) {
            // Handle other exceptions
            throw new IOException("Failed to apply Windows security measures to temporary directory: " + e.getMessage(), e);
        }
    }
    
    /**
     * Attempts to apply Windows Access Control List (ACL) restrictions.
     * 
     * <p>This is a best-effort method that tries to restrict access to the temporary
     * directory using Windows-specific security features. If ACL manipulation is not
     * available or fails, the method fails gracefully.</p>
     * 
     * @param tempDir The temporary directory to apply ACL restrictions to
     */
    private static void applyWindowsACLRestrictions(Path tempDir) {
        // This method provides a placeholder for Windows ACL restrictions
        // In a production implementation, this could use:
        // - Windows-specific JNI calls
        // - Third-party libraries like jna-platform for Windows ACL manipulation
        // - Process execution of icacls or similar Windows tools
        
        // For now, we rely on the basic file permissions which provide
        // reasonable security for most use cases
        
        try {
            // Attempt to use Windows-specific file attributes if available
            // This is a simplified approach that works with standard Java APIs
            
            // Try to set the directory as system and hidden for additional obscurity
            Files.setAttribute(tempDir, "dos:system", true);
            
        } catch (Exception e) {
            // ACL restrictions are best-effort only
            // Basic file permissions are sufficient for most security requirements
        }
    }
    
    /**
     * Sets appropriate permissions on the extracted native library.
     * 
     * <p>This method applies platform-specific security measures to the extracted
     * native library file to ensure it has proper permissions for execution while
     * maintaining security:</p>
     * 
     * <ul>
     *   <li><strong>Unix-like systems:</strong> Sets POSIX permissions to 700 (owner read/write/execute only)</li>
     *   <li><strong>Windows:</strong> Sets appropriate file attributes and permissions</li>
     *   <li><strong>All platforms:</strong> Validates that the library is executable after permission setting</li>
     * </ul>
     * 
     * @param libraryPath Path to the extracted library file
     * @throws IOException if critical permissions cannot be set or the library becomes inaccessible
     */
    private static void setLibraryPermissions(Path libraryPath) throws IOException {
        try {
            if (isWindows()) {
                setWindowsLibraryPermissions(libraryPath);
            } else {
                setUnixLibraryPermissions(libraryPath);
            }
            
            // Verify the library is still accessible and executable after setting permissions
            if (!Files.isReadable(libraryPath)) {
                throw new IOException("Library file became unreadable after setting permissions: " + libraryPath);
            }
            
            // Check if the library file is executable (critical for native libraries)
            File libraryFile = libraryPath.toFile();
            if (!libraryFile.canExecute()) {
                throw new IOException("Library file is not executable after setting permissions: " + libraryPath);
            }
            
        } catch (IOException e) {
            // Re-throw IOException as-is
            throw e;
        } catch (Exception e) {
            // Handle other exceptions
            throw new IOException("Failed to set library permissions: " + e.getMessage(), e);
        }
    }
    
    /**
     * Sets Unix-specific permissions on the native library file.
     * 
     * <p>Uses POSIX file permissions to set the library as owner-readable,
     * owner-writable, and owner-executable (700 permissions).</p>
     * 
     * @param libraryPath Path to the library file
     * @throws IOException if POSIX permissions cannot be set on a POSIX-compliant system
     */
    private static void setUnixLibraryPermissions(Path libraryPath) throws IOException {
        try {
            // Set restrictive POSIX permissions: owner read/write/execute only (700)
            Set<java.nio.file.attribute.PosixFilePermission> permissions = EnumSet.of(
                java.nio.file.attribute.PosixFilePermission.OWNER_READ,
                java.nio.file.attribute.PosixFilePermission.OWNER_WRITE,
                java.nio.file.attribute.PosixFilePermission.OWNER_EXECUTE
            );
            Files.setPosixFilePermissions(libraryPath, permissions);
            
            // Verify permissions were set correctly
            Set<java.nio.file.attribute.PosixFilePermission> actualPermissions = 
                Files.getPosixFilePermissions(libraryPath);
            
            if (!actualPermissions.equals(permissions)) {
                throw new IOException("Failed to set secure POSIX permissions on library file: " + libraryPath);
            }
            
        } catch (UnsupportedOperationException e) {
            // POSIX permissions not supported on this file system
            // Fall back to basic Java file permissions
            setBasicLibraryPermissions(libraryPath);
        } catch (IOException e) {
            // Re-throw IOException as-is
            throw e;
        } catch (Exception e) {
            // Handle other exceptions and try fallback
            try {
                setBasicLibraryPermissions(libraryPath);
            } catch (Exception fallbackException) {
                throw new IOException("Failed to set Unix library permissions: " + e.getMessage(), e);
            }
        }
    }
    
    /**
     * Sets Windows-specific permissions on the native library file.
     * 
     * <p>Applies Windows-appropriate security measures including file attributes
     * and access permissions.</p>
     * 
     * @param libraryPath Path to the library file
     * @throws IOException if Windows permissions cannot be set
     */
    private static void setWindowsLibraryPermissions(Path libraryPath) throws IOException {
        try {
            // Set basic file permissions (owner only)
            setBasicLibraryPermissions(libraryPath);
            
            // Try to set Windows-specific attributes for additional security
            try {
                // Set the file as hidden to reduce visibility
                Files.setAttribute(libraryPath, "dos:hidden", true);
            } catch (Exception e) {
                // Hidden attribute is not critical for functionality
            }
            
            // Try to apply additional Windows security measures
            try {
                applyWindowsLibraryACL(libraryPath);
            } catch (Exception e) {
                // ACL restrictions are best-effort only
            }
            
        } catch (IOException e) {
            // Re-throw IOException as-is
            throw e;
        } catch (Exception e) {
            throw new IOException("Failed to set Windows library permissions: " + e.getMessage(), e);
        }
    }
    
    /**
     * Sets basic cross-platform file permissions using standard Java APIs.
     * 
     * <p>This method provides a fallback for systems that don't support
     * platform-specific permission mechanisms.</p>
     * 
     * @param libraryPath Path to the library file
     * @throws IOException if basic permissions cannot be set
     */
    private static void setBasicLibraryPermissions(Path libraryPath) throws IOException {
        File libraryFile = libraryPath.toFile();
        
        // Set owner-only permissions using basic Java APIs
        boolean readable = libraryFile.setReadable(true, true);   // Owner readable only
        boolean writable = libraryFile.setWritable(true, true);   // Owner writable only
        boolean executable = libraryFile.setExecutable(true, true); // Owner executable only
        
        if (!readable || !writable || !executable) {
            throw new IOException("Failed to set basic file permissions on library: " + libraryPath);
        }
    }
    
    /**
     * Attempts to apply Windows-specific Access Control List (ACL) restrictions to the library file.
     * 
     * <p>This is a best-effort method that tries to enhance security using Windows-specific
     * features. Failures are handled gracefully as the basic permissions provide adequate security.</p>
     * 
     * @param libraryPath Path to the library file
     */
    private static void applyWindowsLibraryACL(Path libraryPath) {
        // This method provides a placeholder for Windows ACL restrictions on library files
        // In a production implementation, this could use Windows-specific security APIs
        
        try {
            // Attempt to set additional Windows attributes for security
            Files.setAttribute(libraryPath, "dos:system", true);
        } catch (Exception e) {
            // ACL restrictions are best-effort only
            // Basic file permissions provide sufficient security for most use cases
        }
    }
    
    /**
     * Registers a shutdown hook to clean up temporary files.
     * 
     * <p>This method implements enhanced cleanup logic that:</p>
     * <ul>
     *   <li>Tracks all temporary directories created during library extraction</li>
     *   <li>Handles cleanup gracefully even if files are in use</li>
     *   <li>Provides logging for cleanup operations</li>
     *   <li>Ensures cleanup happens only once per JVM session</li>
     * </ul>
     * 
     * @param tempDir The temporary directory to clean up
     */
    private static void registerCleanupHook(Path tempDir) {
        // Add the directory to our cleanup tracking set
        tempDirectoriesToCleanup.add(tempDir);
        
        // Register the global cleanup hook only once
        if (!cleanupHookRegistered) {
            synchronized (NativeLibraryLoader.class) {
                if (!cleanupHookRegistered) {
                    Runtime.getRuntime().addShutdownHook(new Thread(() -> {
                        performGlobalCleanup();
                    }, "OverDrive-Cleanup"));
                    cleanupHookRegistered = true;
                    
                    logCleanupOperation("Registered global cleanup hook for temporary files", null);
                }
            }
        }
        
        // Also register individual cleanup for this directory as fallback
        tempDir.toFile().deleteOnExit();
    }
    
    /**
     * Performs comprehensive cleanup of all temporary files and directories.
     * 
     * <p>This method is called during JVM shutdown and implements robust cleanup logic:</p>
     * <ul>
     *   <li>Attempts to clean up all tracked temporary directories</li>
     *   <li>Handles files that may be in use gracefully</li>
     *   <li>Provides detailed logging of cleanup operations</li>
     *   <li>Continues cleanup even if individual operations fail</li>
     * </ul>
     */
    private static void performGlobalCleanup() {
        logCleanupOperation("Starting global cleanup of temporary files", null);
        
        int successCount = 0;
        int failureCount = 0;
        List<String> failedPaths = new ArrayList<>();
        
        // Create a copy of the set to avoid concurrent modification during cleanup
        Set<Path> directoriesToClean = new HashSet<>(tempDirectoriesToCleanup);
        
        for (Path tempDir : directoriesToClean) {
            try {
                if (Files.exists(tempDir)) {
                    boolean cleanupSuccess = cleanupTempDirectoryGracefully(tempDir);
                    if (cleanupSuccess) {
                        successCount++;
                        logCleanupOperation("Successfully cleaned up temporary directory: " + tempDir, null);
                    } else {
                        failureCount++;
                        failedPaths.add(tempDir.toString());
                        logCleanupOperation("Failed to clean up temporary directory: " + tempDir, null);
                    }
                } else {
                    // Directory already doesn't exist, consider it cleaned
                    successCount++;
                }
            } catch (Exception e) {
                failureCount++;
                failedPaths.add(tempDir.toString());
                logCleanupOperation("Exception during cleanup of " + tempDir, e);
            }
        }
        
        // Log cleanup summary
        String summary = String.format("Cleanup completed: %d successful, %d failed", successCount, failureCount);
        if (failureCount > 0) {
            summary += ". Failed paths: " + String.join(", ", failedPaths);
        }
        logCleanupOperation(summary, null);
        
        // Clear the tracking set
        tempDirectoriesToCleanup.clear();
    }
    
    /**
     * Attempts to clean up a temporary directory gracefully, handling files that may be in use.
     * 
     * <p>This method implements a multi-strategy approach to cleanup:</p>
     * <ol>
     *   <li>First attempt: Standard recursive deletion</li>
     *   <li>Second attempt: Force deletion with retry logic</li>
     *   <li>Third attempt: Mark files for deletion on next reboot (Windows)</li>
     *   <li>Fallback: Log remaining files for manual cleanup</li>
     * </ol>
     * 
     * @param tempDir The temporary directory to clean up
     * @return true if cleanup was successful, false otherwise
     */
    private static boolean cleanupTempDirectoryGracefully(Path tempDir) {
        try {
            // Strategy 1: Standard recursive deletion
            if (attemptStandardCleanup(tempDir)) {
                return true;
            }
            
            // Strategy 2: Force deletion with retry logic
            if (attemptForceCleanup(tempDir)) {
                return true;
            }
            
            // Strategy 3: Platform-specific cleanup strategies
            if (attemptPlatformSpecificCleanup(tempDir)) {
                return true;
            }
            
            // Strategy 4: Mark for future cleanup
            markForFutureCleanup(tempDir);
            return false;
            
        } catch (Exception e) {
            logCleanupOperation("Exception during graceful cleanup of " + tempDir, e);
            return false;
        }
    }
    
    /**
     * Attempts standard recursive deletion of the temporary directory.
     * 
     * @param tempDir The directory to delete
     * @return true if deletion was successful
     */
    private static boolean attemptStandardCleanup(Path tempDir) {
        try {
            deleteRecursively(tempDir);
            return !Files.exists(tempDir);
        } catch (Exception e) {
            return false;
        }
    }
    
    /**
     * Attempts force deletion with retry logic for files that may be temporarily locked.
     * 
     * @param tempDir The directory to delete
     * @return true if deletion was successful
     */
    private static boolean attemptForceCleanup(Path tempDir) {
        int maxRetries = 3;
        long retryDelayMs = 100;
        
        for (int attempt = 1; attempt <= maxRetries; attempt++) {
            try {
                // Try to release any potential file handles
                System.gc();
                
                if (attempt > 1) {
                    Thread.sleep(retryDelayMs * attempt);
                }
                
                // Attempt to change file permissions to make them deletable
                makeFilesDeleteable(tempDir);
                
                // Try deletion again
                deleteRecursively(tempDir);
                
                if (!Files.exists(tempDir)) {
                    logCleanupOperation("Force cleanup successful on attempt " + attempt + " for: " + tempDir, null);
                    return true;
                }
                
            } catch (Exception e) {
                if (attempt == maxRetries) {
                    logCleanupOperation("Force cleanup failed after " + maxRetries + " attempts for: " + tempDir, e);
                }
            }
        }
        
        return false;
    }
    
    /**
     * Attempts platform-specific cleanup strategies.
     * 
     * @param tempDir The directory to delete
     * @return true if deletion was successful
     */
    private static boolean attemptPlatformSpecificCleanup(Path tempDir) {
        try {
            if (isWindows()) {
                return attemptWindowsSpecificCleanup(tempDir);
            } else {
                return attemptUnixSpecificCleanup(tempDir);
            }
        } catch (Exception e) {
            logCleanupOperation("Platform-specific cleanup failed for: " + tempDir, e);
            return false;
        }
    }
    
    /**
     * Attempts Windows-specific cleanup strategies.
     * 
     * @param tempDir The directory to delete
     * @return true if deletion was successful
     */
    private static boolean attemptWindowsSpecificCleanup(Path tempDir) {
        try {
            // On Windows, try to use the MoveFileEx API to mark files for deletion on reboot
            // This is a fallback for files that are currently in use
            
            // First, try to remove read-only attributes that might prevent deletion
            removeWindowsReadOnlyAttributes(tempDir);
            
            // Try deletion again after attribute removal
            deleteRecursively(tempDir);
            return !Files.exists(tempDir);
            
        } catch (Exception e) {
            return false;
        }
    }
    
    /**
     * Attempts Unix-specific cleanup strategies.
     * 
     * @param tempDir The directory to delete
     * @return true if deletion was successful
     */
    private static boolean attemptUnixSpecificCleanup(Path tempDir) {
        try {
            // On Unix systems, try to change permissions to make files deletable
            makeUnixFilesDeleteable(tempDir);
            
            // Try deletion again
            deleteRecursively(tempDir);
            return !Files.exists(tempDir);
            
        } catch (Exception e) {
            return false;
        }
    }
    
    /**
     * Makes files deleteable by changing their permissions.
     * 
     * @param tempDir The directory containing files to make deleteable
     */
    private static void makeFilesDeleteable(Path tempDir) {
        try {
            if (isWindows()) {
                removeWindowsReadOnlyAttributes(tempDir);
            } else {
                makeUnixFilesDeleteable(tempDir);
            }
        } catch (Exception e) {
            // Ignore permission change errors
        }
    }
    
    /**
     * Removes read-only attributes from Windows files to make them deleteable.
     * 
     * @param tempDir The directory to process
     */
    private static void removeWindowsReadOnlyAttributes(Path tempDir) {
        try {
            Files.walk(tempDir)
                .forEach(path -> {
                    try {
                        File file = path.toFile();
                        if (file.exists()) {
                            file.setWritable(true);
                            // Try to remove read-only attribute
                            Files.setAttribute(path, "dos:readonly", false);
                        }
                    } catch (Exception e) {
                        // Ignore individual file errors
                    }
                });
        } catch (Exception e) {
            // Ignore walk errors
        }
    }
    
    /**
     * Makes Unix files deleteable by setting appropriate permissions.
     * 
     * @param tempDir The directory to process
     */
    private static void makeUnixFilesDeleteable(Path tempDir) {
        try {
            Files.walk(tempDir)
                .forEach(path -> {
                    try {
                        File file = path.toFile();
                        if (file.exists()) {
                            file.setWritable(true);
                            file.setReadable(true);
                            if (Files.isDirectory(path)) {
                                file.setExecutable(true);
                            }
                        }
                    } catch (Exception e) {
                        // Ignore individual file errors
                    }
                });
        } catch (Exception e) {
            // Ignore walk errors
        }
    }
    
    /**
     * Marks files for future cleanup when immediate deletion is not possible.
     * 
     * @param tempDir The directory to mark for future cleanup
     */
    private static void markForFutureCleanup(Path tempDir) {
        try {
            // Create a marker file indicating this directory needs cleanup
            Path markerFile = tempDir.getParent().resolve(tempDir.getFileName() + ".cleanup-needed");
            Files.write(markerFile, 
                ("Temporary directory cleanup needed: " + tempDir + "\n" +
                 "Created: " + java.time.Instant.now() + "\n" +
                 "Process: " + java.lang.management.ManagementFactory.getRuntimeMXBean().getName())
                .getBytes());
            
            logCleanupOperation("Marked for future cleanup: " + tempDir + " (marker: " + markerFile + ")", null);
            
        } catch (Exception e) {
            logCleanupOperation("Failed to create cleanup marker for: " + tempDir, e);
        }
    }
    
    /**
     * Logs cleanup operations for debugging and monitoring.
     * 
     * @param message The cleanup operation message
     * @param exception Optional exception that occurred during cleanup
     */
    private static void logCleanupOperation(String message, Exception exception) {
        boolean debugEnabled = Boolean.parseBoolean(System.getProperty("overdrive.debug.cleanup", "false"));
        
        if (debugEnabled) {
            String timestamp = java.time.Instant.now().toString();
            String logMessage = "[OverDrive-Cleanup " + timestamp + "] " + message;
            
            if (exception != null) {
                System.err.println(logMessage + " - Exception: " + exception.getMessage());
                if (Boolean.parseBoolean(System.getProperty("overdrive.debug.cleanup.stacktrace", "false"))) {
                    exception.printStackTrace();
                }
            } else {
                System.out.println(logMessage);
            }
        }
    }
    
    /**
     * Provides a manual cleanup method for applications that want to clean up immediately.
     * 
     * <p>This method allows applications to trigger cleanup of temporary files before
     * JVM shutdown, which can be useful for long-running applications or when
     * immediate cleanup is desired.</p>
     * 
     * @return CleanupResult containing details about the cleanup operation
     */
    public static CleanupResult performManualCleanup() {
        logCleanupOperation("Manual cleanup requested", null);
        
        int successCount = 0;
        int failureCount = 0;
        List<String> failedPaths = new ArrayList<>();
        List<String> cleanedPaths = new ArrayList<>();
        
        // Create a copy to avoid concurrent modification
        Set<Path> directoriesToClean = new HashSet<>(tempDirectoriesToCleanup);
        
        for (Path tempDir : directoriesToClean) {
            try {
                if (Files.exists(tempDir)) {
                    boolean cleanupSuccess = cleanupTempDirectoryGracefully(tempDir);
                    if (cleanupSuccess) {
                        successCount++;
                        cleanedPaths.add(tempDir.toString());
                        tempDirectoriesToCleanup.remove(tempDir);
                    } else {
                        failureCount++;
                        failedPaths.add(tempDir.toString());
                    }
                } else {
                    // Directory already doesn't exist
                    successCount++;
                    tempDirectoriesToCleanup.remove(tempDir);
                }
            } catch (Exception e) {
                failureCount++;
                failedPaths.add(tempDir.toString() + " (" + e.getMessage() + ")");
            }
        }
        
        CleanupResult result = new CleanupResult(successCount, failureCount, cleanedPaths, failedPaths);
        logCleanupOperation("Manual cleanup completed: " + result.getSummary(), null);
        
        return result;
    }
    
    /**
     * Gets information about temporary directories that are tracked for cleanup.
     * 
     * @return CleanupInfo containing details about tracked temporary directories
     */
    public static CleanupInfo getCleanupInfo() {
        List<String> trackedPaths = tempDirectoriesToCleanup.stream()
            .map(Path::toString)
            .collect(Collectors.toList());
        
        List<String> existingPaths = tempDirectoriesToCleanup.stream()
            .filter(Files::exists)
            .map(Path::toString)
            .collect(Collectors.toList());
        
        return new CleanupInfo(trackedPaths, existingPaths, cleanupHookRegistered);
    }
    
    /**
     * Result of a cleanup operation.
     */
    public static class CleanupResult {
        private final int successCount;
        private final int failureCount;
        private final List<String> cleanedPaths;
        private final List<String> failedPaths;
        
        public CleanupResult(int successCount, int failureCount, List<String> cleanedPaths, List<String> failedPaths) {
            this.successCount = successCount;
            this.failureCount = failureCount;
            this.cleanedPaths = new ArrayList<>(cleanedPaths);
            this.failedPaths = new ArrayList<>(failedPaths);
        }
        
        public int getSuccessCount() { return successCount; }
        public int getFailureCount() { return failureCount; }
        public List<String> getCleanedPaths() { return new ArrayList<>(cleanedPaths); }
        public List<String> getFailedPaths() { return new ArrayList<>(failedPaths); }
        
        public boolean isFullySuccessful() { return failureCount == 0; }
        public int getTotalCount() { return successCount + failureCount; }
        
        public String getSummary() {
            return String.format("%d successful, %d failed out of %d total", 
                successCount, failureCount, getTotalCount());
        }
        
        @Override
        public String toString() {
            return "CleanupResult{" + getSummary() + "}";
        }
    }
    
    /**
     * Information about cleanup state and tracked directories.
     */
    public static class CleanupInfo {
        private final List<String> trackedPaths;
        private final List<String> existingPaths;
        private final boolean cleanupHookRegistered;
        
        public CleanupInfo(List<String> trackedPaths, List<String> existingPaths, boolean cleanupHookRegistered) {
            this.trackedPaths = new ArrayList<>(trackedPaths);
            this.existingPaths = new ArrayList<>(existingPaths);
            this.cleanupHookRegistered = cleanupHookRegistered;
        }
        
        public List<String> getTrackedPaths() { return new ArrayList<>(trackedPaths); }
        public List<String> getExistingPaths() { return new ArrayList<>(existingPaths); }
        public boolean isCleanupHookRegistered() { return cleanupHookRegistered; }
        
        public int getTrackedCount() { return trackedPaths.size(); }
        public int getExistingCount() { return existingPaths.size(); }
        
        @Override
        public String toString() {
            return String.format("CleanupInfo{tracked=%d, existing=%d, hookRegistered=%s}", 
                getTrackedCount(), getExistingCount(), cleanupHookRegistered);
        }
    }
    
    /**
     * Recursively deletes a directory and all its contents with enhanced error handling.
     * 
     * <p>This method implements robust deletion logic that:</p>
     * <ul>
     *   <li>Handles files that may be in use or locked</li>
     *   <li>Attempts to change permissions before deletion</li>
     *   <li>Provides detailed error information for debugging</li>
     *   <li>Continues deletion even if individual files fail</li>
     * </ul>
     * 
     * @param path The path to delete
     * @throws IOException if deletion fails and the directory still exists
     */
    private static void deleteRecursively(Path path) throws IOException {
        if (!Files.exists(path)) {
            return;
        }
        
        List<IOException> suppressedExceptions = new ArrayList<>();
        
        try {
            Files.walk(path)
                .sorted(Comparator.reverseOrder())
                .forEach(filePath -> {
                    try {
                        // Try to make the file deleteable before deletion
                        File file = filePath.toFile();
                        if (file.exists()) {
                            file.setWritable(true);
                            file.setReadable(true);
                            if (Files.isDirectory(filePath)) {
                                file.setExecutable(true);
                            }
                        }
                        
                        Files.deleteIfExists(filePath);
                        
                    } catch (IOException e) {
                        suppressedExceptions.add(new IOException("Failed to delete: " + filePath + " - " + e.getMessage(), e));
                    } catch (Exception e) {
                        suppressedExceptions.add(new IOException("Unexpected error deleting: " + filePath + " - " + e.getMessage(), e));
                    }
                });
        } catch (IOException e) {
            IOException mainException = new IOException("Failed to walk directory tree: " + path, e);
            suppressedExceptions.forEach(mainException::addSuppressed);
            throw mainException;
        }
        
        // Check if deletion was successful
        if (Files.exists(path)) {
            IOException mainException = new IOException("Directory still exists after deletion attempt: " + path);
            suppressedExceptions.forEach(mainException::addSuppressed);
            throw mainException;
        }
        
        // Log any suppressed exceptions for debugging
        if (!suppressedExceptions.isEmpty() && 
            Boolean.parseBoolean(System.getProperty("overdrive.debug.cleanup", "false"))) {
            logCleanupOperation("Deletion completed with " + suppressedExceptions.size() + " suppressed exceptions for: " + path, null);
        }
    }
    
    /**
     * Gets the detected platform string.
     * 
     * @return The platform string (e.g., "windows-x64", "linux-arm64")
     */
    public static String getPlatform() {
        return detectPlatform();
    }
    
    /**
     * Gets the library name for the current platform.
     * 
     * @return The library file name, or null if platform is unsupported
     */
    public static String getLibraryName() {
        return LIBRARY_MAPPING.get(detectPlatform());
    }
    
    /**
     * Gets all supported platforms.
     * 
     * @return Set of supported platform strings
     */
    public static Set<String> getSupportedPlatforms() {
        return LIBRARY_MAPPING.keySet();
    }
    
    /**
     * Checks if the current platform is supported.
     * 
     * @return true if the current platform is supported
     */
    public static boolean isPlatformSupported() {
        return LIBRARY_MAPPING.containsKey(detectPlatform());
    }
    
    // ── Enhanced Diagnostic Methods ────────────────────────────────────
    
    /**
     * Provides comprehensive diagnostic information about the native library loading environment.
     * 
     * <p>This method collects detailed information about the current system environment,
     * platform detection results, library availability, and potential issues that might
     * affect native library loading. It's designed to help users and developers troubleshoot
     * library loading problems.</p>
     * 
     * @return DiagnosticInfo containing comprehensive system and library information
     */
    public static DiagnosticInfo getDiagnosticInfo() {
        DiagnosticInfoBuilder builder = new DiagnosticInfoBuilder();
        
        // Platform detection information
        builder.setPlatformInfo(detectPlatform(), detectOS(), detectArchitecture());
        builder.setSupportedPlatforms(getSupportedPlatforms());
        builder.setPlatformSupported(isPlatformSupported());
        
        // Library information
        String libraryName = getLibraryName();
        builder.setLibraryInfo(libraryName, LIBRARY_MAPPING);
        
        // System environment information
        builder.setSystemInfo(
            System.getProperty("java.version"),
            System.getProperty("java.vendor"),
            System.getProperty("os.name"),
            System.getProperty("os.version"),
            System.getProperty("os.arch"),
            System.getProperty("java.library.path"),
            System.getProperty("java.io.tmpdir")
        );
        
        // Environment variables
        builder.setEnvironmentVariables(
            System.getenv("OVERDRIVE_LIBRARY_PATH"),
            System.getenv("PATH"),
            System.getenv("LD_LIBRARY_PATH"),
            System.getenv("DYLD_LIBRARY_PATH")
        );
        
        // Library availability checks
        builder.setLibraryAvailability(checkLibraryAvailability());
        
        // Cleanup information
        builder.setCleanupInfo(getCleanupInfo());
        
        // Debug settings
        builder.setDebugSettings(
            Boolean.parseBoolean(System.getProperty("overdrive.debug.library.loading", "false")),
            Boolean.parseBoolean(System.getProperty("overdrive.debug.cleanup", "false"))
        );
        
        return builder.build();
    }
    
    /**
     * Performs comprehensive library availability checks across different loading strategies.
     * 
     * <p>This method tests various library loading approaches to determine which ones
     * are available and which ones fail, providing detailed information for troubleshooting.</p>
     * 
     * @return LibraryAvailabilityInfo containing results of availability checks
     */
    public static LibraryAvailabilityInfo checkLibraryAvailability() {
        String platform = detectPlatform();
        String libraryName = getLibraryName();
        
        LibraryAvailabilityInfoBuilder builder = new LibraryAvailabilityInfoBuilder();
        builder.setPlatform(platform);
        builder.setLibraryName(libraryName);
        
        if (libraryName == null) {
            builder.addResult("Platform Check", false, "Platform not supported: " + platform);
            return builder.build();
        }
        
        // Test bundled library availability
        testBundledLibraryAvailability(builder, libraryName, platform);
        
        // Test system library availability
        testSystemLibraryAvailability(builder, libraryName);
        
        // Test alternative library names
        testAlternativeLibraryAvailability(builder, platform);
        
        // Test common library paths
        testCommonPathAvailability(builder, libraryName, platform);
        
        // Test environment variable paths
        testEnvironmentPathAvailability(builder, libraryName, platform);
        
        return builder.build();
    }
    
    /**
     * Tests bundled library availability from JAR resources.
     */
    private static void testBundledLibraryAvailability(LibraryAvailabilityInfoBuilder builder, 
                                                      String libraryName, String platform) {
        try {
            String[] platformParts = platform.split("-");
            if (platformParts.length == 2) {
                String os = platformParts[0];
                String arch = platformParts[1];
                
                // Test organized structure
                String organizedPath = "/native/" + os + "/" + arch + "/" + libraryName;
                boolean organizedExists = NativeLibraryLoader.class.getResourceAsStream(organizedPath) != null;
                builder.addResult("Bundled Library (Organized)", organizedExists, 
                    organizedExists ? "Found at " + organizedPath : "Not found at " + organizedPath);
                
                // Test legacy structure
                String legacyPath = "/" + libraryName;
                boolean legacyExists = NativeLibraryLoader.class.getResourceAsStream(legacyPath) != null;
                builder.addResult("Bundled Library (Legacy)", legacyExists,
                    legacyExists ? "Found at " + legacyPath : "Not found at " + legacyPath);
            }
        } catch (Exception e) {
            builder.addResult("Bundled Library Check", false, "Exception: " + e.getMessage());
        }
    }
    
    /**
     * Tests system library availability using standard loading mechanisms.
     */
    private static void testSystemLibraryAvailability(LibraryAvailabilityInfoBuilder builder, String libraryName) {
        // Test platform-specific library name
        try {
            String systemLibraryName = getSystemLibraryName(libraryName);
            boolean available = testSystemLibraryLoad(systemLibraryName);
            builder.addResult("System Library (" + systemLibraryName + ")", available,
                available ? "Available in system path" : "Not found in system path");
        } catch (Exception e) {
            builder.addResult("System Library Test", false, "Exception: " + e.getMessage());
        }
        
        // Test generic "overdrive" name
        try {
            boolean available = testSystemLibraryLoad("overdrive");
            builder.addResult("System Library (overdrive)", available,
                available ? "Available in system path" : "Not found in system path");
        } catch (Exception e) {
            builder.addResult("System Library (overdrive)", false, "Exception: " + e.getMessage());
        }
    }
    
    /**
     * Tests alternative library name availability.
     */
    private static void testAlternativeLibraryAvailability(LibraryAvailabilityInfoBuilder builder, String platform) {
        String[] alternativeNames = {
            "overdrive-db", "overdrivedb", "liboverdrive", "overdrive_native", "odb"
        };
        
        for (String altName : alternativeNames) {
            try {
                boolean available = testSystemLibraryLoad(altName);
                builder.addResult("Alternative Name (" + altName + ")", available,
                    available ? "Available in system path" : "Not found in system path");
            } catch (Exception e) {
                builder.addResult("Alternative Name (" + altName + ")", false, "Exception: " + e.getMessage());
            }
        }
    }
    
    /**
     * Tests library availability in common installation paths.
     */
    private static void testCommonPathAvailability(LibraryAvailabilityInfoBuilder builder, 
                                                  String libraryName, String platform) {
        List<String> commonPaths = getCommonLibraryPaths(platform);
        
        for (String path : commonPaths) {
            try {
                Path libraryPath = Paths.get(path, libraryName);
                boolean exists = Files.exists(libraryPath) && Files.isReadable(libraryPath);
                builder.addResult("Common Path (" + path + ")", exists,
                    exists ? "Found at " + libraryPath : "Not found at " + libraryPath);
            } catch (Exception e) {
                builder.addResult("Common Path (" + path + ")", false, "Exception: " + e.getMessage());
            }
        }
    }
    
    /**
     * Tests library availability in environment variable specified paths.
     */
    private static void testEnvironmentPathAvailability(LibraryAvailabilityInfoBuilder builder,
                                                       String libraryName, String platform) {
        // Test OVERDRIVE_LIBRARY_PATH
        String overdriveLibPath = System.getenv("OVERDRIVE_LIBRARY_PATH");
        if (overdriveLibPath != null && !overdriveLibPath.trim().isEmpty()) {
            testPathForLibrary(builder, "OVERDRIVE_LIBRARY_PATH", overdriveLibPath, libraryName);
        } else {
            builder.addResult("OVERDRIVE_LIBRARY_PATH", false, "Environment variable not set");
        }
        
        // Test platform-specific environment variables
        List<String> envVars = getPlatformSpecificLibraryEnvVars(platform);
        for (String envVar : envVars) {
            String envValue = System.getenv(envVar);
            if (envValue != null && !envValue.trim().isEmpty()) {
                String[] paths = envValue.split(getPlatformPathSeparator(platform));
                for (String path : paths) {
                    if (!path.trim().isEmpty()) {
                        testPathForLibrary(builder, envVar + " (" + path.trim() + ")", 
                                         path.trim(), libraryName);
                    }
                }
            } else {
                builder.addResult(envVar, false, "Environment variable not set");
            }
        }
    }
    
    /**
     * Tests a specific path for library availability.
     */
    private static void testPathForLibrary(LibraryAvailabilityInfoBuilder builder, 
                                          String testName, String path, String libraryName) {
        try {
            Path libraryPath = Paths.get(path, libraryName);
            boolean exists = Files.exists(libraryPath) && Files.isReadable(libraryPath);
            builder.addResult(testName, exists,
                exists ? "Found at " + libraryPath : "Not found at " + libraryPath);
        } catch (Exception e) {
            builder.addResult(testName, false, "Exception: " + e.getMessage());
        }
    }
    
    /**
     * Tests if a system library can be loaded without actually loading it.
     */
    private static boolean testSystemLibraryLoad(String libraryName) {
        try {
            // This is a simplified test - in a real implementation, you might want to
            // use more sophisticated detection methods that don't actually load the library
            String libraryPath = System.mapLibraryName(libraryName);
            
            // Check if the library exists in java.library.path
            String javaLibraryPath = System.getProperty("java.library.path", "");
            if (!javaLibraryPath.isEmpty()) {
                String[] paths = javaLibraryPath.split(System.getProperty("path.separator"));
                for (String path : paths) {
                    if (!path.trim().isEmpty()) {
                        Path fullPath = Paths.get(path.trim(), libraryPath);
                        if (Files.exists(fullPath) && Files.isReadable(fullPath)) {
                            return true;
                        }
                    }
                }
            }
            
            return false;
        } catch (Exception e) {
            return false;
        }
    }
    
    /**
     * Provides troubleshooting suggestions based on the current system state.
     * 
     * <p>This method analyzes the current platform, library availability, and system
     * configuration to provide specific, actionable troubleshooting suggestions.</p>
     * 
     * @return TroubleshootingInfo containing platform-specific suggestions and solutions
     */
    public static TroubleshootingInfo getTroubleshootingSuggestions() {
        TroubleshootingInfoBuilder builder = new TroubleshootingInfoBuilder();
        
        String platform = detectPlatform();
        boolean isSupported = isPlatformSupported();
        LibraryAvailabilityInfo availability = checkLibraryAvailability();
        
        // Platform-specific suggestions
        if (!isSupported) {
            builder.addCriticalIssue("Unsupported Platform", 
                "Your platform (" + platform + ") is not supported by this version of the OverDrive SDK.",
                Arrays.asList(
                    "Check if a newer version of the SDK supports your platform",
                    "Contact support to request support for your platform",
                    "Consider using a supported platform: " + String.join(", ", getSupportedPlatforms())
                ));
        } else {
            // Check for library availability issues
            if (!availability.hasAnyAvailableLibrary()) {
                builder.addCriticalIssue("No Libraries Available",
                    "No OverDrive native libraries were found in any of the checked locations.",
                    Arrays.asList(
                        "Reinstall the OverDrive SDK package",
                        "Verify the package installation completed successfully",
                        "Check that native libraries are included in your JAR file",
                        "Set OVERDRIVE_LIBRARY_PATH to a directory containing the native library"
                    ));
            }
            
            // Platform-specific suggestions
            addPlatformSpecificSuggestions(builder, platform, availability);
        }
        
        // General suggestions
        addGeneralTroubleshootingSuggestions(builder, availability);
        
        // Debug suggestions
        addDebugSuggestions(builder);
        
        return builder.build();
    }
    
    /**
     * Adds platform-specific troubleshooting suggestions.
     */
    private static void addPlatformSpecificSuggestions(TroubleshootingInfoBuilder builder, 
                                                       String platform, LibraryAvailabilityInfo availability) {
        if (platform.startsWith("windows")) {
            addWindowsTroubleshootingSuggestions(builder, availability);
        } else if (platform.startsWith("linux")) {
            addLinuxTroubleshootingSuggestions(builder, availability);
        } else if (platform.startsWith("macos")) {
            addMacOSTroubleshootingSuggestions(builder, availability);
        }
    }
    
    /**
     * Adds Windows-specific troubleshooting suggestions.
     */
    private static void addWindowsTroubleshootingSuggestions(TroubleshootingInfoBuilder builder,
                                                            LibraryAvailabilityInfo availability) {
        builder.addSuggestion("Windows Library Path",
            "Ensure the OverDrive DLL is in your system PATH or application directory.",
            Arrays.asList(
                "Add the library directory to your PATH environment variable",
                "Copy overdrive.dll to your application's working directory",
                "Install Microsoft Visual C++ Redistributable if missing",
                "Check Windows Event Viewer for DLL loading errors"
            ));
        
        if (!availability.isSystemLibraryAvailable()) {
            builder.addWarning("System Library Not Found",
                "The OverDrive library was not found in the Windows system path.",
                Arrays.asList(
                    "Install the OverDrive library system-wide",
                    "Use the bundled library instead of system installation",
                    "Check that the library architecture (x64) matches your Java runtime"
                ));
        }
    }
    
    /**
     * Adds Linux-specific troubleshooting suggestions.
     */
    private static void addLinuxTroubleshootingSuggestions(TroubleshootingInfoBuilder builder,
                                                          LibraryAvailabilityInfo availability) {
        builder.addSuggestion("Linux Library Path",
            "Configure LD_LIBRARY_PATH or install the library system-wide.",
            Arrays.asList(
                "Set LD_LIBRARY_PATH to include the library directory",
                "Copy the library to /usr/local/lib or /usr/lib",
                "Run 'ldconfig' after installing the library system-wide",
                "Check library dependencies with 'ldd' command"
            ));
        
        String ldLibraryPath = System.getenv("LD_LIBRARY_PATH");
        if (ldLibraryPath == null || ldLibraryPath.trim().isEmpty()) {
            builder.addWarning("LD_LIBRARY_PATH Not Set",
                "The LD_LIBRARY_PATH environment variable is not configured.",
                Arrays.asList(
                    "Set LD_LIBRARY_PATH to include directories containing OverDrive libraries",
                    "Add 'export LD_LIBRARY_PATH=/path/to/overdrive/lib:$LD_LIBRARY_PATH' to your shell profile",
                    "Use the bundled library instead of system installation"
                ));
        }
    }
    
    /**
     * Adds macOS-specific troubleshooting suggestions.
     */
    private static void addMacOSTroubleshootingSuggestions(TroubleshootingInfoBuilder builder,
                                                          LibraryAvailabilityInfo availability) {
        builder.addSuggestion("macOS Library Path",
            "Configure DYLD_LIBRARY_PATH or install the library in standard locations.",
            Arrays.asList(
                "Set DYLD_LIBRARY_PATH to include the library directory",
                "Copy the library to /usr/local/lib",
                "Check code signing and notarization status",
                "Verify Gatekeeper allows the library to run"
            ));
        
        String dyldLibraryPath = System.getenv("DYLD_LIBRARY_PATH");
        if (dyldLibraryPath == null || dyldLibraryPath.trim().isEmpty()) {
            builder.addInfo("DYLD_LIBRARY_PATH Not Set",
                "The DYLD_LIBRARY_PATH environment variable is not configured.",
                Arrays.asList(
                    "Set DYLD_LIBRARY_PATH if using custom library locations",
                    "Note: DYLD_LIBRARY_PATH may be restricted in some macOS versions",
                    "Use the bundled library for most reliable operation"
                ));
        }
    }
    
    /**
     * Adds general troubleshooting suggestions.
     */
    private static void addGeneralTroubleshootingSuggestions(TroubleshootingInfoBuilder builder,
                                                            LibraryAvailabilityInfo availability) {
        builder.addSuggestion("General Troubleshooting",
            "Common solutions for native library loading issues.",
            Arrays.asList(
                "Restart your application to clear any cached library loading failures",
                "Check that your Java architecture matches the library architecture",
                "Verify file permissions allow reading and executing the library",
                "Ensure no antivirus software is blocking the library"
            ));
        
        // Check Java architecture compatibility
        String javaArch = System.getProperty("os.arch", "").toLowerCase();
        String detectedArch = detectArchitecture();
        if (!javaArch.contains("64") && detectedArch.equals("x64")) {
            builder.addWarning("Architecture Mismatch",
                "You may be running 32-bit Java on a 64-bit system.",
                Arrays.asList(
                    "Install 64-bit Java to match the native library architecture",
                    "Use a 32-bit version of the OverDrive library if available",
                    "Check your Java installation with 'java -version'"
                ));
        }
    }
    
    /**
     * Adds debug-related suggestions.
     */
    private static void addDebugSuggestions(TroubleshootingInfoBuilder builder) {
        builder.addInfo("Debug Options",
            "Enable debug logging to get more detailed information about library loading.",
            Arrays.asList(
                "Set system property: -Doverdrive.debug.library.loading=true",
                "Set system property: -Doverdrive.debug.cleanup=true",
                "Check application logs for detailed error messages",
                "Use getDiagnosticInfo() method for comprehensive system information"
            ));
    }
    
    /**
     * Validates the current system configuration for OverDrive library loading.
     * 
     * <p>This method performs a comprehensive validation of the system environment
     * to identify potential issues that might prevent successful library loading.</p>
     * 
     * @return ValidationResult containing validation status and any issues found
     */
    public static ValidationResult validateSystemConfiguration() {
        ValidationResultBuilder builder = new ValidationResultBuilder();
        
        // Platform validation
        validatePlatform(builder);
        
        // Java environment validation
        validateJavaEnvironment(builder);
        
        // Library availability validation
        validateLibraryAvailability(builder);
        
        // File system permissions validation
        validateFileSystemPermissions(builder);
        
        // Environment variables validation
        validateEnvironmentVariables(builder);
        
        return builder.build();
    }
    
    /**
     * Validates platform support and detection.
     */
    private static void validatePlatform(ValidationResultBuilder builder) {
        try {
            String platform = detectPlatform();
            String os = detectOS();
            String arch = detectArchitecture();
            
            builder.addCheck("Platform Detection", true, 
                "Detected platform: " + platform + " (OS: " + os + ", Arch: " + arch + ")");
            
            boolean isSupported = isPlatformSupported();
            if (isSupported) {
                builder.addCheck("Platform Support", true, "Platform is supported");
            } else {
                builder.addCheck("Platform Support", false, 
                    "Platform " + platform + " is not supported. Supported platforms: " + 
                    String.join(", ", getSupportedPlatforms()));
            }
            
        } catch (Exception e) {
            builder.addCheck("Platform Detection", false, "Exception during platform detection: " + e.getMessage());
        }
    }
    
    /**
     * Validates Java environment compatibility.
     */
    private static void validateJavaEnvironment(ValidationResultBuilder builder) {
        try {
            String javaVersion = System.getProperty("java.version");
            String javaVendor = System.getProperty("java.vendor");
            String javaArch = System.getProperty("os.arch");
            
            builder.addCheck("Java Version", true, 
                "Java " + javaVersion + " by " + javaVendor + " (" + javaArch + ")");
            
            // Check Java architecture compatibility
            String detectedArch = detectArchitecture();
            boolean archCompatible = (javaArch.contains("64") && detectedArch.equals("x64")) ||
                                   (javaArch.contains("aarch64") && detectedArch.equals("arm64"));
            
            if (archCompatible) {
                builder.addCheck("Architecture Compatibility", true, 
                    "Java architecture (" + javaArch + ") is compatible with detected architecture (" + detectedArch + ")");
            } else {
                builder.addCheck("Architecture Compatibility", false,
                    "Java architecture (" + javaArch + ") may not be compatible with detected architecture (" + detectedArch + ")");
            }
            
        } catch (Exception e) {
            builder.addCheck("Java Environment", false, "Exception during Java environment validation: " + e.getMessage());
        }
    }
    
    /**
     * Validates library availability across different loading strategies.
     */
    private static void validateLibraryAvailability(ValidationResultBuilder builder) {
        try {
            LibraryAvailabilityInfo availability = checkLibraryAvailability();
            
            if (availability.hasAnyAvailableLibrary()) {
                builder.addCheck("Library Availability", true, "At least one library loading method is available");
                
                // Report specific availability
                if (availability.isBundledLibraryAvailable()) {
                    builder.addCheck("Bundled Library", true, "Bundled library is available in JAR resources");
                }
                if (availability.isSystemLibraryAvailable()) {
                    builder.addCheck("System Library", true, "System library is available in library path");
                }
                
            } else {
                builder.addCheck("Library Availability", false, 
                    "No OverDrive libraries found in any checked locations");
            }
            
        } catch (Exception e) {
            builder.addCheck("Library Availability", false, 
                "Exception during library availability check: " + e.getMessage());
        }
    }
    
    /**
     * Validates file system permissions for temporary file operations.
     */
    private static void validateFileSystemPermissions(ValidationResultBuilder builder) {
        try {
            String tempDir = System.getProperty("java.io.tmpdir");
            Path tempDirPath = Paths.get(tempDir);
            
            if (Files.exists(tempDirPath) && Files.isDirectory(tempDirPath)) {
                builder.addCheck("Temp Directory Exists", true, "Temporary directory: " + tempDir);
                
                if (Files.isWritable(tempDirPath)) {
                    builder.addCheck("Temp Directory Writable", true, "Temporary directory is writable");
                } else {
                    builder.addCheck("Temp Directory Writable", false, "Temporary directory is not writable");
                }
                
                // Test actual file creation
                try {
                    Path testFile = Files.createTempFile(tempDirPath, "overdrive-test-", ".tmp");
                    Files.deleteIfExists(testFile);
                    builder.addCheck("Temp File Creation", true, "Can create temporary files");
                } catch (Exception e) {
                    builder.addCheck("Temp File Creation", false, "Cannot create temporary files: " + e.getMessage());
                }
                
            } else {
                builder.addCheck("Temp Directory Exists", false, "Temporary directory does not exist or is not a directory");
            }
            
        } catch (Exception e) {
            builder.addCheck("File System Permissions", false, 
                "Exception during file system validation: " + e.getMessage());
        }
    }
    
    /**
     * Validates environment variables that affect library loading.
     */
    private static void validateEnvironmentVariables(ValidationResultBuilder builder) {
        try {
            // Check OVERDRIVE_LIBRARY_PATH
            String overdriveLibPath = System.getenv("OVERDRIVE_LIBRARY_PATH");
            if (overdriveLibPath != null && !overdriveLibPath.trim().isEmpty()) {
                Path path = Paths.get(overdriveLibPath);
                if (Files.exists(path) && Files.isDirectory(path)) {
                    builder.addCheck("OVERDRIVE_LIBRARY_PATH", true, "Valid directory: " + overdriveLibPath);
                } else {
                    builder.addCheck("OVERDRIVE_LIBRARY_PATH", false, "Directory does not exist: " + overdriveLibPath);
                }
            } else {
                builder.addCheck("OVERDRIVE_LIBRARY_PATH", true, "Not set (using default behavior)");
            }
            
            // Check platform-specific library paths
            String platform = detectPlatform();
            List<String> envVars = getPlatformSpecificLibraryEnvVars(platform);
            
            for (String envVar : envVars) {
                String envValue = System.getenv(envVar);
                if (envValue != null && !envValue.trim().isEmpty()) {
                    builder.addCheck(envVar, true, "Set to: " + envValue);
                } else {
                    builder.addCheck(envVar, true, "Not set (using system defaults)");
                }
            }
            
        } catch (Exception e) {
            builder.addCheck("Environment Variables", false, 
                "Exception during environment variable validation: " + e.getMessage());
        }
    }

    // ── Diagnostic Data Classes ────────────────────────────────────────

    /**
     * Comprehensive diagnostic information about the native library loading environment.
     *
     * <p>Contains platform info, system environment, library availability, and debug settings
     * to help troubleshoot native library loading issues.</p>
     */
    public static class DiagnosticInfo {
        private final String platform;
        private final String os;
        private final String architecture;
        private final boolean platformSupported;
        private final Set<String> supportedPlatforms;
        private final String libraryName;
        private final Map<String, String> libraryMapping;
        private final String javaVersion;
        private final String javaVendor;
        private final String osName;
        private final String osVersion;
        private final String osArch;
        private final String javaLibraryPath;
        private final String tempDir;
        private final String overdriveLibraryPath;
        private final String pathEnv;
        private final String ldLibraryPath;
        private final String dyldLibraryPath;
        private final LibraryAvailabilityInfo libraryAvailability;
        private final CleanupInfo cleanupInfo;
        private final boolean debugLibraryLoading;
        private final boolean debugCleanup;

        DiagnosticInfo(String platform, String os, String architecture, boolean platformSupported,
                       Set<String> supportedPlatforms, String libraryName, Map<String, String> libraryMapping,
                       String javaVersion, String javaVendor, String osName, String osVersion, String osArch,
                       String javaLibraryPath, String tempDir, String overdriveLibraryPath, String pathEnv,
                       String ldLibraryPath, String dyldLibraryPath, LibraryAvailabilityInfo libraryAvailability,
                       CleanupInfo cleanupInfo, boolean debugLibraryLoading, boolean debugCleanup) {
            this.platform = platform;
            this.os = os;
            this.architecture = architecture;
            this.platformSupported = platformSupported;
            this.supportedPlatforms = supportedPlatforms;
            this.libraryName = libraryName;
            this.libraryMapping = libraryMapping;
            this.javaVersion = javaVersion;
            this.javaVendor = javaVendor;
            this.osName = osName;
            this.osVersion = osVersion;
            this.osArch = osArch;
            this.javaLibraryPath = javaLibraryPath;
            this.tempDir = tempDir;
            this.overdriveLibraryPath = overdriveLibraryPath;
            this.pathEnv = pathEnv;
            this.ldLibraryPath = ldLibraryPath;
            this.dyldLibraryPath = dyldLibraryPath;
            this.libraryAvailability = libraryAvailability;
            this.cleanupInfo = cleanupInfo;
            this.debugLibraryLoading = debugLibraryLoading;
            this.debugCleanup = debugCleanup;
        }

        public String getPlatform() { return platform; }
        public String getOs() { return os; }
        public String getArchitecture() { return architecture; }
        public boolean isPlatformSupported() { return platformSupported; }
        public Set<String> getSupportedPlatforms() { return supportedPlatforms; }
        public String getLibraryName() { return libraryName; }
        public Map<String, String> getLibraryMapping() { return libraryMapping; }
        public String getJavaVersion() { return javaVersion; }
        public String getJavaVendor() { return javaVendor; }
        public String getOsName() { return osName; }
        public String getOsVersion() { return osVersion; }
        public String getOsArch() { return osArch; }
        public String getJavaLibraryPath() { return javaLibraryPath; }
        public String getTempDir() { return tempDir; }
        public String getOverdriveLibraryPath() { return overdriveLibraryPath; }
        public String getPathEnv() { return pathEnv; }
        public String getLdLibraryPath() { return ldLibraryPath; }
        public String getDyldLibraryPath() { return dyldLibraryPath; }
        public LibraryAvailabilityInfo getLibraryAvailability() { return libraryAvailability; }
        public CleanupInfo getCleanupInfo() { return cleanupInfo; }
        public boolean isDebugLibraryLoading() { return debugLibraryLoading; }
        public boolean isDebugCleanup() { return debugCleanup; }

        /**
         * Generates a human-readable diagnostic report suitable for bug reports and support tickets.
         *
         * @return Formatted multi-line diagnostic report string
         */
        public String generateReport() {
            StringBuilder sb = new StringBuilder();
            sb.append("=== OverDrive Native Library Diagnostic Report ===\n");
            sb.append("\n--- Platform Information ---\n");
            sb.append("Detected Platform:  ").append(platform).append("\n");
            sb.append("Operating System:   ").append(os).append("\n");
            sb.append("Architecture:       ").append(architecture).append("\n");
            sb.append("Platform Supported: ").append(platformSupported).append("\n");
            sb.append("Supported Platforms: ").append(String.join(", ", supportedPlatforms)).append("\n");
            sb.append("Library Name:       ").append(libraryName != null ? libraryName : "(none - unsupported platform)").append("\n");

            sb.append("\n--- Java Environment ---\n");
            sb.append("Java Version:  ").append(javaVersion).append("\n");
            sb.append("Java Vendor:   ").append(javaVendor).append("\n");
            sb.append("OS Name:       ").append(osName).append("\n");
            sb.append("OS Version:    ").append(osVersion).append("\n");
            sb.append("OS Arch:       ").append(osArch).append("\n");
            sb.append("Temp Dir:      ").append(tempDir).append("\n");
            sb.append("Library Path:  ").append(javaLibraryPath != null ? javaLibraryPath : "(not set)").append("\n");

            sb.append("\n--- Environment Variables ---\n");
            sb.append("OVERDRIVE_LIBRARY_PATH: ").append(overdriveLibraryPath != null ? overdriveLibraryPath : "(not set)").append("\n");
            sb.append("LD_LIBRARY_PATH:        ").append(ldLibraryPath != null ? ldLibraryPath : "(not set)").append("\n");
            sb.append("DYLD_LIBRARY_PATH:      ").append(dyldLibraryPath != null ? dyldLibraryPath : "(not set)").append("\n");

            sb.append("\n--- Library Availability ---\n");
            if (libraryAvailability != null) {
                for (Map.Entry<String, String> entry : libraryAvailability.getResults().entrySet()) {
                    sb.append("  ").append(entry.getKey()).append(": ").append(entry.getValue()).append("\n");
                }
            }

            sb.append("\n--- Cleanup State ---\n");
            if (cleanupInfo != null) {
                sb.append("Tracked Temp Dirs: ").append(cleanupInfo.getTrackedCount()).append("\n");
                sb.append("Existing Temp Dirs: ").append(cleanupInfo.getExistingCount()).append("\n");
                sb.append("Cleanup Hook Registered: ").append(cleanupInfo.isCleanupHookRegistered()).append("\n");
            }

            sb.append("\n--- Debug Settings ---\n");
            sb.append("Library Loading Debug: ").append(debugLibraryLoading).append("\n");
            sb.append("Cleanup Debug:         ").append(debugCleanup).append("\n");
            sb.append("  (Enable with: -Doverdrive.debug.library.loading=true)\n");

            sb.append("\n=== End of Diagnostic Report ===\n");
            return sb.toString();
        }

        @Override
        public String toString() {
            return "DiagnosticInfo{platform='" + platform + "', supported=" + platformSupported +
                   ", javaVersion='" + javaVersion + "', os='" + osName + "'}";
        }
    }

    /**
     * Builder for {@link DiagnosticInfo}.
     */
    static class DiagnosticInfoBuilder {
        private String platform;
        private String os;
        private String architecture;
        private boolean platformSupported;
        private Set<String> supportedPlatforms = new HashSet<>();
        private String libraryName;
        private Map<String, String> libraryMapping = new HashMap<>();
        private String javaVersion;
        private String javaVendor;
        private String osName;
        private String osVersion;
        private String osArch;
        private String javaLibraryPath;
        private String tempDir;
        private String overdriveLibraryPath;
        private String pathEnv;
        private String ldLibraryPath;
        private String dyldLibraryPath;
        private LibraryAvailabilityInfo libraryAvailability;
        private CleanupInfo cleanupInfo;
        private boolean debugLibraryLoading;
        private boolean debugCleanup;

        void setPlatformInfo(String platform, String os, String architecture) {
            this.platform = platform;
            this.os = os;
            this.architecture = architecture;
        }

        void setSupportedPlatforms(Set<String> supportedPlatforms) {
            this.supportedPlatforms = new HashSet<>(supportedPlatforms);
        }

        void setPlatformSupported(boolean platformSupported) {
            this.platformSupported = platformSupported;
        }

        void setLibraryInfo(String libraryName, Map<String, String> libraryMapping) {
            this.libraryName = libraryName;
            this.libraryMapping = new HashMap<>(libraryMapping);
        }

        void setSystemInfo(String javaVersion, String javaVendor, String osName, String osVersion,
                           String osArch, String javaLibraryPath, String tempDir) {
            this.javaVersion = javaVersion;
            this.javaVendor = javaVendor;
            this.osName = osName;
            this.osVersion = osVersion;
            this.osArch = osArch;
            this.javaLibraryPath = javaLibraryPath;
            this.tempDir = tempDir;
        }

        void setEnvironmentVariables(String overdriveLibraryPath, String pathEnv,
                                     String ldLibraryPath, String dyldLibraryPath) {
            this.overdriveLibraryPath = overdriveLibraryPath;
            this.pathEnv = pathEnv;
            this.ldLibraryPath = ldLibraryPath;
            this.dyldLibraryPath = dyldLibraryPath;
        }

        void setLibraryAvailability(LibraryAvailabilityInfo libraryAvailability) {
            this.libraryAvailability = libraryAvailability;
        }

        void setCleanupInfo(CleanupInfo cleanupInfo) {
            this.cleanupInfo = cleanupInfo;
        }

        void setDebugSettings(boolean debugLibraryLoading, boolean debugCleanup) {
            this.debugLibraryLoading = debugLibraryLoading;
            this.debugCleanup = debugCleanup;
        }

        DiagnosticInfo build() {
            return new DiagnosticInfo(platform, os, architecture, platformSupported, supportedPlatforms,
                    libraryName, libraryMapping, javaVersion, javaVendor, osName, osVersion, osArch,
                    javaLibraryPath, tempDir, overdriveLibraryPath, pathEnv, ldLibraryPath, dyldLibraryPath,
                    libraryAvailability, cleanupInfo, debugLibraryLoading, debugCleanup);
        }
    }

    /**
     * Information about library availability across different loading strategies.
     */
    public static class LibraryAvailabilityInfo {
        private final String platform;
        private final String libraryName;
        private final Map<String, String> results;
        private final boolean bundledLibraryAvailable;
        private final boolean systemLibraryAvailable;

        LibraryAvailabilityInfo(String platform, String libraryName, Map<String, String> results,
                                boolean bundledLibraryAvailable, boolean systemLibraryAvailable) {
            this.platform = platform;
            this.libraryName = libraryName;
            this.results = Collections.unmodifiableMap(new LinkedHashMap<>(results));
            this.bundledLibraryAvailable = bundledLibraryAvailable;
            this.systemLibraryAvailable = systemLibraryAvailable;
        }

        public String getPlatform() { return platform; }
        public String getLibraryName() { return libraryName; }
        public Map<String, String> getResults() { return results; }
        public boolean isBundledLibraryAvailable() { return bundledLibraryAvailable; }
        public boolean isSystemLibraryAvailable() { return systemLibraryAvailable; }

        /** Returns true if at least one library loading method is available. */
        public boolean hasAnyAvailableLibrary() {
            return bundledLibraryAvailable || systemLibraryAvailable;
        }

        @Override
        public String toString() {
            return "LibraryAvailabilityInfo{platform='" + platform + "', bundled=" + bundledLibraryAvailable +
                   ", system=" + systemLibraryAvailable + ", checks=" + results.size() + "}";
        }
    }

    /**
     * Builder for {@link LibraryAvailabilityInfo}.
     */
    static class LibraryAvailabilityInfoBuilder {
        private String platform;
        private String libraryName;
        private final Map<String, String> results = new LinkedHashMap<>();
        private boolean bundledLibraryAvailable = false;
        private boolean systemLibraryAvailable = false;

        void setPlatform(String platform) { this.platform = platform; }
        void setLibraryName(String libraryName) { this.libraryName = libraryName; }

        void addResult(String checkName, boolean available, String detail) {
            results.put(checkName, (available ? "[OK] " : "[FAIL] ") + detail);
            if (available) {
                if (checkName.startsWith("Bundled")) {
                    bundledLibraryAvailable = true;
                } else if (checkName.startsWith("System")) {
                    systemLibraryAvailable = true;
                }
            }
        }

        LibraryAvailabilityInfo build() {
            return new LibraryAvailabilityInfo(platform, libraryName, results,
                    bundledLibraryAvailable, systemLibraryAvailable);
        }
    }

    /**
     * Troubleshooting information with platform-specific suggestions and solutions.
     */
    public static class TroubleshootingInfo {
        /** Severity level for a troubleshooting item. */
        public enum Severity { CRITICAL, WARNING, INFO, SUGGESTION }

        /** A single troubleshooting item with title, description, and action steps. */
        public static class Item {
            private final Severity severity;
            private final String title;
            private final String description;
            private final List<String> actions;

            Item(Severity severity, String title, String description, List<String> actions) {
                this.severity = severity;
                this.title = title;
                this.description = description;
                this.actions = Collections.unmodifiableList(new ArrayList<>(actions));
            }

            public Severity getSeverity() { return severity; }
            public String getTitle() { return title; }
            public String getDescription() { return description; }
            public List<String> getActions() { return actions; }

            @Override
            public String toString() {
                return "[" + severity + "] " + title + ": " + description;
            }
        }

        private final List<Item> items;

        TroubleshootingInfo(List<Item> items) {
            this.items = Collections.unmodifiableList(new ArrayList<>(items));
        }

        public List<Item> getItems() { return items; }

        /** Returns only critical issues. */
        public List<Item> getCriticalIssues() {
            List<Item> critical = new ArrayList<>();
            for (Item item : items) {
                if (item.getSeverity() == Severity.CRITICAL) critical.add(item);
            }
            return critical;
        }

        /** Returns only warnings. */
        public List<Item> getWarnings() {
            List<Item> warnings = new ArrayList<>();
            for (Item item : items) {
                if (item.getSeverity() == Severity.WARNING) warnings.add(item);
            }
            return warnings;
        }

        /** Returns true if there are any critical issues. */
        public boolean hasCriticalIssues() {
            for (Item item : items) {
                if (item.getSeverity() == Severity.CRITICAL) return true;
            }
            return false;
        }

        /**
         * Generates a human-readable troubleshooting report.
         *
         * @return Formatted multi-line troubleshooting report
         */
        public String generateReport() {
            StringBuilder sb = new StringBuilder();
            sb.append("=== OverDrive Native Library Troubleshooting Guide ===\n");
            for (Item item : items) {
                sb.append("\n[").append(item.getSeverity()).append("] ").append(item.getTitle()).append("\n");
                sb.append("  ").append(item.getDescription()).append("\n");
                if (!item.getActions().isEmpty()) {
                    sb.append("  Actions:\n");
                    for (String action : item.getActions()) {
                        sb.append("    \u2022 ").append(action).append("\n");
                    }
                }
            }
            sb.append("\n=== End of Troubleshooting Guide ===\n");
            return sb.toString();
        }

        @Override
        public String toString() {
            return "TroubleshootingInfo{items=" + items.size() + ", critical=" + getCriticalIssues().size() + "}";
        }
    }

    /**
     * Builder for {@link TroubleshootingInfo}.
     */
    static class TroubleshootingInfoBuilder {
        private final List<TroubleshootingInfo.Item> items = new ArrayList<>();

        void addCriticalIssue(String title, String description, List<String> actions) {
            items.add(new TroubleshootingInfo.Item(TroubleshootingInfo.Severity.CRITICAL, title, description, actions));
        }

        void addWarning(String title, String description, List<String> actions) {
            items.add(new TroubleshootingInfo.Item(TroubleshootingInfo.Severity.WARNING, title, description, actions));
        }

        void addSuggestion(String title, String description, List<String> actions) {
            items.add(new TroubleshootingInfo.Item(TroubleshootingInfo.Severity.SUGGESTION, title, description, actions));
        }

        void addInfo(String title, String description, List<String> actions) {
            items.add(new TroubleshootingInfo.Item(TroubleshootingInfo.Severity.INFO, title, description, actions));
        }

        TroubleshootingInfo build() {
            return new TroubleshootingInfo(items);
        }
    }

    /**
     * Result of a system configuration validation.
     */
    public static class ValidationResult {
        /** A single validation check result. */
        public static class Check {
            private final String name;
            private final boolean passed;
            private final String detail;

            Check(String name, boolean passed, String detail) {
                this.name = name;
                this.passed = passed;
                this.detail = detail;
            }

            public String getName() { return name; }
            public boolean isPassed() { return passed; }
            public String getDetail() { return detail; }

            @Override
            public String toString() {
                return (passed ? "[PASS] " : "[FAIL] ") + name + ": " + detail;
            }
        }

        private final List<Check> checks;

        ValidationResult(List<Check> checks) {
            this.checks = Collections.unmodifiableList(new ArrayList<>(checks));
        }

        public List<Check> getChecks() { return checks; }

        /** Returns true if all checks passed. */
        public boolean isValid() {
            for (Check check : checks) {
                if (!check.isPassed()) return false;
            }
            return true;
        }

        /** Returns only failed checks. */
        public List<Check> getFailedChecks() {
            List<Check> failed = new ArrayList<>();
            for (Check check : checks) {
                if (!check.isPassed()) failed.add(check);
            }
            return failed;
        }

        /** Returns only passed checks. */
        public List<Check> getPassedChecks() {
            List<Check> passed = new ArrayList<>();
            for (Check check : checks) {
                if (check.isPassed()) passed.add(check);
            }
            return passed;
        }

        /**
         * Generates a human-readable validation report.
         *
         * @return Formatted multi-line validation report
         */
        public String generateReport() {
            StringBuilder sb = new StringBuilder();
            sb.append("=== OverDrive System Configuration Validation ===\n");
            int passCount = 0;
            int failCount = 0;
            for (Check check : checks) {
                sb.append(check.isPassed() ? "  [PASS] " : "  [FAIL] ");
                sb.append(check.getName()).append(": ").append(check.getDetail()).append("\n");
                if (check.isPassed()) passCount++; else failCount++;
            }
            sb.append("\nResult: ").append(passCount).append(" passed, ").append(failCount).append(" failed\n");
            sb.append("Overall: ").append(isValid() ? "VALID" : "INVALID").append("\n");
            sb.append("=== End of Validation Report ===\n");
            return sb.toString();
        }

        @Override
        public String toString() {
            return "ValidationResult{checks=" + checks.size() + ", valid=" + isValid() + "}";
        }
    }

    /**
     * Builder for {@link ValidationResult}.
     */
    static class ValidationResultBuilder {
        private final List<ValidationResult.Check> checks = new ArrayList<>();

        void addCheck(String name, boolean passed, String detail) {
            checks.add(new ValidationResult.Check(name, passed, detail));
        }

        ValidationResult build() {
            return new ValidationResult(checks);
        }
    }
}