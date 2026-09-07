package dev.rekall.mcp.protocol;

import java.util.Arrays;
import java.util.List;
import java.util.Optional;

public enum ProtocolVersion {

    V2026_07_28("2026-07-28", Era.MODERN, true),

    V2025_11_25("2025-11-25", Era.LEGACY, true),
    V2025_06_18("2025-06-18", Era.LEGACY, true),
    V2025_03_26("2025-03-26", Era.LEGACY, true),

    V2024_11_05("2024-11-05", Era.LEGACY, false);

    public enum Era {
        LEGACY,
        MODERN
    }

    public static final ProtocolVersion ASSUMED_WHEN_HEADER_ABSENT = V2025_03_26;

    private final String wire;
    private final Era era;
    private final boolean advertised;

    ProtocolVersion(String wire, Era era, boolean advertised) {
        this.wire = wire;
        this.era = era;
        this.advertised = advertised;
    }

    public String wire() {
        return wire;
    }

    public Era era() {
        return era;
    }

    public boolean isModern() {
        return era == Era.MODERN;
    }

    public static Optional<ProtocolVersion> parse(String wire) {
        return Arrays.stream(values()).filter(version -> version.wire.equals(wire)).findFirst();
    }

    public static List<String> advertisedVersions() {
        return Arrays.stream(values())
                .filter(version -> version.advertised)
                .map(ProtocolVersion::wire)
                .toList();
    }

    public static ProtocolVersion latestLegacy() {
        return V2025_11_25;
    }
}
