package com.afot.overdrive;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Nested;
import static org.junit.jupiter.api.Assertions.*;

import java.util.List;
import java.util.Map;
import java.util.Set;

/**
 * Comprehensive tests for NativeLibraryLoader diagnostic and error-handling methods.
 *
 * <p>Covers:</p>
 * <ul>
 *   <li>getDiagnosticInfo() — system state snapshot</li>
 *   <li>checkLibraryAvailability() — library availability checks</li>
 *   <li>getTroubleshootingSuggestions() — platform-specific suggestions</li>
 *   <li>validateSystemConfiguration() — configuration validation</li>
 *   <li>DiagnosticInfo.generateReport() — human-readable report</li>
 *   <li>ValidationResult.generateReport() — validation report</li>
 *   <li>TroubleshootingInfo.generateReport() — troubleshooting report</li>
 * </ul>
 */
public class DiagnosticsTest {

    // ── getDiagnosticInfo ──────────────────────────────────────────────

    @Nested
    @DisplayName("getDiagnosticInfo()")
    class GetDiagnosticInfoTests {

        @Test
        @DisplayName("Returns non-null DiagnosticInfo")
        void returnsNonNull() {
            NativeLibraryLoader.DiagnosticInfo info = NativeLibraryLoader.getDiagnosticInfo();
            assertNotNull(info, "getDiagnosticInfo() must not return null");
        }

        @Test
        @DisplayName("Platform fields are populated")
        void platformFieldsPopulated() {
            NativeLibraryLoader.DiagnosticInfo info = NativeLibraryLoader.getDiagnosticInfo();

            assertNotNull(info.getPlatform(), "platform must not be null");
            assertFalse(info.getPlatform().isEmpty(), "platform must not be empty");

            assertNotNull(info.getOs(), "os must not be null");
            assertFalse(info.getOs().isEmpty(), "os must not be empty");

            assertNotNull(info.getArchitecture(), "architecture must not be null");
            assertFalse(info.getArchitecture().isEmpty(), "architecture must not be empty");
        }

        @Test
        @DisplayName("Platform string matches os-arch format")
        void platformMatchesOsArch() {
            NativeLibraryLoader.DiagnosticInfo info = NativeLibraryLoader.getDiagnosticInfo();
            String expected = info.getOs() + "-" + info.getArchitecture();
            assertEquals(expected, info.getPlatform(),
                "platform should be os + '-' + architecture");
        }

        @Test
        @DisplayName("Supported platforms set is non-empty")
        void supportedPlatformsNonEmpty() {
            NativeLibraryLoader.DiagnosticInfo info = NativeLibraryLoader.getDiagnosticInfo();
            Set<String> supported = info.getSupportedPlatforms();
            assertNotNull(supported, "supportedPlatforms must not be null");
            assertFalse(supported.isEmpty(), "supportedPlatforms must not be empty");
        }

        @Test
        @DisplayName("platformSupported flag is consistent with supported set")
        void platformSupportedConsistent() {
            NativeLibraryLoader.DiagnosticInfo info = NativeLibraryLoader.getDiagnosticInfo();
            boolean expectedSupported = info.getSupportedPlatforms().contains(info.getPlatform());
            assertEquals(expectedSupported, info.isPlatformSupported(),
                "isPlatformSupported() should match whether platform is in supportedPlatforms");
        }

        @Test
        @DisplayName("Java environment fields are populated")
        void javaEnvironmentFieldsPopulated() {
            NativeLibraryLoader.DiagnosticInfo info = NativeLibraryLoader.getDiagnosticInfo();

            assertNotNull(info.getJavaVersion(), "javaVersion must not be null");
            assertFalse(info.getJavaVersion().isEmpty(), "javaVersion must not be empty");

            assertNotNull(info.getJavaVendor(), "javaVendor must not be null");

            assertNotNull(info.getOsName(), "osName must not be null");
            assertFalse(info.getOsName().isEmpty(), "osName must not be empty");

            assertNotNull(info.getTempDir(), "tempDir must not be null");
            assertFalse(info.getTempDir().isEmpty(), "tempDir must not be empty");
        }

