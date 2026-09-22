package dev.rekall.claude;

import lombok.RequiredArgsConstructor;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.web.socket.config.annotation.EnableWebSocket;
import org.springframework.web.socket.config.annotation.WebSocketConfigurer;
import org.springframework.web.socket.config.annotation.WebSocketHandlerRegistry;
import org.springframework.web.socket.server.standard.ServletServerContainerFactoryBean;

/**
 * Registers Rekall's one WebSocket endpoint, the terminal byte pipe. A browser does not apply
 * CORS to a WebSocket, so the handshake is only accepted from a loopback origin: the packaged
 * WebView on 127.0.0.1 and the Vite dev server on localhost. Any other site open in the same
 * browser is refused before it can type into a terminal.
 */
@Configuration
@EnableWebSocket
@RequiredArgsConstructor
public class TerminalSocketConfig implements WebSocketConfigurer {

    private static final int MAX_BINARY_FRAME_BYTES = 524_288;
    private static final int MAX_TEXT_FRAME_BYTES = 65_536;

    private static final String[] LOOPBACK_ORIGINS = {
            "http://localhost:[*]", "http://127.0.0.1:[*]", "http://[::1]:[*]",
            "http://localhost", "http://127.0.0.1", "http://[::1]"
    };

    private final TerminalSocketHandler handler;

    @Override
    public void registerWebSocketHandlers(WebSocketHandlerRegistry registry) {
        registry.addHandler(handler, "/api/terminal/*/io").setAllowedOriginPatterns(LOOPBACK_ORIGINS);
    }

    @Bean
    ServletServerContainerFactoryBean terminalWebSocketContainer() {
        ServletServerContainerFactoryBean container = new ServletServerContainerFactoryBean();
        container.setMaxBinaryMessageBufferSize(MAX_BINARY_FRAME_BYTES);
        container.setMaxTextMessageBufferSize(MAX_TEXT_FRAME_BYTES);
        return container;
    }
}
