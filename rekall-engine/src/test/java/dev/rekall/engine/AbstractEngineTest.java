package dev.rekall.engine;

import dev.rekall.engine.execute.DdlApplyResult;
import dev.rekall.engine.execute.DdlExecutor;
import dev.rekall.engine.plan.DdlPlan;
import dev.rekall.engine.plan.DdlPlanner;
import dev.rekall.engine.plan.PlanOptions;
import dev.rekall.engine.schema.SchemaRegistry;
import dev.rekall.meta.AbstractPostgresTest;
import dev.rekall.meta.domain.MetaTable;
import dev.rekall.meta.repository.DdlLogRepository;
import dev.rekall.meta.repository.DocumentRepository;
import dev.rekall.meta.repository.MetaRelationRepository;
import dev.rekall.meta.repository.MetaTableRepository;
import org.junit.jupiter.api.BeforeEach;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.jdbc.core.JdbcTemplate;

@SpringBootTest
public abstract class AbstractEngineTest extends AbstractPostgresTest {

    @Autowired
    protected DdlPlanner planner;

    @Autowired
    protected DdlExecutor executor;

    @Autowired
    protected SchemaRegistry schemaRegistry;

    @Autowired
    protected MetaTableRepository metaTables;

    @Autowired
    protected MetaRelationRepository metaRelations;

    @Autowired
    protected DocumentRepository documents;

    @Autowired
    protected DdlLogRepository ddlLogs;

    @Autowired
    protected JdbcTemplate jdbc;

    /**
     * Empties both the generated schema and the meta-model between tests.
     *
     * <p>Drops the tables rather than the schema so the default privileges granted to
     * {@code rekall_reader} at migration time stay attached to {@code rekall_data}.
     */
    @BeforeEach
    void resetSchema() {
        jdbc.execute(
                """
                DO $$
                DECLARE r record;
                BEGIN
                    FOR r IN SELECT tablename FROM pg_tables WHERE schemaname = 'rekall_data' LOOP
                        EXECUTE format('DROP TABLE IF EXISTS rekall_data.%I CASCADE', r.tablename);
                    END LOOP;
                END
                $$;
                """);
        documents.deleteAll();
        ddlLogs.deleteAll();
        metaRelations.deleteAll();
        metaTables.deleteAll();
        schemaRegistry.invalidate();
    }

    protected MetaTable save(MetaTable table) {
        return metaTables.saveAndFlush(table);
    }

    /** Plans with no answers supplied, the state the UI shows before the user is asked anything. */
    protected DdlPlan plan() {
        return planner.plan(PlanOptions.empty());
    }

    protected DdlPlan plan(PlanOptions options) {
        return planner.plan(options);
    }

    /** Plans and applies in one step, for the many tests that only care about the end state. */
    protected DdlApplyResult applyAll() {
        return executor.apply(plan());
    }

    protected DdlApplyResult applyAll(PlanOptions options) {
        return executor.apply(plan(options));
    }

    protected boolean tableExists(String name) {
        Integer count = jdbc.queryForObject(
                "SELECT count(*) FROM information_schema.tables WHERE table_schema = 'rekall_data' AND table_name = ?",
                Integer.class,
                name);
        return count != null && count > 0;
    }

    protected String columnType(String table, String column) {
        return jdbc.queryForObject(
                """
                SELECT CASE
                         WHEN data_type = 'ARRAY' THEN udt_name
                         WHEN character_maximum_length IS NOT NULL
                              THEN data_type || '(' || character_maximum_length || ')'
                         ELSE data_type
                       END
                FROM information_schema.columns
                WHERE table_schema = 'rekall_data' AND table_name = ? AND column_name = ?
                """,
                String.class,
                table,
                column);
    }

    protected boolean columnIsNullable(String table, String column) {
        return "YES"
                .equals(jdbc.queryForObject(
                        """
                        SELECT is_nullable FROM information_schema.columns
                        WHERE table_schema = 'rekall_data' AND table_name = ? AND column_name = ?
                        """,
                        String.class,
                        table,
                        column));
    }

    protected boolean constraintExists(String name) {
        Integer count = jdbc.queryForObject(
                """
                SELECT count(*) FROM pg_constraint con
                JOIN pg_namespace ns ON ns.oid = con.connamespace
                WHERE ns.nspname = 'rekall_data' AND con.conname = ?
                """,
                Integer.class,
                name);
        return count != null && count > 0;
    }
}
