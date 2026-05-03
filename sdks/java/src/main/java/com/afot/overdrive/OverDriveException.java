package com.afot.overdrive;

import java.util.Collections;
import java.util.List;

/**
 * Base exception for all OverDrive SDK errors.
 *
 * <p>Each exception carries a machine-readable error code (e.g. {@code ODB-AUTH-001}),
 * human-readable context, actionable suggestions, and a documentation link.</p>
 */
public class OverDriveException extends RuntimeException {

    private final String code;
    private final String context;
    private final List<String> suggestions;
    private final String docLink;

    public OverDriveException(String message) {
        this(message, "", "", Collections.emptyList(), "");
    }

    public OverDriveException(String message, Throwable cause) {
        super(message, cause);
        this.code = "";
        this.context = "";
        this.suggestions = Collections.emptyList();
        this.docLink = "";
    }

    public OverDriveException(String message, String code, String context,
                              List<String> suggestions, String docLink) {
        super(formatMessage(message, code, context, suggestions, docLink));
        this.code = code != null ? code : "";
        this.context = context != null ? context : "";
        this.suggestions = suggestions != null ? suggestions : Collections.emptyList();
        this.docLink = docLink != null ? docLink : "";
    }

    public String getCode() { return code; }
    public String getContext() { return context; }
    public List<String> getSuggestions() { return suggestions; }
    public String getDocLink() { return docLink; }

    private static String formatMessage(String message, String code, String context,
                                        List<String> suggestions, String docLink) {
        StringBuilder sb = new StringBuilder();
        if (code != null && !code.isEmpty()) {
            sb.append("Error ").append(code).append(": ");
        }
        sb.append(message);
        if (context != null && !context.isEmpty()) {
            sb.append("\nContext: ").append(context);
        }
        if (suggestions != null && !suggestions.isEmpty()) {
            sb.append("\nSuggestions:");
            for (String s : suggestions) {
                sb.append("\n  \u2022 ").append(s);
            }
        }
        if (docLink != null && !docLink.isEmpty()) {
            sb.append("\nFor more help: ").append(docLink);
        }
        return sb.toString();
    }

    // ── Specific Exception Subclasses ──────────────

    /** Authentication / encryption errors (ODB-AUTH-*). */
    public static class AuthenticationException extends OverDriveException {
        public AuthenticationException(String msg, String code, String ctx,
                                       List<String> suggestions, String doc) {
            super(msg, code, ctx, suggestions, doc);
        }
    }

    /** Table operation errors (ODB-TABLE-*). */
    public static class TableException extends OverDriveException {
        public TableException(String msg, String code, String ctx,
                              List<String> suggestions, String doc) {
            super(msg, code, ctx, suggestions, doc);
        }
    }

    /** Query execution errors (ODB-QUERY-*). */
    public static class QueryException extends OverDriveException {
        public QueryException(String msg, String code, String ctx,
                              List<String> suggestions, String doc) {
            super(msg, code, ctx, suggestions, doc);
        }
    }

    /** Transaction errors (ODB-TXN-*). */
    public static class TransactionException extends OverDriveException {
        public TransactionException(String msg, String code, String ctx,
                                    List<String> suggestions, String doc) {
            super(msg, code, ctx, suggestions, doc);
        }
    }

    /** File I/O errors (ODB-IO-*). */
    public static class IOError extends OverDriveException {
        public IOError(String msg, String code, String ctx,
                       List<String> suggestions, String doc) {
            super(msg, code, ctx, suggestions, doc);
        }
    }

    /** Native library / FFI errors (ODB-FFI-*). */
    public static class FFIException extends OverDriveException {
        public FFIException(String msg, String code, String ctx,
                            List<String> suggestions, String doc) {
            super(msg, code, ctx, suggestions, doc);
        }
    }
}
