package dev.rekall;

import dev.rekall.bootstrap.DatabaseEntry;
import dev.rekall.bootstrap.DatabaseRegistry;
import dev.rekall.domain.Company;
import dev.rekall.domain.Document;
import dev.rekall.domain.Project;
import dev.rekall.domain.Task;
import dev.rekall.domain.TimeEntry;
import dev.rekall.domain.Wrapup;
import org.springframework.aot.hint.annotation.RegisterReflectionForBinding;
import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.context.ConfigurableApplicationContext;

/**
 * Rekall: structured memory and context for the projects and tasks you work on.
 *
 * <p>Declared in {@code dev.rekall} so component scan and entity scan reach every module
 * without any explicit package configuration. One process serves the REST API, the MCP
 * endpoint and the built UI.
 *
 * <p>{@code DatabaseRegistry}/{@code DatabaseEntry} are (de)serialized by a plain
 * {@code ObjectMapper} in {@code DatabaseRegistryStore}, outside any Spring bean — Spring's AOT
 * engine never sees that call site, so under GraalVM native image it needs the binding
 * registered here explicitly.
 *
 * <p>The domain entities are listed here too: under the {@code native} Maven profile,
 * hibernate-maven-plugin weaves Hibernate-internal {@code $$_hibernate_*} methods (entity
 * identity, dirty tracking, lazy loading) directly into their bytecode - see
 * rekall-domain/pom.xml. Spring's own JPA/Jackson AOT hints only cover what binding needs
 * (getters, setters, constructors), not these Hibernate-internal methods, so GraalVM's
 * closed-world analysis doesn't see them as reachable and strips them, throwing
 * AbstractMethodError the first time Hibernate calls one at runtime.
 */
@SpringBootApplication
@RegisterReflectionForBinding({
        DatabaseRegistry.class, DatabaseEntry.class,
        Company.class, Project.class, Task.class, TimeEntry.class, Wrapup.class, Document.class
})
public class RekallApplication {

    public static void main(String[] args) {
        ConfigurableApplicationContext context = SpringApplication.run(RekallApplication.class, args);
        ApplicationRestarter.register(context, args);
    }
}
