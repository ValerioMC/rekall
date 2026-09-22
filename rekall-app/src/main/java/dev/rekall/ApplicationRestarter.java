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

    /** Whether a restart can be scheduled at all: false in a context that never registered, like a test. */
    public static boolean canRestart() {
        return context != null;
    }

    public static void restart() {
        restart(() -> { }, () -> { });
    }

    /**
     * Restarts with two hooks around the moment nothing holds the database: {@code beforeClose}
     * runs while the old context is still up (to shut the database down, say), {@code afterClose}
     * once it is gone and before the new one starts (to swap its file). A hook that throws is
     * logged and the restart goes on, so the application always comes back.
     *
     * @return whether a restart was scheduled; false in a context that never registered (a test)
     */
    public static boolean restart(Runnable beforeClose, Runnable afterClose) {
        if (context == null) {
            log.warn("Restart requested before the application registered itself; ignoring (test context?)");
            return false;
        }
        Thread restarter = new Thread(() -> doRestart(beforeClose, afterClose), "rekall-restart");
        restarter.setDaemon(false);
        restarter.start();
        return true;
    }

    private static void doRestart(Runnable beforeClose, Runnable afterClose) {
        try {
            Thread.sleep(400);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            return;
        }
        log.info("Restarting to pick up a database change");
        runHook("before close", beforeClose);
        context.close();
        runHook("after close", afterClose);
        // Set explicitly: the run() overload deduces the main class by stack-walking for a "main"
        // frame this restart thread lacks, which fails hard under GraalVM native image.
        SpringApplication application = new SpringApplication(RekallApplication.class);
        application.setMainApplicationClass(RekallApplication.class);
        context = application.run(args);
    }

    private static void runHook(String when, Runnable hook) {
        try {
            hook.run();
        } catch (RuntimeException e) {
            log.error("The restart hook {} failed; restarting anyway", when, e);
        }
    }
}
