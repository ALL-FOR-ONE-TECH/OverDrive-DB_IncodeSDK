package com.afot.overdrive;

/**
 * Exception thrown by OverDrive operations.
 */
public class OverDriveException extends RuntimeException {
    public OverDriveException(String message) {
        super(message);
    }

    public OverDriveException(String message, Throwable cause) {
        super(message, cause);
    }
}
