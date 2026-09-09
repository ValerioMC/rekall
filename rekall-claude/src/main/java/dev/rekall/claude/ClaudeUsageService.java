package dev.rekall.claude;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import dev.rekall.claude.ClaudeUsageView.Limit;
import dev.rekall.claude.ClaudeUsageView.Severity;
import lombok.extern.slf4j.Slf4j;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Service;

import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.time.Duration;
import java.time.Instant;
import java.time.OffsetDateTime;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

/**
 * Fetches the logged-in account's Claude usage from Anthropic and shapes it for the console meter.
 *
 * <p>This is the same figure Claude Code's own {@code /usage} shows: a GET to the OAuth usage
 * endpoint with the token {@link ClaudeCredentials} found. The response carries several named
 * windows; only the four the meter draws are kept, in a fixed order, and each is given a severity
 * from its own percentage so the whole panel escalates on one scale.
 *
 * <p>The result is cached for {@link #CACHE_TTL}: the underlying numbers move in minutes, not
 * seconds, and every open console polls this. A call that fails after a good one has been seen
 * returns that last good one rather than an error, so a blip does not blank the meter.
 */
@Service
@Slf4j
public class ClaudeUsageService {

    private static final Duration CACHE_TTL = Duration.ofSeconds(60);
    private static final Duration HTTP_TIMEOUT = Duration.ofSeconds(10);
    private static final double WARNING_AT = 80.0;
    private static final double CRITICAL_AT = 95.0;

    /** The named windows the meter draws, mapped to their response key and display label. */
    private static final Map<String, String[]> WINDOWS = new LinkedHashMap<>();

    static {
        WINDOWS.put("five_hour", new String[] {"session", "Session"});
        WINDOWS.put("seven_day", new String[] {"weekly_all", "Weekly · all models"});
        WINDOWS.put("seven_day_opus", new String[] {"weekly_opus", "Weekly · Opus"});
        WINDOWS.put("seven_day_sonnet", new String[] {"weekly_sonnet", "Weekly · Sonnet"});
    }

    private final ClaudeCredentials credentials;
    private final ObjectMapper mapper = new ObjectMapper();
    private final HttpClient http = HttpClient.newBuilder().connectTimeout(HTTP_TIMEOUT).build();
    private final String usageUrl;

    private ClaudeUsageView cached;
    private Instant cachedAt = Instant.EPOCH;
    private ClaudeUsageView lastGood;

    public ClaudeUsageService(
            ClaudeCredentials credentials,
            @Value("${rekall.claude.usage-url:https://api.anthropic.com/api/oauth/usage}") String usageUrl) {
        this.credentials = credentials;
        this.usageUrl = usageUrl;
    }

    public synchronized ClaudeUsageView current() {
        if (cached != null && Duration.between(cachedAt, Instant.now()).compareTo(CACHE_TTL) < 0) {
            return cached;
        }
        ClaudeUsageView fresh = fetch();
        cached = fresh;
        cachedAt = Instant.now();
        if (fresh.status() == ClaudeUsageView.Status.OK) {
            lastGood = fresh;
        }
        return fresh;
    }

    private ClaudeUsageView fetch() {
        Optional<String> token = credentials.accessToken();
        if (token.isEmpty()) {
            return ClaudeUsageView.unauthenticated();
        }
        try {
            HttpRequest request = HttpRequest.newBuilder(URI.create(usageUrl))
                    .timeout(HTTP_TIMEOUT)
                    .header("Authorization", "Bearer " + token.get())
                    .header("anthropic-beta", "oauth-2025-04-20")
                    .header("Accept", "application/json")
                    .GET()
                    .build();
            HttpResponse<String> response = http.send(request, HttpResponse.BodyHandlers.ofString());
            if (response.statusCode() == 401 || response.statusCode() == 403) {
                return ClaudeUsageView.unauthenticated();
            }
            if (response.statusCode() != 200) {
                log.debug("Claude usage endpoint returned {}", response.statusCode());
                return degraded();
            }
            return new ClaudeUsageView(ClaudeUsageView.Status.OK, parse(response.body()), Instant.now());
        } catch (IllegalArgumentException | java.io.IOException failure) {
            log.debug("Claude usage endpoint unreachable: {}", failure.getMessage());
            return degraded();
        } catch (InterruptedException interrupted) {
            Thread.currentThread().interrupt();
            return degraded();
        }
    }

    private ClaudeUsageView degraded() {
        return lastGood != null ? lastGood : ClaudeUsageView.unavailable();
    }

    private List<Limit> parse(String body) throws java.io.IOException {
        JsonNode root = mapper.readTree(body);
        List<Limit> limits = new ArrayList<>();
        for (Map.Entry<String, String[]> window : WINDOWS.entrySet()) {
            JsonNode node = root.path(window.getKey());
            if (node.isMissingNode() || node.isNull() || !node.hasNonNull("utilization")) {
                continue;
            }
            double percent = clamp(node.path("utilization").asDouble(0));
            limits.add(new Limit(
                    window.getValue()[0],
                    window.getValue()[1],
                    percent,
                    severityOf(percent),
                    instantOrNull(node.path("resets_at").asText(null))));
        }
        return List.copyOf(limits);
    }

    private static Severity severityOf(double percent) {
        if (percent >= CRITICAL_AT) {
            return Severity.CRITICAL;
        }
        return percent >= WARNING_AT ? Severity.WARNING : Severity.NORMAL;
    }

    private static double clamp(double value) {
        return Math.max(0, Math.min(100, value));
    }

    private static Instant instantOrNull(String text) {
        if (text == null || text.isBlank() || "null".equals(text)) {
            return null;
        }
        try {
            return OffsetDateTime.parse(text).toInstant();
        } catch (RuntimeException notADate) {
            return null;
        }
    }
}