        @Test
        @DisplayName("Library mapping is non-empty")
        void libraryMappingNonEmpty() {
            NativeLibraryLoader.DiagnosticInfo info = NativeLibraryLoader.getDiagnosticInfo();
            Map<String, String> mapping = info.getLibraryMapping();
            assertNotNull(mapping, "libraryMapping must not be null");
            assertFalse(mapping.isEmpty(), "libraryMapping must not be empty");
        }

        @Test
        @DisplayName("libraryName is consistent with platform support")
        void libraryNameConsistentWithSupport() {
            NativeLibraryLoader.DiagnosticInfo info = NativeLibraryLoader.getDiagnosticInfo();
            if (info.isPlatformSupported()) {
                assertNotNull(info.getLibraryName(),
                    "libraryName must not be null for a supported platform");
                assertFalse(info.getLibraryName().isEmpty(),
                    "libraryName must not be empty for a supported platform");
            } else {
                assertNull(info.getLibraryName(),
                    "libraryName should be null for an unsupported platform");
            }
        }

        @Test
        @DisplayName("libraryAvailability is non-null")
        void libraryAvailabilityNonNull() {
            NativeLibraryLoader.DiagnosticInfo info = NativeLibraryLoader.getDiagnosticInfo();
            assertNotNull(info.getLibraryAvailability(),
                "libraryAvailability must not be null");
        }

        @Test
        @DisplayName("cleanupInfo is non-null")
        void cleanupInfoNonNull() {
            NativeLibraryLoader.DiagnosticInfo info = NativeLibraryLoader.getDiagnosticInfo();
            assertNotNull(info.getCleanupInfo(), "cleanupInfo must not be null");
        }

        @Test
        @DisplayName("generateReport() returns non-empty string with key sections")
        void generateReportContainsKeySections() {
            NativeLibraryLoader.DiagnosticInfo info = NativeLibraryLoader.getDiagnosticInfo();
            String report = info.generateReport();

            assertNotNull(report, "report must not be null");
            assertFalse(report.isEmpty(), "report must not be empty");

            assertTrue(report.contains("Platform"), "report should mention Platform");
            assertTrue(report.contains("Java Environment"), "report should mention Java Environment");
            assertTrue(report.contains("Environment Variables"), "report should mention Environment Variables");
            assertTrue(report.contains("Library Availability"), "report should mention Library Availability");
            assertTrue(report.contains("Cleanup State"), "report should mention Cleanup State");
            assertTrue(report.contains("Debug Settings"), "report should mention Debug Settings");
        }

        @Test
        @DisplayName("generateReport() includes actual platform value")
        void generateReportIncludesPlatform() {
            NativeLibraryLoader.DiagnosticInfo info = NativeLibraryLoader.getDiagnosticInfo();
            String report = info.generateReport();
            assertTrue(report.contains(info.getPlatform()),
                "report should include the detected platform string");
        }

        @Test
        @DisplayName("generateReport() includes Java version")
        void generateReportIncludesJavaVersion() {
            NativeLibraryLoader.DiagnosticInfo info = NativeLibraryLoader.getDiagnosticInfo();
            String report = info.generateReport();
            assertTrue(report.contains(info.getJavaVersion()),
                "report should include the Java version");
        }

        @Test
        @DisplayName("toString() returns non-empty string")
        void toStringNonEmpty() {
            NativeLibraryLoader.DiagnosticInfo info = NativeLibraryLoader.getDiagnosticInfo();
            String str = info.toString();
            assertNotNull(str);
            assertFalse(str.isEmpty());
            assertTrue(str.contains("DiagnosticInfo"), "toString should identify the class");
        }
    }

    // ── checkLibraryAvailability ───────────────────────────────────────

    @Nested
    @DisplayName("checkLibraryAvailability()")
    class CheckLibraryAvailabilityTests {

        @Test
        @DisplayName("Returns non-null LibraryAvailabilityInfo")
        void returnsNonNull() {
            NativeLibraryLoader.LibraryAvailabilityInfo info = NativeLibraryLoader.checkLibraryAvailability();
            assertNotNull(info, "checkLibraryAvailability() must not return null");
        }

