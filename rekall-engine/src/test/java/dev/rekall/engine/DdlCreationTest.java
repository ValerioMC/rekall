package dev.rekall.engine;

import dev.rekall.engine.plan.ChangeClass;
import dev.rekall.engine.plan.DdlPhase;
import dev.rekall.engine.plan.DdlPlan;
import dev.rekall.engine.plan.DdlRenderer;
import dev.rekall.engine.plan.DdlStatement;
import dev.rekall.meta.MetaModelFixtures;
import dev.rekall.meta.domain.DdlStatus;
import dev.rekall.meta.domain.MetaField;
import dev.rekall.meta.domain.MetaFieldType;
import dev.rekall.meta.domain.MetaRelation;
import dev.rekall.meta.domain.MetaTable;
import dev.rekall.meta.domain.MetaTableStatus;
import dev.rekall.meta.domain.OnDeleteAction;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.springframework.dao.DataIntegrityViolationException;

import java.util.List;
import java.util.UUID;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

/** Creating entities, their infrastructure and their relations from an empty schema. */
class DdlCreationTest extends AbstractEngineTest {

    @Test
    @DisplayName("computing a plan changes nothing: only apply touches the database")
    void planningHasNoSideEffect() {
        MetaTable project = MetaModelFixtures.project();
        project.addField(MetaModelFixtures.text("name", "Name"));
        save(project);

        DdlPlan plan = plan();

        assertThat(plan.statements()).isNotEmpty();
        assertThat(tableExists("project")).as("the table must not exist until apply is called").isFalse();
    }

    @Test
    @DisplayName("a new entity gets its system columns, primary key and updated_at trigger")
    void createsTableWithSystemColumns() {
        MetaTable project = MetaModelFixtures.project();
        project.addField(MetaModelFixtures.text("name", "Name"));
        save(project);

        applyAll();

        assertThat(tableExists("project")).isTrue();
        assertThat(columnType("project", "id")).isEqualTo("uuid");
        assertThat(columnType("project", "created_at")).isEqualTo("timestamp with time zone");
        assertThat(columnType("project", "updated_at")).isEqualTo("timestamp with time zone");
        assertThat(constraintExists("pk_project")).isTrue();

        Integer triggers = jdbc.queryForObject(
                "SELECT count(*) FROM pg_trigger WHERE tgname = ?",
                Integer.class,
                DdlRenderer.triggerName("project"));
        assertThat(triggers).isEqualTo(1);
    }

    @Test
    @DisplayName("the updated_at trigger actually fires on update")
    void updatedAtIsMaintainedByTheDatabase() {
        MetaTable project = MetaModelFixtures.project();
        project.addField(MetaModelFixtures.text("name", "Name"));
        save(project);
        applyAll();

        jdbc.update("INSERT INTO rekall_data.project (name) VALUES ('STVV')");
        jdbc.execute("SELECT pg_sleep(0.01)");
        jdbc.update("UPDATE rekall_data.project SET name = 'STVV-A'");

        Boolean touched = jdbc.queryForObject(
                "SELECT updated_at > created_at FROM rekall_data.project", Boolean.class);
        assertThat(touched).isTrue();
    }

