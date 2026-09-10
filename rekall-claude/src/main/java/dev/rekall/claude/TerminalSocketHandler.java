package dev.rekall.claude;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import dev.rekall.claude.TerminalApiDtos.TerminalView;
import dev.rekall.common.ConflictException;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Component;
import org.springframework.web.socket.BinaryMessage;
import org.springframework.web.socket.CloseStatus;
import org.springframework.web.socket.TextMessage;
import org.springframework.web.socket.WebSocketSession;
import org.springframework.web.socket.handler.AbstractWebSocketHandler;
import org.springframework.web.socket.handler.ConcurrentWebSocketSessionDecorator;

import java.io.IOException;
import java.nio.ByteBuffer;
import java.util.Arrays;
import java.util.Base64;
import java.util.Map;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

/**
 * The byte pipe for one open terminal pane. Binary frames are stdin, {@code {"resize":[cols,rows]}}
 * text frames set the window size, PTY output returns as binary frames, and a {@code {"type":"ended"}}
 * frame is sent on exit. Sends go through a {@link ConcurrentWebSocketSessionDecorator} because the
 * pump thread and the container both write; it also caps the outbound buffer.
 */
@Component
@Slf4j
public class TerminalSocketHandler extends AbstractWebSocketHandler {

    private static final Pattern ID_IN_PATH = Pattern.compile("/api/terminal/([0-9a-fA-F-]{36})/io");
    private static final int SEND_TIME_LIMIT_MS = 10_000;
    private static final int SEND_BUFFER_LIMIT_BYTES = 1_048_576;

    private final PtyTerminalManager manager;

    // Created directly: Spring Boot 4's context has no Jackson 2 ObjectMapper bean.
    private final ObjectMapper json = new ObjectMapper();

    private final Map<String, Attachment> attachments = new ConcurrentHashMap<>();

    public TerminalSocketHandler(PtyTerminalManager manager) {
        this.manager = manager;
    }

    private record Attachment(UUID terminalId, PtyTerminalManager.Listener listener) {
    }

    @Override
    public void afterConnectionEstablished(WebSocketSession rawSession) throws IOException {
        UUID terminalId = terminalIdOf(rawSession);
        if (terminalId == null) {
            rawSession.close(CloseStatus.BAD_DATA.withReason("Malformed terminal path."));
            return;
        }

        WebSocketSession session =
                new ConcurrentWebSocketSessionDecorator(rawSession, SEND_TIME_LIMIT_MS, SEND_BUFFER_LIMIT_BYTES);

        PtyTerminalManager.Listener listener = new PtyTerminalManager.Listener() {
            @Override
            public void output(byte[] data, int length) {
                if (!session.isOpen()) {
                    return;
                }
                try {
                    session.sendMessage(new BinaryMessage(ByteBuffer.wrap(Arrays.copyOf(data, length))));
                } catch (IOException sendFailed) {
                    closeQuietly(session, CloseStatus.SERVER_ERROR);
                }
            }

            @Override
            public void ended(int exitCode, String detail) {
                try {
                    if (session.isOpen()) {
                        session.sendMessage(new TextMessage(json.writeValueAsString(
                                Map.of("type", "ended", "exitCode", exitCode, "detail", detail == null ? "" : detail))));
                    }
                } catch (IOException ignored) {
                    // Closing anyway.
                }
                closeQuietly(session, CloseStatus.NORMAL);
            }
        };

        TerminalView view;
        try {
            view = manager.attach(terminalId, listener);
        } catch (ConflictException gone) {
            rawSession.close(CloseStatus.NOT_ACCEPTABLE.withReason(gone.getMessage()));
            return;
        }

        attachments.put(rawSession.getId(), new Attachment(terminalId, listener));
        session.sendMessage(new TextMessage(json.writeValueAsString(Map.of(
                "type", "ready",
                "id", view.id().toString(),
                "anchors", view.anchors(),
                "workingDir", view.workingDir()))));
    }

    @Override
    protected void handleBinaryMessage(WebSocketSession session, BinaryMessage message) {
        Attachment attachment = attachments.get(session.getId());
        if (attachment == null) {
            return;
        }
        ByteBuffer payload = message.getPayload();
        byte[] bytes = new byte[payload.remaining()];
        payload.get(bytes);
        feed(session, attachment, bytes);
    }

    @Override
    protected void handleTextMessage(WebSocketSession session, TextMessage message) {
        Attachment attachment = attachments.get(session.getId());
        if (attachment == null) {
            return;
        }
        JsonNode node;
        try {
            node = json.readTree(message.getPayload());
        } catch (IOException malformed) {
            return;
        }
        JsonNode resize = node.get("resize");
        if (resize != null && resize.isArray() && resize.size() == 2) {
            manager.resize(attachment.terminalId(), resize.get(0).asInt(), resize.get(1).asInt());
            return;
        }
        JsonNode input = node.get("in");
        if (input != null && input.isTextual()) {
            feed(session, attachment, Base64.getDecoder().decode(input.asText()));
        }
    }

    @Override
    public void handleTransportError(WebSocketSession session, Throwable exception) {
        log.debug("Terminal socket {} transport error: {}", session.getId(), exception.getMessage());
        closeQuietly(session, CloseStatus.SERVER_ERROR);
    }

    @Override
    public void afterConnectionClosed(WebSocketSession session, CloseStatus status) {
        Attachment attachment = attachments.remove(session.getId());
        if (attachment != null) {
            manager.detach(attachment.terminalId(), attachment.listener());
        }
    }

    private void feed(WebSocketSession session, Attachment attachment, byte[] bytes) {
        if (bytes.length == 0) {
            return;
        }
        try {
            manager.write(attachment.terminalId(), bytes);
        } catch (ConflictException gone) {
            closeQuietly(session, CloseStatus.NORMAL.withReason(gone.getMessage()));
        }
    }

    private static UUID terminalIdOf(WebSocketSession session) {
        if (session.getUri() == null) {
            return null;
        }
        Matcher matcher = ID_IN_PATH.matcher(session.getUri().getPath());
        if (!matcher.find()) {
            return null;
        }
        try {
            return UUID.fromString(matcher.group(1));
        } catch (IllegalArgumentException notAUuid) {
            return null;
        }
    }

    private static void closeQuietly(WebSocketSession session, CloseStatus status) {
        try {
            if (session.isOpen()) {
                session.close(status);
            }
        } catch (IOException ignored) {
            // Nothing left to do.
        }
    }
}
