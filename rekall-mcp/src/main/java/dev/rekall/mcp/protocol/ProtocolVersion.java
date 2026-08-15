package dev.rekall.mcp.protocol;

import java.util.Arrays;
import java.util.List;
import java.util.Optional;

/**
 * The MCP revisions this server answers, newest first.
 *
 * <p>MCP splits into two eras. Up to {@code 2025-11-25} a client opens with an
 * {@code initialize} handshake and the version agreed there holds for the rest of the session.
 * From {@code 2026-07-28} there is no handshake and no session: every request carries its own
 * version, in the {@code MCP-Protocol-Version} header and again in {@code params._meta}, and
 * the server accepts or rejects each one on its own.
 *
 * <p>Rekall answers both, which the specification calls a dual-era server. That costs one
 * branch in {@link dev.rekall.mcp.McpController} and nothing else, because this endpoint never
 * kept state between requests to begin with: the stateless model the new revision mandates is
 * how it already worked.
 */
public enum ProtocolVersion {

    /** Stateless, per-request metadata, mandatory {@code server/discover}. */
    V2026_07_28("2026-07-28", Era.MODERN, true),

    V2025_11_25("2025-11-25", Era.LEGACY, true),
    V2025_06_18("2025-06-18", Era.LEGACY, true),
    V2025_03_26("2025-03-26", Era.LEGACY, true),

    /**
     * Accepted, but not offered. This is the revision the endpoint used to pin, so a client
     * registered before this change still opens with it and has to keep working. It is left out
     * of {@code server/discover} because its transport was HTTP+SSE, which this server has never
     * spoken: a client free to choose should not choose this one.
     */
    V2024_11_05("2024-11-05", Era.LEGACY, false);

    /** Which side of the handshake removal a revision falls on. */
    public enum Era {
        LEGACY,
        MODERN
    }

    /**
     * What a request with no {@code MCP-Protocol-Version} header is read as. The header only
     * appeared in {@code 2025-06-18}, so its absence means an older client rather than a broken
     * one, and this is the reading the specification tells a server to assume.
     */
    public static final ProtocolVersion ASSUMED_WHEN_HEADER_ABSENT = V2025_03_26;

    private final String wire;
    private final Era era;
    private final boolean advertised;

    ProtocolVersion(String wire, Era era, boolean advertised) {
        this.wire = wire;
        this.era = era;
        this.advertised = advertised;
    }

    /** The revision string as it travels, e.g. {@code 2026-07-28}. */
    public String wire() {
        return wire;
    }

    public Era era() {
        return era;
    }

    public boolean isModern() {
        return era == Era.MODERN;
    }

    /** @return empty when the revision is unknown to this server, including for a null input */
    public static Optional<ProtocolVersion> parse(String wire) {
        return Arrays.stream(values()).filter(version -> version.wire.equals(wire)).findFirst();
    }

    /**
     * The revisions a client is invited to pick from, newest first. Used both by
     * {@code server/discover} and by the error that refuses an unknown version, because a client
     * reading either one is answering the same question: what can I say to this server?
     */
    public static List<String> advertisedVersions() {
        return Arrays.stream(values())
                .filter(version -> version.advertised)
                .map(ProtocolVersion::wire)
                .toList();
    }

    /**
     * The answer to an {@code initialize} that asked for something this server does not speak.
     * {@code initialize} exists only in the legacy era, so replying with the modern revision
     * would name a version the client that just asked cannot possibly use.
     */
    public static ProtocolVersion latestLegacy() {
        return V2025_11_25;
    }
}