    @Test
    @DisplayName("every field type maps to the intended physical type")
    void mapsEveryFieldType() {
        MetaTable table = MetaModelFixtures.project();
        table.addField(MetaModelFixtures.text("short_text", "Short text"));
        table.addField(MetaModelFixtures.longText("long_text", "Long text"));
        table.addField(new MetaField("body", "Body", "Markdown di test", MetaFieldType.MARKDOWN));
        table.addField(MetaModelFixtures.integer("counter", "Counter"));
        table.addField(MetaModelFixtures.decimal("amount", "Amount", 10, 2));
        table.addField(new MetaField("active", "Active", "Booleano di test", MetaFieldType.BOOLEAN));
        table.addField(new MetaField("due_on", "Due on", "Data di test", MetaFieldType.DATE));
        table.addField(MetaModelFixtures.timestamp("seen_at", "Seen at"));
        table.addField(MetaModelFixtures.enumeration("status", "Status", "active", "paused", "done"));
        table.addField(MetaModelFixtures.tags("labels", "Labels"));
        table.addField(MetaModelFixtures.reference("owner_id", "Owner"));
        save(table);

        applyAll();

        assertThat(columnType("project", "short_text")).isEqualTo("character varying(255)");
        assertThat(columnType("project", "long_text")).isEqualTo("text");
        assertThat(columnType("project", "body")).isEqualTo("text");
        assertThat(columnType("project", "counter")).as("INTEGER always maps to bigint").isEqualTo("bigint");
        assertThat(columnType("project", "amount")).isEqualTo("numeric");
        assertThat(columnType("project", "active")).isEqualTo("boolean");
        assertThat(columnType("project", "due_on")).isEqualTo("date");
        assertThat(columnType("project", "seen_at")).isEqualTo("timestamp with time zone");
        assertThat(columnType("project", "status")).isEqualTo("character varying(50)");
        assertThat(columnType("project", "labels")).as("TAGS maps to text[]").isEqualTo("_text");
        assertThat(columnType("project", "owner_id")).isEqualTo("uuid");
    }

    @Test
    @DisplayName("an enum field is constrained by a generated check, not only by the application")
    void enumIsEnforcedByTheDatabase() {
        MetaTable project = MetaModelFixtures.project();
        project.addField(MetaModelFixtures.enumeration("status", "Status", "active", "paused", "done"));
        save(project);
        applyAll();

        assertThat(constraintExists(DdlRenderer.enumCheckName("project", "status"))).isTrue();
        jdbc.update("INSERT INTO rekall_data.project (status) VALUES ('active')");

        assertThatThrownBy(() -> jdbc.update("INSERT INTO rekall_data.project (status) VALUES ('cancelled')"))
                .isInstanceOf(DataIntegrityViolationException.class);
    }

    @Test
    @DisplayName("a null passes the enum check, so ENUM does not silently imply required")
    void enumAllowsNullWhenOptional() {
        MetaTable project = MetaModelFixtures.project();
        project.addField(MetaModelFixtures.enumeration("status", "Status", "active", "done"));
        save(project);
        applyAll();

        jdbc.update("INSERT INTO rekall_data.project (status) VALUES (NULL)");

        assertThat(jdbc.queryForObject("SELECT count(*) FROM rekall_data.project", Integer.class)).isEqualTo(1);
    }

    @Test
    @DisplayName("a tags field gets a GIN index so containment queries stay indexed")
    void tagsColumnIsIndexed() {
        MetaTable project = MetaModelFixtures.project();
        project.addField(MetaModelFixtures.tags("labels", "Labels"));
        save(project);

        applyAll();

        String indexDefinition = jdbc.queryForObject(
                "SELECT indexdef FROM pg_indexes WHERE schemaname = 'rekall_data' AND indexname = ?",
                String.class,
                DdlRenderer.tagsIndexName("project", "labels"));
        assertThat(indexDefinition).containsIgnoringCase("USING gin");
    }

    @Test
    @DisplayName("a default value is written into the column definition")
    void defaultValueIsApplied() {
        MetaTable project = MetaModelFixtures.project();
        project.addField(MetaModelFixtures.withDefault(MetaModelFixtures.text("status", "Status"), "active"));
        save(project);
        applyAll();

        jdbc.update("INSERT INTO rekall_data.project DEFAULT VALUES");

        assertThat(jdbc.queryForObject("SELECT status FROM rekall_data.project", String.class)).isEqualTo("active");
    }