        @Test
        @DisplayName("Platform field matches detected platform")
        void platformMatchesDetected() {
            NativeLibraryLoader.LibraryAvailabilityInfo info = NativeLibraryLoader.checkLibraryAvailability();
            assertEquals(NativeLibraryLoader.getPlatform(), info.getPlatform(),
                "LibraryAvailabilityInfo.platform should match NativeLibraryLoader.getPlatform()");
        }

        @Test
        @DisplayName("Results map is non-null and non-empty for supported platform")
        void resultsNonEmptyForSupportedPlatform() {
            if (!NativeLibraryLoader.isPlatformSupported()) return;

            NativeLibraryLoader.LibraryAvailabilityInfo info = NativeLibraryLoader.checkLibraryAvailability();
            Map<String, String> results = info.getResults();
            assertNotNull(results, "results map must not be null");
            assertFalse(results.isEmpty(), "results map must not be empty for supported platform");
        }

        @Test
        @DisplayName("Results map is non-null for unsupported platform")
        void resultsNonNullForUnsupportedPlatform() {
            NativeLibraryLoader.LibraryAvailabilityInfo info = NativeLibraryLoader.checkLibraryAvailability();
            assertNotNull(info.getResults(), "results map must not be null even for unsupported platform");
        }

        @Test
        @DisplayName("libraryName is consistent with platform support")
        void libraryNameConsistentWithSupport() {
            NativeLibraryLoader.LibraryAvailabilityInfo info = NativeLibraryLoader.checkLibraryAvailability();
            if (NativeLibraryLoader.isPlatformSupported()) {
                assertNotNull(info.getLibraryName(),
                    "libraryName must not be null for supported platform");
            } else {
                assertNull(info.getLibraryName(),
                    "libraryName should be null for unsupported platform");
            }
        }

        @Test
        @DisplayName("hasAnyAvailableLibrary() is consistent with bundled/system flags")
        void hasAnyAvailableLibraryConsistent() {
            NativeLibraryLoader.LibraryAvailabilityInfo info = NativeLibraryLoader.checkLibraryAvailability();
            boolean expected = info.isBundledLibraryAvailable() || info.isSystemLibraryAvailable();
            assertEquals(expected, info.hasAnyAvailableLibrary(),
                "hasAnyAvailableLibrary() should be true iff bundled or system library is available");
        }

        @Test
        @DisplayName("toString() returns non-empty string")
        void toStringNonEmpty() {
            NativeLibraryLoader.LibraryAvailabilityInfo info = NativeLibraryLoader.checkLibraryAvailability();
            String str = info.toString();
            assertNotNull(str);
            assertFalse(str.isEmpty());
            assertTrue(str.contains("LibraryAvailabilityInfo"), "toString should identify the class");
        }
    }

    // ── getTroubleshootingSuggestions ──────────────────────────────────

    @Nested
    @DisplayName("getTroubleshootingSuggestions()")
    class GetTroubleshootingSuggestionsTests {

        @Test
        @DisplayName("Returns non-null TroubleshootingInfo")
        void returnsNonNull() {
            NativeLibraryLoader.TroubleshootingInfo info = NativeLibraryLoader.getTroubleshootingSuggestions();
            assertNotNull(info, "getTroubleshootingSuggestions() must not return null");
        }

        @Test
        @DisplayName("Items list is non-null")
        void itemsListNonNull() {
            NativeLibraryLoader.TroubleshootingInfo info = NativeLibraryLoader.getTroubleshootingSuggestions();
            assertNotNull(info.getItems(), "items list must not be null");
        }

        @Test
        @DisplayName("At least one item is returned")
        void atLeastOneItem() {
            NativeLibraryLoader.TroubleshootingInfo info = NativeLibraryLoader.getTroubleshootingSuggestions();
            assertFalse(info.getItems().isEmpty(),
                "getTroubleshootingSuggestions() should return at least one item");
        }

        @Test
        @DisplayName("Each item has non-null title and description")
        void itemsHaveNonNullTitleAndDescription() {
            NativeLibraryLoader.TroubleshootingInfo info = NativeLibraryLoader.getTroubleshootingSuggestions();
            for (NativeLibraryLoader.TroubleshootingInfo.Item item : info.getItems()) {
                assertNotNull(item.getTitle(), "item title must not be null");
                assertFalse(item.getTitle().isEmpty(), "item title must not be empty");
                assertNotNull(item.getDescription(), "item description must not be null");
                assertNotNull(item.getSeverity(), "item severity must not be null");
                assertNotNull(item.getActions(), "item actions must not be null");
            }
        }

