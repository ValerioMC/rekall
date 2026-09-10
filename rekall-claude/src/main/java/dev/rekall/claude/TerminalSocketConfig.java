package dev.rekall.claude;

import lombok.RequiredArgsConstructor;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.web.socket.config.annotation.EnableWebSocket;
import org.springframework.web.socket.config.annotation.WebSocketConfigurer;
import org.springframework.web.socket.config.annotation.WebSocketHandlerRegistry;
import org.springframework.web.socket.server.standard.ServletServerContainerFactoryBean;

/**
 * Registers Rekall's one WebSocket endpoint, the terminal byte pipe. Origins are open: the only
 * clients are the packaged WebView and the Vite dev proxy.
 */
@Configuration
@EnableWebSocket
@RequiredArgsConstructor
public class TerminalSocketConfig implements WebSocketConfigurer {

    private static final int MAX_BINARY_FRAME_BYTES = 524_288;
    private static final int MAX_TEXT_FRAME_BYTES = 65_536;

    private final TerminalSocketHandler handler;

    @Override
    public void registerWebSocketHandlers(WebSocketHandlerRegistry registry) {
        registry.addHandler(handler, "/api/terminal/*/io").setAllowedOriginPatterns("*");
    }

    @Bean
    ServletServerContainerFactoryBean terminalWebSocketContainer() {
        ServletServerContainerFactoryBean container = new ServletServerContainerFactoryBean();
        container.setMaxBinaryMessageBufferSize(MAX_BINARY_FRAME_BYTES);
        container.setMaxTextMessageBufferSize(MAX_TEXT_FRAME_BYTES);
        return container;
    }
}