    @Test
    @DisplayName("a many-to-one relation becomes a real foreign key that refuses orphans")
    void manyToOneIsEnforced() {
        MetaTable environment = save(MetaModelFixtures.environment());
        MetaTable task = MetaModelFixtures.task();
        MetaField environmentId = MetaModelFixtures.reference("environment_id", "Environment");
        task.addField(environmentId);
        save(task);

        MetaRelation relation = MetaRelation.manyToOne(
                task, environment, environmentId.getId(), OnDeleteAction.RESTRICT, "L'ambiente su cui gira il task");
        metaRelations.saveAndFlush(relation);

        applyAll();

        assertThat(constraintExists(relation.constraintName())).isTrue();
        assertThatThrownBy(() -> jdbc.update(
                        "INSERT INTO rekall_data.task (environment_id) VALUES (?)", UUID.randomUUID()))
                .as("a task pointing at a non-existent environment must be refused by the database")
                .isInstanceOf(DataIntegrityViolationException.class);
    }

    @Test
    @DisplayName("on delete restrict stops an environment from being removed while a task uses it")
    void onDeleteRestrictProtectsReferencedRows() {
        MetaTable environment = save(MetaModelFixtures.environment());
        MetaTable task = MetaModelFixtures.task();
        MetaField environmentId = MetaModelFixtures.reference("environment_id", "Environment");
        task.addField(environmentId);
        save(task);
        metaRelations.saveAndFlush(MetaRelation.manyToOne(
                task, environment, environmentId.getId(), OnDeleteAction.RESTRICT, "L'ambiente su cui gira"));
        applyAll();

        jdbc.update("INSERT INTO rekall_data.environment DEFAULT VALUES");
        UUID environmentRow = jdbc.queryForObject("SELECT id FROM rekall_data.environment", UUID.class);
        jdbc.update("INSERT INTO rekall_data.task (environment_id) VALUES (?)", environmentRow);

        assertThatThrownBy(() -> jdbc.update("DELETE FROM rekall_data.environment"))
                .isInstanceOf(DataIntegrityViolationException.class);
    }

    @Test
    @DisplayName("foreign keys are added after every table exists, whatever order they were declared in")
    void foreignKeysComeAfterTables() {
        MetaTable environment = save(MetaModelFixtures.environment());
        MetaTable task = MetaModelFixtures.task();
        MetaField environmentId = MetaModelFixtures.reference("environment_id", "Environment");
        task.addField(environmentId);
        save(task);
        metaRelations.saveAndFlush(MetaRelation.manyToOne(
                task, environment, environmentId.getId(), OnDeleteAction.CASCADE, "L'ambiente su cui gira"));

        List<DdlStatement> statements = plan().executable();

        int lastCreate = lastIndexOfPhase(statements, DdlPhase.CREATE_TABLE);
        int firstConstraint = firstIndexOfPhase(statements, DdlPhase.ADD_CONSTRAINT);
        assertThat(firstConstraint).isGreaterThan(lastCreate);
    }

    @Test
    @DisplayName("a many-to-many relation becomes a join table cascading on both sides")
    void manyToManyCreatesJoinTable() {
        MetaTable project = save(MetaModelFixtures.project());
        MetaTable task = save(MetaModelFixtures.task());
        metaRelations.saveAndFlush(
                MetaRelation.manyToMany(project, task, "rel_project_task", "Task collegati a piu progetti"));

        applyAll();

        assertThat(tableExists("rel_project_task")).isTrue();
        assertThat(columnType("rel_project_task", "project_id")).isEqualTo("uuid");
        assertThat(columnType("rel_project_task", "task_id")).isEqualTo("uuid");

        jdbc.update("INSERT INTO rekall_data.project DEFAULT VALUES");
        jdbc.update("INSERT INTO rekall_data.task DEFAULT VALUES");
        UUID projectRow = jdbc.queryForObject("SELECT id FROM rekall_data.project", UUID.class);
        UUID taskRow = jdbc.queryForObject("SELECT id FROM rekall_data.task", UUID.class);
        jdbc.update("INSERT INTO rekall_data.rel_project_task VALUES (?, ?)", projectRow, taskRow);

        jdbc.update("DELETE FROM rekall_data.project");

        assertThat(jdbc.queryForObject("SELECT count(*) FROM rekall_data.rel_project_task", Integer.class))
                .as("a join row is meaningless once either side is gone")
                .isZero();
    }