        @Test
        @DisplayName("Unsupported platform produces a CRITICAL issue")
        void unsupportedPlatformProducesCriticalIssue() {
            if (NativeLibraryLoader.isPlatformSupported()) return; // skip on supported platforms

            NativeLibraryLoader.TroubleshootingInfo info = NativeLibraryLoader.getTroubleshootingSuggestions();
            assertTrue(info.hasCriticalIssues(),
                "Unsupported platform should produce at least one CRITICAL issue");
        }

        @Test
        @DisplayName("getCriticalIssues() returns only CRITICAL severity items")
        void getCriticalIssuesOnlyCritical() {
            NativeLibraryLoader.TroubleshootingInfo info = NativeLibraryLoader.getTroubleshootingSuggestions();
            for (NativeLibraryLoader.TroubleshootingInfo.Item item : info.getCriticalIssues()) {
                assertEquals(NativeLibraryLoader.TroubleshootingInfo.Severity.CRITICAL, item.getSeverity(),
                    "getCriticalIssues() should only return CRITICAL items");
            }
        }

        @Test
        @DisplayName("getWarnings() returns only WARNING severity items")
        void getWarningsOnlyWarning() {
            NativeLibraryLoader.TroubleshootingInfo info = NativeLibraryLoader.getTroubleshootingSuggestions();
            for (NativeLibraryLoader.TroubleshootingInfo.Item item : info.getWarnings()) {
                assertEquals(NativeLibraryLoader.TroubleshootingInfo.Severity.WARNING, item.getSeverity(),
                    "getWarnings() should only return WARNING items");
            }
        }

        @Test
        @DisplayName("generateReport() returns non-empty string with header")
        void generateReportNonEmpty() {
            NativeLibraryLoader.TroubleshootingInfo info = NativeLibraryLoader.getTroubleshootingSuggestions();
            String report = info.generateReport();
            assertNotNull(report, "report must not be null");
            assertFalse(report.isEmpty(), "report must not be empty");
            assertTrue(report.contains("Troubleshooting"), "report should contain 'Troubleshooting'");
        }

        @Test
        @DisplayName("generateReport() includes item titles")
        void generateReportIncludesItemTitles() {
            NativeLibraryLoader.TroubleshootingInfo info = NativeLibraryLoader.getTroubleshootingSuggestions();
            String report = info.generateReport();
            for (NativeLibraryLoader.TroubleshootingInfo.Item item : info.getItems()) {
                assertTrue(report.contains(item.getTitle()),
                    "report should include item title: " + item.getTitle());
            }
        }

        @Test
        @DisplayName("toString() returns non-empty string")
        void toStringNonEmpty() {
            NativeLibraryLoader.TroubleshootingInfo info = NativeLibraryLoader.getTroubleshootingSuggestions();
            String str = info.toString();
            assertNotNull(str);
            assertFalse(str.isEmpty());
            assertTrue(str.contains("TroubleshootingInfo"), "toString should identify the class");
        }
    }

    // ── validateSystemConfiguration ───────────────────────────────────

    @Nested
    @DisplayName("validateSystemConfiguration()")
    class ValidateSystemConfigurationTests {

        @Test
        @DisplayName("Returns non-null ValidationResult")
        void returnsNonNull() {
            NativeLibraryLoader.ValidationResult result = NativeLibraryLoader.validateSystemConfiguration();
            assertNotNull(result, "validateSystemConfiguration() must not return null");
        }

        @Test
        @DisplayName("Checks list is non-null and non-empty")
        void checksListNonEmpty() {
            NativeLibraryLoader.ValidationResult result = NativeLibraryLoader.validateSystemConfiguration();
            assertNotNull(result.getChecks(), "checks list must not be null");
            assertFalse(result.getChecks().isEmpty(), "checks list must not be empty");
        }

        @Test
        @DisplayName("Each check has non-null name and detail")
        void checksHaveNonNullNameAndDetail() {
            NativeLibraryLoader.ValidationResult result = NativeLibraryLoader.validateSystemConfiguration();
            for (NativeLibraryLoader.ValidationResult.Check check : result.getChecks()) {
                assertNotNull(check.getName(), "check name must not be null");
                assertFalse(check.getName().isEmpty(), "check name must not be empty");
                assertNotNull(check.getDetail(), "check detail must not be null");
            }
        }

