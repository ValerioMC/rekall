package rekalltest;

import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.boot.persistence.autoconfigure.EntityScan;
import org.springframework.data.jpa.repository.config.EnableJpaRepositories;

/**
 * Boot entry point for this module's own tests.
 *
 * <p>Deliberately outside {@code dev.rekall}. This class ships in the test-jar that every
 * downstream module depends on, and a reactor build resolves that dependency to
 * {@code target/test-classes} rather than to the packaged jar, so excluding it at packaging
 * time does not help. Sitting outside the scanned package means a downstream context cannot
 * pick it up and register the JPA repositories a second time, whatever the build does.
 */
@SpringBootApplication(scanBasePackages = "dev.rekall.meta")
@EntityScan("dev.rekall.meta.domain")
@EnableJpaRepositories("dev.rekall.meta.repository")
public class RekallMetaTestApplication {
}
