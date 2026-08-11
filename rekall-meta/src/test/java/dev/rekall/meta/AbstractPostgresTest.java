package dev.rekall.meta;

import org.springframework.test.context.DynamicPropertyRegistry;
import org.springframework.test.context.DynamicPropertySource;
import org.testcontainers.containers.PostgreSQLContainer;

/**
 * Singleton PostgreSQL container shared by every test in the JVM.
 *
 * <p>Started once in a static initialiser rather than through {@code @Testcontainers}, which
 * would start and stop one container per test class. Ryuk reaps it when the JVM exits.
 */
public abstract class AbstractPostgresTest {

    protected static final PostgreSQLContainer<?> POSTGRES =
            new PostgreSQLContainer<>("postgres:16-alpine")
                    .withDatabaseName("rekall")
                    .withUsername("rekall")
                    .withPassword("rekall");

    static {
        POSTGRES.start();
    }

    @DynamicPropertySource
    static void datasourceProperties(DynamicPropertyRegistry registry) {
        registry.add("spring.datasource.url", POSTGRES::getJdbcUrl);
        registry.add("spring.datasource.username", POSTGRES::getUsername);
        registry.add("spring.datasource.password", POSTGRES::getPassword);
        registry.add("spring.liquibase.parameters.readerPassword", () -> "reader-test-password");
    }
}