        @Test
        @DisplayName("getFailedChecks() returns only failed checks")
        void getFailedChecksOnlyFailed() {
            NativeLibraryLoader.ValidationResult result = NativeLibraryLoader.validateSystemConfiguration();
            for (NativeLibraryLoader.ValidationResult.Check check : result.getFailedChecks()) {
                assertFalse(check.isPassed(),
                    "getFailedChecks() should only return checks where isPassed() is false");
            }
        }

        @Test
        @DisplayName("getPassedChecks() returns only passed checks")
        void getPassedChecksOnlyPassed() {
            NativeLibraryLoader.ValidationResult result = NativeLibraryLoader.validateSystemConfiguration();
            for (NativeLibraryLoader.ValidationResult.Check check : result.getPassedChecks()) {
                assertTrue(check.isPassed(),
                    "getPassedChecks() should only return checks where isPassed() is true");
            }
        }

        @Test
        @DisplayName("isValid() is true iff all checks pass")
        void isValidConsistentWithChecks() {
            NativeLibraryLoader.ValidationResult result = NativeLibraryLoader.validateSystemConfiguration();
            boolean allPassed = result.getChecks().stream()
                .allMatch(NativeLibraryLoader.ValidationResult.Check::isPassed);
            assertEquals(allPassed, result.isValid(),
                "isValid() should be true iff all checks pass");
        }

        @Test
        @DisplayName("Failed + passed counts equal total checks")
        void failedPlusPassedEqualsTotal() {
            NativeLibraryLoader.ValidationResult result = NativeLibraryLoader.validateSystemConfiguration();
            int total = result.getChecks().size();
            int passed = result.getPassedChecks().size();
            int failed = result.getFailedChecks().size();
            assertEquals(total, passed + failed,
                "passed + failed should equal total checks");
        }

        @Test
        @DisplayName("Platform detection check is present and passes")
        void platformDetectionCheckPresent() {
            NativeLibraryLoader.ValidationResult result = NativeLibraryLoader.validateSystemConfiguration();
            boolean hasPlatformCheck = result.getChecks().stream()
                .anyMatch(c -> c.getName().contains("Platform"));
            assertTrue(hasPlatformCheck, "Validation should include a Platform check");
        }

        @Test
        @DisplayName("Java version check is present")
        void javaVersionCheckPresent() {
            NativeLibraryLoader.ValidationResult result = NativeLibraryLoader.validateSystemConfiguration();
            boolean hasJavaCheck = result.getChecks().stream()
                .anyMatch(c -> c.getName().contains("Java"));
            assertTrue(hasJavaCheck, "Validation should include a Java environment check");
        }

        @Test
        @DisplayName("Temp directory check is present")
        void tempDirectoryCheckPresent() {
            NativeLibraryLoader.ValidationResult result = NativeLibraryLoader.validateSystemConfiguration();
            boolean hasTempCheck = result.getChecks().stream()
                .anyMatch(c -> c.getName().contains("Temp"));
            assertTrue(hasTempCheck, "Validation should include a temp directory check");
        }

        @Test
        @DisplayName("generateReport() returns non-empty string with key sections")
        void generateReportContainsKeySections() {
            NativeLibraryLoader.ValidationResult result = NativeLibraryLoader.validateSystemConfiguration();
            String report = result.generateReport();

            assertNotNull(report, "report must not be null");
            assertFalse(report.isEmpty(), "report must not be empty");
            assertTrue(report.contains("Validation"), "report should mention Validation");
            assertTrue(report.contains("PASS") || report.contains("FAIL"),
                "report should contain PASS or FAIL markers");
        }

        @Test
        @DisplayName("generateReport() includes all check names")
        void generateReportIncludesCheckNames() {
            NativeLibraryLoader.ValidationResult result = NativeLibraryLoader.validateSystemConfiguration();
            String report = result.generateReport();
            for (NativeLibraryLoader.ValidationResult.Check check : result.getChecks()) {
                assertTrue(report.contains(check.getName()),
                    "report should include check name: " + check.getName());
            }
        }

