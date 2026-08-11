package dev.rekall.mcp;

import com.zaxxer.hikari.HikariDataSource;
import org.jooq.DSLContext;
import org.jooq.SQLDialect;
import org.jooq.conf.RenderQuotedNames;
import org.jooq.conf.Settings;
import org.jooq.impl.DSL;
import org.jooq.impl.DataSourceConnectionProvider;
import org.jooq.impl.DefaultConfiguration;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.boot.context.properties.EnableConfigurationProperties;
import org.springframework.boot.jdbc.autoconfigure.DataSourceProperties;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

import javax.sql.DataSource;

/**
 * A second connection pool, authenticated as {@code rekall_reader}.
 *
 * <p>This is what makes "Claude is read-only" a property of the system rather than a rule
 * someone has to remember. A write attempted through an MCP tool does not fail a check in
 * Java; PostgreSQL refuses it, because the role holds {@code SELECT} and nothing else.
 */
@Configuration
@EnableConfigurationProperties(McpReaderProperties.class)
public class McpReaderConfiguration {

    /**
     * {@code defaultCandidate = false} keeps this pool out of injection by type. Without it the
     * application has two {@code DataSource} beans and everything that asks for one, Liquibase
     * included, might get the identity that cannot write.
     */
    @Bean(destroyMethod = "close", defaultCandidate = false)
    public HikariDataSource mcpReaderDataSource(
            DataSourceProperties mainDataSource, McpReaderProperties readerProperties) {

        HikariDataSource dataSource = new HikariDataSource();
        // Same database, different identity.
        dataSource.setJdbcUrl(mainDataSource.determineUrl());
        dataSource.setUsername(readerProperties.getUsername());
        dataSource.setPassword(readerProperties.getPassword());
        dataSource.setMaximumPoolSize(readerProperties.getMaximumPoolSize());
        dataSource.setPoolName("rekall-mcp-reader");
        dataSource.setReadOnly(true);
        return dataSource;
    }

    /**
     * Deliberately not transaction-aware. Every MCP read is a single statement, so auto-commit
     * is the honest description of what happens, and wiring this pool into the application's
     * transaction manager would only blur which identity a given statement runs as.
     */
    @Bean(defaultCandidate = false)
    public DSLContext mcpReaderDslContext(@Qualifier("mcpReaderDataSource") DataSource mcpReaderDataSource) {
        DefaultConfiguration configuration = new DefaultConfiguration();
        configuration.set(new DataSourceConnectionProvider(mcpReaderDataSource));
        configuration.set(SQLDialect.POSTGRES);
        configuration.set(new Settings().withRenderQuotedNames(RenderQuotedNames.ALWAYS).withRenderSchema(true));
        return DSL.using(configuration);
    }
}
