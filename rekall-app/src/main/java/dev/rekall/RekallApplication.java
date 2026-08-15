package dev.rekall;

import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.context.ConfigurableApplicationContext;

/**
 * Rekall: structured memory and context for the projects and tasks you work on.
 *
 * <p>Declared in {@code dev.rekall} so component scan and entity scan reach every module
 * without any explicit package configuration. One process serves the REST API, the MCP
 * endpoint and the built UI.
 */
@SpringBootApplication
public class RekallApplication {

    public static void main(String[] args) {
        ConfigurableApplicationContext context = SpringApplication.run(RekallApplication.class, args);
        ApplicationRestarter.register(context, args);
    }
}
