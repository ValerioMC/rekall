package dev.rekall.security;

import jakarta.servlet.FilterChain;
import jakarta.servlet.ServletException;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;
import lombok.extern.slf4j.Slf4j;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;
import org.springframework.http.HttpHeaders;
import org.springframework.http.HttpStatus;
import org.springframework.stereotype.Component;
import org.springframework.web.filter.OncePerRequestFilter;

import java.io.IOException;
import java.net.InetAddress;
import java.net.URI;
import java.net.UnknownHostException;
import java.util.Locale;
import java.util.Set;
import java.util.regex.Pattern;

/**
 * Keeps Rekall a single-user, localhost application. Nothing here authenticates, and the API can
 * open a {@code claude} terminal in a project folder, so every request has to prove it comes from
 * this machine and from a page this machine serves. Three checks, all refused with a 403:
 *
 * <ul>
 *   <li>The peer address is loopback. The server listens on every interface so that both
 *       {@code 127.0.0.1} and {@code ::1} answer whichever one a client resolves {@code localhost}
 *       to; this is what stops a machine on the same network.</li>
 *   <li>The {@code Host} header names a loopback host. A page on another domain that rebinds its
 *       DNS to {@code 127.0.0.1} still sends its own name here, so this is what stops DNS
 *       rebinding.</li>
 *   <li>An {@code Origin} header, when there is one, is a loopback origin. A browser sends it on
 *       every cross-site write and WebSocket handshake, so this is what stops another site open in
 *       the same browser from driving the API. Clients that send none (Claude Code, curl) pass.</li>
 * </ul>
 *
 * <p>{@code rekall.security.remote-access=true} turns the first two off for someone who
 * deliberately serves Rekall to another machine. The origin check then also accepts the page's
 * own origin, the one its {@code Host} names, so the console served that way still works; that
 * mode gives up the protection against DNS rebinding and is off by default.
 */
@Slf4j
@Component
@Order(Ordered.HIGHEST_PRECEDENCE)
public class LocalAccessFilter extends OncePerRequestFilter {

    private static final Set<String> LOOPBACK_NAMES = Set.of("localhost", "127.0.0.1", "::1", "[::1]");

    private static final Pattern IPV4_LOOPBACK = Pattern.compile("^127(\\.\\d{1,3}){3}$");

    private final boolean remoteAccess;

    public LocalAccessFilter(@Value("${rekall.security.remote-access:false}") boolean remoteAccess) {
        this.remoteAccess = remoteAccess;
        if (remoteAccess) {
            log.warn("rekall.security.remote-access is on: requests from other machines are accepted");
        }
    }

    @Override
    protected void doFilterInternal(HttpServletRequest request, HttpServletResponse response, FilterChain chain)
            throws ServletException, IOException {
        String refusal = refusal(request);
        if (refusal != null) {
            log.warn("Refused {} {} from {}: {}", request.getMethod(), request.getRequestURI(),
                    request.getRemoteAddr(), refusal);
            response.sendError(HttpStatus.FORBIDDEN.value(), refusal);
            return;
        }
        chain.doFilter(request, response);
    }

    String refusal(HttpServletRequest request) {
        if (!remoteAccess && !isLoopbackAddress(request.getRemoteAddr())) {
            return "Rekall only answers requests from this machine";
        }
        String host = request.getHeader(HttpHeaders.HOST);
        if (!remoteAccess && host != null && !isLoopbackName(hostName(host))) {
            return "Host '" + host + "' is not this machine";
        }
        String origin = request.getHeader(HttpHeaders.ORIGIN);
        if (origin != null && !isLoopbackOrigin(origin) && !(remoteAccess && isSameOrigin(origin, host))) {
            return "Origin '" + origin + "' is not a page this machine serves";
        }
        return null;
    }

    static boolean isLoopbackAddress(String address) {
        if (address == null || address.isBlank()) {
            return false;
        }
        // The servlet container hands over an IP literal, so this never goes to DNS.
        try {
            return InetAddress.getByName(address).isLoopbackAddress();
        } catch (UnknownHostException e) {
            return false;
        }
    }

    static boolean isLoopbackOrigin(String origin) {
        try {
            URI uri = URI.create(origin.strip());
            String scheme = uri.getScheme();
            if (!"http".equalsIgnoreCase(scheme) && !"https".equalsIgnoreCase(scheme)) {
                return false;
            }
            return uri.getHost() != null && isLoopbackName(uri.getHost());
        } catch (IllegalArgumentException e) {
            return false;
        }
    }

    private static boolean isSameOrigin(String origin, String hostHeader) {
        if (hostHeader == null) {
            return false;
        }
        try {
            return hostHeader.strip().equalsIgnoreCase(URI.create(origin.strip()).getRawAuthority());
        } catch (IllegalArgumentException e) {
            return false;
        }
    }

    static String hostName(String hostHeader) {
        String value = hostHeader.strip();
        if (value.startsWith("[")) {
            int close = value.indexOf(']');
            return close < 0 ? value : value.substring(0, close + 1);
        }
        int colon = value.lastIndexOf(':');
        return colon < 0 ? value : value.substring(0, colon);
    }

    private static boolean isLoopbackName(String host) {
        String name = host.toLowerCase(Locale.ROOT);
        return LOOPBACK_NAMES.contains(name) || IPV4_LOOPBACK.matcher(name).matches();
    }
}
