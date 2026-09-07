package dev.rekall;

import lombok.extern.slf4j.Slf4j;
import org.springframework.boot.SpringApplication;
import org.springframework.context.ConfigurableApplicationContext;

@Slf4j
public final class ApplicationRestarter {

    private static volatile ConfigurableApplicationContext context;
    private static volatile String[] args;

    private ApplicationRestarter() {
    }

    static void register(ConfigurableApplicationContext initialContext, String[] initialArgs) {
        context = initialContext;
        args = initialArgs;
    }

    public static void restart() {
        if (context == null) {
            log.warn("Restart requested before the application registered itself; ignoring (test context?)");
            return;
        }
        Thread restarter = new Thread(ApplicationRestarter::doRestart, "rekall-restart");
        restarter.setDaemon(false);
        restarter.start();
    }

    private static void doRestart() {
        try {
            Thread.sleep(400);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            return;
        }
        log.info("Restarting to pick up a database location change");
        context.close();
        // Not SpringApplication.run(RekallApplication.class, args): that convenience overload
        // deduces the "main application class" by stack-walking for a frame literally named
        // "main", which this thread ("rekall-restart", started fresh rather than descending
        // from the process's own main thread) never has. On the JVM that deduction failing
        // just leaves it null and is silently ignored; under GraalVM native image, AOT mode is
        // always on and needs the main application class to look up the generated
        // __ApplicationContextInitializer by name, so a null there is a hard
        // IllegalStateException instead. Setting it explicitly sidesteps the deduction
        // entirely.
        SpringApplication application = new SpringApplication(RekallApplication.class);
        application.setMainApplicationClass(RekallApplication.class);
        context = application.run(args);
    }
}
