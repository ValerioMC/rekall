package dev.rekall.meta;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.jdbc.core.JdbcTemplate;

import java.util.List;

import static org.assertj.core.api.Assertions.assertThat;

/**
 * Proves the Liquibase changelogs and the JPA entities agree.
 *
 * <p>{@code ddl-auto: validate} means the context simply fails to start if any entity does not
 * match its table, so most of the value here is in the fact that these tests run at all.
 */
@SpringBootTest(classes = rekalltest.RekallMetaTestApplication.class)
class MetaSchemaMigrationTest extends AbstractPostgresTest {

    @Autowired
    private JdbcTemplate jdbc;

    @Test
    @DisplayName("both schemas are created, and rekall_data is left empty for the engine")
    void schemasAreCreated() {
        List<String> schemas = jdbc.queryForList(
                "SELECT schema_name FROM information_schema.schemata WHERE schema_name LIKE 'rekall%'",
                String.class);

        assertThat(schemas).containsExactlyInAnyOrder("rekall_meta", "rekall_data");

        Integer dataTables = jdbc.queryForObject(
                "SELECT count(*) FROM information_schema.tables WHERE table_schema = 'rekall_data'", Integer.class);
        assertThat(dataTables).isZero();
    }

    @Test
    @DisplayName("every meta-model table exists in rekall_meta")
    void metaTablesAreCreated() {
        List<String> tables = jdbc.queryForList(
                "SELECT table_name FROM information_schema.tables WHERE table_schema = 'rekall_meta'", String.class);

        assertThat(tables).contains("meta_table", "meta_field", "meta_relation", "ddl_log", "document");
    }

    @Test
    @DisplayName("the read-only role exists and can log in")
    void readerRoleExists() {
        Boolean canLogin = jdbc.queryForObject(
                "SELECT rolcanlogin FROM pg_roles WHERE rolname = 'rekall_reader'", Boolean.class);

        assertThat(canLogin).isTrue();
    }

    @Test
    @DisplayName("the reader role has no write privilege anywhere in rekall_data")
    void readerCannotWrite() {
        Boolean canCreate = jdbc.queryForObject(
                "SELECT has_schema_privilege('rekall_reader', 'rekall_data', 'CREATE')", Boolean.class);
        Boolean canUse = jdbc.queryForObject(
                "SELECT has_schema_privilege('rekall_reader', 'rekall_data', 'USAGE')", Boolean.class);

        assertThat(canCreate).as("reader must not be able to create objects").isFalse();
        assertThat(canUse).as("reader must be able to see the schema").isTrue();
    }

    @Test
    @DisplayName("default privileges are in place so tables generated later are readable")
    void defaultPrivilegesCoverFutureTables() {
        Integer defaults = jdbc.queryForObject(
                """
                SELECT count(*) FROM pg_default_acl acl
                JOIN pg_namespace ns ON ns.oid = acl.defaclnamespace
                WHERE ns.nspname = 'rekall_data' AND acl.defaclobjtype = 'r'
                """,
                Integer.class);

        assertThat(defaults)
                .as("without default privileges every generated table would be invisible to the MCP server")
                .isEqualTo(1);
    }

    @Test
    @DisplayName("the document search vector is a stored generated column")
    void documentSearchVectorIsGenerated() {
        String generated = jdbc.queryForObject(
                """
                SELECT is_generated FROM information_schema.columns
                WHERE table_schema = 'rekall_meta' AND table_name = 'document' AND column_name = 'search_vector'
                """,
                String.class);

        assertThat(generated).isEqualTo("ALWAYS");
    }
}
