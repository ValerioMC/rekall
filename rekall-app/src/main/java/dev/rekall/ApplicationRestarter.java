package dev.rekall;

import lombok.extern.slf4j.Slf4j;
import org.springframework.boot.SpringApplication;
import org.springframework.context.ConfigurableApplicationContext;

/**
 * Reboots the application in place, on the same port, without the process itself being
 * relaunched.
 *
 * <p>Spring cannot swap a running {@code DataSource} for a different JDBC URL, so every change
 * to which database folder is active needs a fresh {@code ApplicationContext}. Doing that as an
 * in-process restart rather than asking the user to relaunch the jar is what makes switching
 * databases from Settings a normal, one-click action instead of a shutdown.
 */
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

    /**
     * Runs on a separate thread so the HTTP response that triggered this finishes writing before
     * the server it is written through goes down. A no-op if the application never registered
     * itself, which is the case in tests that boot a context directly rather than through
     * {@code main}.
     */
    public static void restart() {
        if (context == null) {
            log.warn("Restart requested before the application registered itself; ignoring (test context?)");
            return;
        }
        Thread restarter = new Thread(ApplicationRestarter::doRestart, "rekall-restart");
        restarter.setDaemon(false);
        restarter.start();
    }

    /**
     * The old context has to be closed, releasing port 8080, before the new one is started: the
     * new context binds the same port while starting up, and Tomcat cannot bind a port the old
     * process is still holding. Doing this the other way around fails every time with a
     * {@code BindException}, and leaves the old context — the one meant to be replaced — as the
     * only one still running.
     */
    private static void doRestart() {
        try {
            Thread.sleep(400);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            return;
        }
        log.info("Restarting to pick up a database location change");
        context.close();
        context = SpringApplication.run(RekallApplication.class, args);
    }
}
