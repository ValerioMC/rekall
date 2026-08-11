package dev.rekall.mcp;

import lombok.Getter;
import lombok.Setter;
import org.springframework.boot.context.properties.ConfigurationProperties;

/**
 * Credentials of the read-only database role the MCP server authenticates as.
 *
 * <p>The URL is inherited from the main datasource: it is the same database, reached with a
 * different identity. Only the identity is configurable here, because that is the entire
 * point.
 */
@ConfigurationProperties(prefix = "rekall.mcp.reader")
@Getter
@Setter
public class McpReaderProperties {

    /** Role created by Liquibase with SELECT and nothing else. */
    private String username = "rekall_reader";

    private String password;

    /** Small on purpose: the MCP server serves one conversation at a time. */
    private int maximumPoolSize = 4;
}