    @Test
    @DisplayName("a self-referencing many-to-many avoids duplicate column names")
    void selfReferencingManyToManyUsesDistinctColumns() {
        MetaTable task = save(MetaModelFixtures.task());
        metaRelations.saveAndFlush(MetaRelation.manyToMany(task, task, "rel_task_task", "Task dipendenti fra loro"));

        applyAll();

        assertThat(columnType("rel_task_task", "source_id")).isEqualTo("uuid");
        assertThat(columnType("rel_task_task", "target_id")).isEqualTo("uuid");
    }

    @Test
    @DisplayName("applying records every statement in the ddl log and marks the entity applied")
    void applyRecordsTheLog() {
        MetaTable project = MetaModelFixtures.project();
        project.addField(MetaModelFixtures.text("name", "Name"));
        save(project);

        DdlPlan plan = plan();
        executor.apply(plan);

        var log = ddlLogs.findByPlanIdOrderBySequenceAsc(plan.planId());
        assertThat(log).isNotEmpty();
        assertThat(log).allSatisfy(entry -> assertThat(entry.getStatus()).isEqualTo(DdlStatus.APPLIED));
        assertThat(log.getFirst().getStatement()).containsIgnoringCase("create table");
        assertThat(metaTables.findByPhysicalName("project").orElseThrow().getStatus())
                .isEqualTo(MetaTableStatus.APPLIED);
    }

    @Test
    @DisplayName("a second apply with no model change produces an empty plan")
    void applyingTwiceIsIdempotent() {
        MetaTable project = MetaModelFixtures.project();
        project.addField(MetaModelFixtures.text("name", "Name"));
        save(project);
        applyAll();

        DdlPlan second = plan();

        assertThat(second.isEmpty()).isTrue();
        assertThat(second.isApplicable()).isFalse();
    }

    @Test
    @DisplayName("generated identifiers are quoted, so a name can never reshape a statement")
    void identifiersAreQuoted() {
        MetaTable project = MetaModelFixtures.project();
        project.addField(MetaModelFixtures.text("name", "Name"));
        save(project);

        String createTable = plan().executable().stream()
                .filter(statement -> statement.phase() == DdlPhase.CREATE_TABLE)
                .map(DdlStatement::sql)
                .findFirst()
                .orElseThrow();

        assertThat(createTable).contains("\"rekall_data\".\"project\"").contains("\"name\"");
    }

    @Test
    @DisplayName("a fresh model produces only safe changes")
    void creationIsAlwaysSafe() {
        MetaTable project = MetaModelFixtures.project();
        project.addField(MetaModelFixtures.required(MetaModelFixtures.text("name", "Name")));
        save(project);

        DdlPlan plan = plan();

        assertThat(plan.statements()).allSatisfy(statement ->
                assertThat(statement.changeClass()).isEqualTo(ChangeClass.SAFE));
        assertThat(plan.isApplicable()).isTrue();
    }

    private static int firstIndexOfPhase(List<DdlStatement> statements, DdlPhase phase) {
        for (int i = 0; i < statements.size(); i++) {
            if (statements.get(i).phase() == phase) {
                return i;
            }
        }
        throw new AssertionError("No statement in phase " + phase);
    }

    private static int lastIndexOfPhase(List<DdlStatement> statements, DdlPhase phase) {
        int found = -1;
        for (int i = 0; i < statements.size(); i++) {
            if (statements.get(i).phase() == phase) {
                found = i;
            }
        }
        if (found < 0) {
            throw new AssertionError("No statement in phase " + phase);
        }
        return found;
    }
}
