package dev.rekall.mcp;

import org.springframework.boot.context.properties.ConfigurationProperties;

/**
 * Credentials of the read-only database role the MCP server authenticates as.
 *
 * <p>The URL is inherited from the main datasource: it is the same database, reached with a
 * different identity. Only the identity is configurable here, because that is the entire
 * point.
 */
@ConfigurationProperties(prefix = "rekall.mcp.reader")
public class McpReaderProperties {

    /** Role created by Liquibase with SELECT and nothing else. */
    private String username = "rekall_reader";

    private String password;

    /** Small on purpose: the MCP server serves one conversation at a time. */
    private int maximumPoolSize = 4;

    public String getUsername() {
        return username;
    }

    public void setUsername(String username) {
        this.username = username;
    }

    public String getPassword() {
        return password;
    }

    public void setPassword(String password) {
        this.password = password;
    }

    public int getMaximumPoolSize() {
        return maximumPoolSize;
    }

    public void setMaximumPoolSize(int maximumPoolSize) {
        this.maximumPoolSize = maximumPoolSize;
    }
}
