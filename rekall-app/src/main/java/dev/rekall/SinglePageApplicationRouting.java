package dev.rekall;

import org.springframework.context.annotation.Configuration;
import org.springframework.web.servlet.config.annotation.ViewControllerRegistry;
import org.springframework.web.servlet.config.annotation.WebMvcConfigurer;

/**
 * Sends UI routes to the Vue entry point.
 *
 * <p>The frontend owns its own routing, so a deep link like {@code /schema/project} must reach
 * {@code index.html} rather than 404. Only the known top-level UI paths are forwarded, so a
 * mistyped API or MCP path still fails as itself instead of quietly returning HTML.
 */
@Configuration
public class SinglePageApplicationRouting implements WebMvcConfigurer {

    @Override
    public void addViewControllers(ViewControllerRegistry registry) {
        registry.addViewController("/").setViewName("forward:/index.html");
        registry.addViewController("/schema").setViewName("forward:/index.html");
        registry.addViewController("/schema/**").setViewName("forward:/index.html");
        registry.addViewController("/data").setViewName("forward:/index.html");
        registry.addViewController("/data/**").setViewName("forward:/index.html");
        registry.addViewController("/plan").setViewName("forward:/index.html");
        registry.addViewController("/search").setViewName("forward:/index.html");
    }
}
