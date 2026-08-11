package dev.rekall;

import org.springframework.boot.autoconfigure.SpringBootApplication;

/**
 * Boot entry point for the engine tests.
 *
 * <p>Declared in {@code dev.rekall} so that component scan and entity scan cover both
 * {@code dev.rekall.meta} and {@code dev.rekall.engine} without any explicit configuration.
 */
@SpringBootApplication
public class EngineTestApplication {
}
