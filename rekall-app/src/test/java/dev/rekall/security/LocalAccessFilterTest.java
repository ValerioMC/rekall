package dev.rekall.security;

import jakarta.servlet.FilterChain;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.ValueSource;
import org.springframework.mock.web.MockFilterChain;
import org.springframework.mock.web.MockHttpServletRequest;
import org.springframework.mock.web.MockHttpServletResponse;

import static org.assertj.core.api.Assertions.assertThat;

class LocalAccessFilterTest {

    private final LocalAccessFilter local = new LocalAccessFilter(false);

    @Test
    @DisplayName("the console on 127.0.0.1 and Claude Code on localhost both pass")
    void loopbackClientsPass() throws Exception {
        assertThat(status(local, request("127.0.0.1", "127.0.0.1:47355", "http://127.0.0.1:47355"))).isEqualTo(200);
        assertThat(status(local, request("0:0:0:0:0:0:0:1", "localhost:47355", null))).isEqualTo(200);
        assertThat(status(local, request("127.0.0.1", "localhost:47355", "http://localhost:5173"))).isEqualTo(200);
        assertThat(status(local, request("::1", "[::1]:47355", "http://[::1]:47355"))).isEqualTo(200);
    }

    @Test
    @DisplayName("a machine on the same network is refused, whatever it claims in its headers")
    void anotherMachineIsRefused() throws Exception {
        assertThat(status(local, request("192.168.1.20", "localhost:47355", null))).isEqualTo(403);
    }

    @ParameterizedTest
    @ValueSource(strings = {"evil.example:47355", "rebind.attacker.test", "192.168.1.5:47355"})
    @DisplayName("a Host that is not this machine is refused: that is DNS rebinding")
    void aForeignHostIsRefused(String host) throws Exception {
        assertThat(status(local, request("127.0.0.1", host, null))).isEqualTo(403);
    }

    @ParameterizedTest
    @ValueSource(strings = {"https://evil.example", "null", "file://", "http://localhost.evil.example"})
    @DisplayName("an Origin from another site is refused: that is a page in the same browser")
    void aForeignOriginIsRefused(String origin) throws Exception {
        assertThat(status(local, request("127.0.0.1", "127.0.0.1:47355", origin))).isEqualTo(403);
    }

    @Test
    @DisplayName("with remote access on, another machine and its own origin pass, but a third site still does not")
    void remoteAccessKeepsTheOriginCheck() throws Exception {
        LocalAccessFilter remote = new LocalAccessFilter(true);

        assertThat(status(remote, request("192.168.1.20", "192.168.1.10:47355", "http://192.168.1.10:47355")))
                .isEqualTo(200);
        assertThat(status(remote, request("192.168.1.20", "192.168.1.10:47355", "https://evil.example")))
                .isEqualTo(403);
    }

    @Test
    @DisplayName("the host is read without its port, bracketed IPv6 included")
    void theHostIsReadWithoutItsPort() {
        assertThat(LocalAccessFilter.hostName("localhost:47355")).isEqualTo("localhost");
        assertThat(LocalAccessFilter.hostName("[::1]:47355")).isEqualTo("[::1]");
        assertThat(LocalAccessFilter.hostName("127.0.0.1")).isEqualTo("127.0.0.1");
    }

    private static MockHttpServletRequest request(String remoteAddress, String host, String origin) {
        MockHttpServletRequest request = new MockHttpServletRequest("POST", "/api/tasks");
        request.setRemoteAddr(remoteAddress);
        request.addHeader("Host", host);
        if (origin != null) {
            request.addHeader("Origin", origin);
        }
        return request;
    }

    private static int status(LocalAccessFilter filter, MockHttpServletRequest request) throws Exception {
        MockHttpServletResponse response = new MockHttpServletResponse();
        FilterChain chain = new MockFilterChain();
        filter.doFilter(request, response, chain);
        return response.getStatus();
    }
}
