package dev.rekall;

import org.springframework.context.annotation.Configuration;
import org.springframework.web.servlet.config.annotation.ViewControllerRegistry;
import org.springframework.web.servlet.config.annotation.WebMvcConfigurer;

/**
 * Sends UI routes to the Vue entry point.
 *
 * <p>The frontend owns its own routing, so a deep link like {@code /tasks/<id>} must reach
 * {@code index.html} rather than 404. Only the known top-level UI paths are forwarded, so a
 * mistyped API or MCP path still fails as itself instead of quietly returning HTML.
 *
 * <p>This list is the router's mirror and has to be updated with it. Getting it wrong is not
 * visible in normal use, because navigating inside the application never asks the server for
 * these paths: it breaks on the first refresh, and on every bookmark.
 */
@Configuration
public class SinglePageApplicationRouting implements WebMvcConfigurer {

    @Override
    public void addViewControllers(ViewControllerRegistry registry) {
        registry.addViewController("/").setViewName("forward:/index.html");
        registry.addViewController("/projects").setViewName("forward:/index.html");
        registry.addViewController("/projects/**").setViewName("forward:/index.html");
        registry.addViewController("/tasks").setViewName("forward:/index.html");
        registry.addViewController("/tasks/**").setViewName("forward:/index.html");
        registry.addViewController("/search").setViewName("forward:/index.html");
    }
}