        @Test
        @DisplayName("toString() returns non-empty string")
        void toStringNonEmpty() {
            NativeLibraryLoader.ValidationResult result = NativeLibraryLoader.validateSystemConfiguration();
            String str = result.toString();
            assertNotNull(str);
            assertFalse(str.isEmpty());
            assertTrue(str.contains("ValidationResult"), "toString should identify the class");
        }
    }

    // ── Integration: all diagnostics together ─────────────────────────

    @Nested
    @DisplayName("Diagnostics Integration")
    class DiagnosticsIntegrationTests {

        @Test
        @DisplayName("All diagnostic methods return consistent platform information")
        void allMethodsConsistentPlatform() {
            String platform = NativeLibraryLoader.getPlatform();

            NativeLibraryLoader.DiagnosticInfo diagInfo = NativeLibraryLoader.getDiagnosticInfo();
            NativeLibraryLoader.LibraryAvailabilityInfo availInfo = NativeLibraryLoader.checkLibraryAvailability();

            assertEquals(platform, diagInfo.getPlatform(),
                "DiagnosticInfo.platform should match NativeLibraryLoader.getPlatform()");
            assertEquals(platform, availInfo.getPlatform(),
                "LibraryAvailabilityInfo.platform should match NativeLibraryLoader.getPlatform()");
        }

        @Test
        @DisplayName("DiagnosticInfo.libraryAvailability matches standalone checkLibraryAvailability()")
        void diagnosticInfoAvailabilityMatchesStandalone() {
            NativeLibraryLoader.DiagnosticInfo diagInfo = NativeLibraryLoader.getDiagnosticInfo();
            NativeLibraryLoader.LibraryAvailabilityInfo standalone = NativeLibraryLoader.checkLibraryAvailability();

            // Both should agree on bundled/system availability
            assertEquals(standalone.isBundledLibraryAvailable(),
                diagInfo.getLibraryAvailability().isBundledLibraryAvailable(),
                "bundled availability should be consistent");
            assertEquals(standalone.isSystemLibraryAvailable(),
                diagInfo.getLibraryAvailability().isSystemLibraryAvailable(),
                "system availability should be consistent");
        }

        @Test
        @DisplayName("Diagnostic methods are callable multiple times without error")
        void diagnosticMethodsIdempotent() {
            // Call each method twice to verify no side effects
            assertDoesNotThrow(() -> {
                NativeLibraryLoader.getDiagnosticInfo();
                NativeLibraryLoader.getDiagnosticInfo();
            }, "getDiagnosticInfo() should be callable multiple times");

            assertDoesNotThrow(() -> {
                NativeLibraryLoader.checkLibraryAvailability();
                NativeLibraryLoader.checkLibraryAvailability();
            }, "checkLibraryAvailability() should be callable multiple times");

            assertDoesNotThrow(() -> {
                NativeLibraryLoader.getTroubleshootingSuggestions();
                NativeLibraryLoader.getTroubleshootingSuggestions();
            }, "getTroubleshootingSuggestions() should be callable multiple times");

            assertDoesNotThrow(() -> {
                NativeLibraryLoader.validateSystemConfiguration();
                NativeLibraryLoader.validateSystemConfiguration();
            }, "validateSystemConfiguration() should be callable multiple times");
        }

        @Test
        @DisplayName("Diagnostic report contains system Java version")
        void diagnosticReportContainsJavaVersion() {
            String javaVersion = System.getProperty("java.version");
            NativeLibraryLoader.DiagnosticInfo info = NativeLibraryLoader.getDiagnosticInfo();
            String report = info.generateReport();
            assertTrue(report.contains(javaVersion),
                "Diagnostic report should include the current Java version: " + javaVersion);
        }

        @Test
        @DisplayName("Diagnostic report contains temp directory path")
        void diagnosticReportContainsTempDir() {
            String tempDir = System.getProperty("java.io.tmpdir");
            NativeLibraryLoader.DiagnosticInfo info = NativeLibraryLoader.getDiagnosticInfo();
            String report = info.generateReport();
            assertTrue(report.contains(tempDir),
                "Diagnostic report should include the temp directory path");
        }
    }
}
