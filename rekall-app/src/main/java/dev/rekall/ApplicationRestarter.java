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
        // Set explicitly: the run() overload deduces the main class by stack-walking for a "main"
        // frame this restart thread lacks, which fails hard under GraalVM native image.
        SpringApplication application = new SpringApplication(RekallApplication.class);
        application.setMainApplicationClass(RekallApplication.class);
        context = application.run(args);
    }
}
