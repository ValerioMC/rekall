package dev.rekall.engine;

import org.jooq.conf.RenderQuotedNames;
import org.springframework.boot.jooq.autoconfigure.DefaultConfigurationCustomizer;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class RekallJooqConfiguration {

    /**
     * Quoting every identifier is the second layer of defence for generated DDL, after the
     * validation applied when a name is persisted. It costs nothing, and it means a name that
     * somehow slipped past validation still cannot change the shape of a statement.
     *
     * <p>Schema qualification is on so that generated SQL names {@code rekall_data} explicitly
     * rather than depending on whatever {@code search_path} the connection happens to carry.
     */
    @Bean
    DefaultConfigurationCustomizer rekallJooqCustomizer() {
        return configuration -> configuration
                .settings()
                .withRenderQuotedNames(RenderQuotedNames.ALWAYS)
                .withRenderSchema(true);
    }
}
