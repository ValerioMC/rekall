package dev.rekall.engine;

import dev.rekall.engine.execute.PlanNotApplicableException;
import dev.rekall.engine.plan.ChangeClass;
import dev.rekall.engine.plan.DdlPlan;
import dev.rekall.engine.plan.DdlStatement;
import dev.rekall.engine.plan.PlanOptions;
import dev.rekall.meta.MetaModelFixtures;
import dev.rekall.meta.domain.DdlStatus;
import dev.rekall.meta.domain.Document;
import dev.rekall.meta.domain.MetaField;
import dev.rekall.meta.domain.MetaFieldType;
import dev.rekall.meta.domain.MetaRelation;
import dev.rekall.meta.domain.MetaTable;
import dev.rekall.meta.domain.OnDeleteAction;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;

import java.util.Map;
import java.util.Set;
import java.util.UUID;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

/**
 * One test per row of the alter rules table in the design document.
 *
 * <p>This is the suite that matters: everything else in the engine can be re-run, but a change
 * misclassified as safe destroys data that is not coming back.
 */
class DdlAlterRulesTest extends AbstractEngineTest {

    private MetaTable applyProjectWith(MetaField... fields) {
        MetaTable project = MetaModelFixtures.project();
        for (MetaField field : fields) {
            project.addField(field);
        }
        save(project);
        applyAll();
        return metaTables.findByPhysicalName("project").orElseThrow();
    }

    @Nested
    @DisplayName("adding a column")
    class AddingColumns {

        @Test
        @DisplayName("a new optional column is safe")
        void newNullableColumnIsSafe() {
            MetaTable project = applyProjectWith(MetaModelFixtures.text("name", "Name"));
            project.addField(MetaModelFixtures.longText("notes", "Notes"));
            save(project);

            DdlPlan plan = plan();

            assertThat(plan.statements()).allSatisfy(s -> assertThat(s.changeClass()).isEqualTo(ChangeClass.SAFE));
            executor.apply(plan);
            assertThat(columnType("project", "notes")).isEqualTo("text");
        }

        @Test
        @DisplayName("a new required column on an empty table is safe")
        void newRequiredColumnOnEmptyTableIsSafe() {
            MetaTable project = applyProjectWith(MetaModelFixtures.text("name", "Name"));
            project.addField(MetaModelFixtures.required(MetaModelFixtures.text("code", "Code")));
            save(project);

            DdlPlan plan = plan();

            assertThat(plan.isApplicable()).isTrue();
            executor.apply(plan);
            assertThat(columnIsNullable("project", "code")).isFalse();
        }

        @Test
        @DisplayName("a new required column on a populated table waits for a default")
        void newRequiredColumnOnPopulatedTableNeedsInput() {
            MetaTable project = applyProjectWith(MetaModelFixtures.text("name", "Name"));
            jdbc.update("INSERT INTO rekall_data.project (name) VALUES ('STVV')");

            project.addField(MetaModelFixtures.required(MetaModelFixtures.text("code", "Code")));
            save(project);

            DdlPlan plan = plan();

            assertThat(plan.awaitingInput()).hasSize(1);
            assertThat(plan.awaitingInput().getFirst().inputKey()).isEqualTo("project.code");
            assertThat(plan.awaitingInput().getFirst().warning()).contains("1 row");
            assertThat(plan.isApplicable()).isFalse();
            assertThatThrownBy(() -> executor.apply(plan)).isInstanceOf(PlanNotApplicableException.class);
        }

        @Test
        @DisplayName("supplying the default turns it into add, backfill, tighten")
        void backfillIsThreeSteps() {
            MetaTable project = applyProjectWith(MetaModelFixtures.text("name", "Name"));
            jdbc.update("INSERT INTO rekall_data.project (name) VALUES ('STVV')");
            project.addField(MetaModelFixtures.required(MetaModelFixtures.text("code", "Code")));
            save(project);

            PlanOptions options = new PlanOptions(Map.of("project.code", "UNKNOWN"), Set.of());
            DdlPlan plan = plan(options);

            assertThat(plan.isApplicable()).isTrue();
            assertThat(plan.executable()).extracting(DdlStatement::sql)
                    .anySatisfy(sql -> assertThat(sql).containsIgnoringCase("add"))
                    .anySatisfy(sql -> assertThat(sql).containsIgnoringCase("update"))
                    .anySatisfy(sql -> assertThat(sql).containsIgnoringCase("set not null"));

            executor.apply(plan);

            assertThat(columnIsNullable("project", "code")).isFalse();
            assertThat(jdbc.queryForObject("SELECT code FROM rekall_data.project", String.class))
                    .isEqualTo("UNKNOWN");
        }

        @Test
        @DisplayName("a required column with its own default needs no answer")
        void requiredColumnWithDefaultIsSafe() {
            MetaTable project = applyProjectWith(MetaModelFixtures.text("name", "Name"));
            jdbc.update("INSERT INTO rekall_data.project (name) VALUES ('STVV')");

            project.addField(MetaModelFixtures.required(
                    MetaModelFixtures.withDefault(MetaModelFixtures.text("code", "Code"), "TBD")));
            save(project);

            DdlPlan plan = plan();

            assertThat(plan.isApplicable()).isTrue();
            executor.apply(plan);
            assertThat(jdbc.queryForObject("SELECT code FROM rekall_data.project", String.class)).isEqualTo("TBD");
        }
    }

    @Nested
    @DisplayName("changing a type")
    class ChangingTypes {

        @Test
        @DisplayName("widening a varchar is safe")
        void wideningVarcharIsSafe() {
            MetaTable project = applyProjectWith(MetaModelFixtures.text("name", "Name", 50));
            jdbc.update("INSERT INTO rekall_data.project (name) VALUES ('STVV')");

            project.findField("name").orElseThrow().setLength(255);
            save(project);

            DdlPlan plan = plan();

            assertThat(plan.statements()).allSatisfy(s -> assertThat(s.changeClass()).isEqualTo(ChangeClass.SAFE));
            executor.apply(plan);
            assertThat(columnType("project", "name")).isEqualTo("character varying(255)");
        }

        @Test
        @DisplayName("narrowing a varchar is refused")
        void narrowingVarcharIsBlocked() {
            MetaTable project = applyProjectWith(MetaModelFixtures.text("name", "Name", 255));
            project.findField("name").orElseThrow().setLength(50);
            save(project);

            DdlPlan plan = plan();

            assertThat(plan.blocked()).hasSize(1);
            assertThat(plan.blocked().getFirst().warning()).contains("can lose data");
            assertThat(plan.isApplicable()).isFalse();
        }

        @Test
        @DisplayName("varchar to text is safe, text to varchar is refused")
        void textDirectionMatters() {
            MetaTable project = applyProjectWith(MetaModelFixtures.text("name", "Name", 50));
            project.findField("name").orElseThrow().setType(MetaFieldType.LONG_TEXT);
            save(project);

            assertThat(plan().statements()).allSatisfy(s -> assertThat(s.changeClass()).isEqualTo(ChangeClass.SAFE));
            applyAll();

            MetaTable reloaded = metaTables.findByPhysicalName("project").orElseThrow();
            reloaded.findField("name").orElseThrow().setType(MetaFieldType.TEXT);
            save(reloaded);

            assertThat(plan().blocked()).hasSize(1);
        }

        @Test
        @DisplayName("an unrelated type change is refused rather than cast")
        void unrelatedTypeChangeIsBlocked() {
            MetaTable project = applyProjectWith(MetaModelFixtures.text("name", "Name"));
            project.findField("name").orElseThrow().setType(MetaFieldType.INTEGER);
            save(project);

            DdlPlan plan = plan();

            assertThat(plan.blocked()).hasSize(1);
            assertThat(plan.blocked().getFirst().warning()).contains("refused");
            assertThat(columnType("project", "name")).isEqualTo("character varying(255)");
        }

        @Test
        @DisplayName("widening a numeric keeps both the integral and the fractional room")
        void numericWideningRules() {
            MetaTable project = applyProjectWith(MetaModelFixtures.decimal("amount", "Amount", 10, 2));

            project.findField("amount").orElseThrow().setPrecision(12);
            save(project);
            assertThat(plan().statements()).allSatisfy(s -> assertThat(s.changeClass()).isEqualTo(ChangeClass.SAFE));
            applyAll();

            // Precision grows but scale grows more, so the integral part actually shrinks.
            MetaTable reloaded = metaTables.findByPhysicalName("project").orElseThrow();
            reloaded.findField("amount").orElseThrow().setPrecision(13);
            reloaded.findField("amount").orElseThrow().setScale(6);
            save(reloaded);

            assertThat(plan().blocked()).hasSize(1);
        }
    }

    @Nested
    @DisplayName("changing nullability")
    class ChangingNullability {

        @Test
        @DisplayName("relaxing a required column to optional is safe")
        void relaxingIsSafe() {
            MetaTable project = applyProjectWith(MetaModelFixtures.required(MetaModelFixtures.text("name", "Name")));
            project.findField("name").orElseThrow().setNullable(true);
            save(project);

            DdlPlan plan = plan();

            assertThat(plan.statements()).allSatisfy(s -> assertThat(s.changeClass()).isEqualTo(ChangeClass.SAFE));
            executor.apply(plan);
            assertThat(columnIsNullable("project", "name")).isTrue();
        }

        @Test
        @DisplayName("tightening an existing column warns before it can fail on real rows")
        void tighteningWarns() {
            MetaTable project = applyProjectWith(MetaModelFixtures.text("name", "Name"));
            project.findField("name").orElseThrow().setNullable(false);
            save(project);

            DdlPlan plan = plan();

            assertThat(plan.awaitingInput()).hasSize(1);
            assertThat(plan.awaitingInput().getFirst().warning()).contains("roll the whole plan back");
        }
    }

    @Nested
    @DisplayName("dropping things")
    class Dropping {

        @Test
        @DisplayName("dropping a column asks first and reports how many rows are affected")
        void droppingColumnNeedsConfirmation() {
            MetaTable project =
                    applyProjectWith(MetaModelFixtures.text("name", "Name"), MetaModelFixtures.text("code", "Code"));
            jdbc.update("INSERT INTO rekall_data.project (name, code) VALUES ('STVV', 'A')");

            project.removeField(project.findField("code").orElseThrow());
            save(project);

            DdlPlan plan = plan();

            assertThat(plan.awaitingInput()).hasSize(1);
            assertThat(plan.awaitingInput().getFirst().inputKey()).isEqualTo("project.code");
            assertThat(plan.awaitingInput().getFirst().warning()).contains("destroys the values held by 1 row");
        }

        @Test
        @DisplayName("a confirmed column drop is applied")
        void confirmedColumnDropIsApplied() {
            MetaTable project =
                    applyProjectWith(MetaModelFixtures.text("name", "Name"), MetaModelFixtures.text("code", "Code"));
            project.removeField(project.findField("code").orElseThrow());
            save(project);

            applyAll(new PlanOptions(Map.of(), Set.of("project.code")));

            Integer columns = jdbc.queryForObject(
                    """
                    SELECT count(*) FROM information_schema.columns
                    WHERE table_schema = 'rekall_data' AND table_name = 'project' AND column_name = 'code'
                    """,
                    Integer.class);
            assertThat(columns).isZero();
        }

        @Test
        @DisplayName("dropping a table still referenced by a foreign key is refused")
        void droppingReferencedTableIsBlocked() {
            MetaTable environment = save(MetaModelFixtures.environment());
            MetaTable task = MetaModelFixtures.task();
            MetaField environmentId = MetaModelFixtures.reference("environment_id", "Environment");
            task.addField(environmentId);
            save(task);
            metaRelations.saveAndFlush(MetaRelation.manyToOne(
                    task, environment, environmentId.getId(), OnDeleteAction.RESTRICT, "L'ambiente su cui gira"));
            applyAll();

            metaRelations.deleteAll();
            metaTables.delete(metaTables.findByPhysicalName("environment").orElseThrow());
            metaTables.flush();

            DdlPlan plan = plan(new PlanOptions(Map.of(), Set.of("environment")));

            assertThat(plan.blocked()).anySatisfy(statement -> {
                assertThat(statement.description()).contains("Drop table environment");
                assertThat(statement.warning()).contains("Still referenced by task");
            });
        }

        @Test
        @DisplayName("dropping an entity takes its documents with it")
        void droppingTableRemovesItsDocuments() {
            applyProjectWith(MetaModelFixtures.text("name", "Name"));
            jdbc.update("INSERT INTO rekall_data.project (name) VALUES ('STVV')");
            UUID recordId = jdbc.queryForObject("SELECT id FROM rekall_data.project", UUID.class);
            documents.saveAndFlush(
                    new Document("project", recordId, "CONTEXT.md", "context", "# Contesto del progetto"));

            metaTables.delete(metaTables.findByPhysicalName("project").orElseThrow());
            metaTables.flush();

            var result = applyAll(new PlanOptions(Map.of(), Set.of("project")));

            assertThat(tableExists("project")).isFalse();
            assertThat(result.documentsDeleted())
                    .as("the soft reference into rekall_data cannot cascade, so the engine has to")
                    .isEqualTo(1);
            assertThat(documents.count()).isZero();
        }
    }

    @Nested
    @DisplayName("failure handling")
    class FailureHandling {

        /**
         * Builds a plan that is guaranteed to succeed for a while and then fail.
         *
         * <p>The new entity is created in the CREATE_TABLE phase, which always precedes
         * ALTER_COLUMN, so by the time the doomed statement runs there is real work to undo.
         * Without that ordering the rollback assertion would pass trivially.
         */
        private DdlPlan planThatFailsPartWayThrough() {
            MetaTable project = applyProjectWith(MetaModelFixtures.text("name", "Name"));
            jdbc.update("INSERT INTO rekall_data.project (name) VALUES (NULL)");

            save(MetaModelFixtures.task());
            project.findField("name").orElseThrow().setNullable(false);
            save(project);

            DdlPlan plan = plan();
            assertThat(plan.awaitingInput())
                    .as("tightening a column is the change this test relies on failing")
                    .hasSize(1);

            // Forcing the NEEDS_INPUT step through is the point: it is the statement the
            // database will reject, and the planner warned about exactly this outcome.
            return new DdlPlan(
                    plan.planId(),
                    plan.statements().stream()
                            .map(statement -> statement.changeClass() == ChangeClass.NEEDS_INPUT
                                    ? DdlStatement.safe(statement.phase(), statement.sql(), statement.description())
                                    : statement)
                            .toList(),
                    plan.droppedEntities());
        }

        @Test
        @DisplayName("a statement failing rolls the whole plan back, including tables already created")
        void failureRollsBackEverything() {
            DdlPlan forced = planThatFailsPartWayThrough();

            assertThatThrownBy(() -> executor.apply(forced)).isInstanceOf(RuntimeException.class);

            assertThat(tableExists("task"))
                    .as("PostgreSQL DDL is transactional, so the table created earlier in the plan goes too")
                    .isFalse();
            assertThat(columnIsNullable("project", "name")).isTrue();
        }

        @Test
        @DisplayName("the failed attempt is still recorded, on a transaction the rollback cannot undo")
        void failureIsLogged() {
            DdlPlan forced = planThatFailsPartWayThrough();

            assertThatThrownBy(() -> executor.apply(forced)).isInstanceOf(RuntimeException.class);

            var log = ddlLogs.findByPlanIdOrderBySequenceAsc(forced.planId());
            assertThat(log)
                    .as("the record of the attempt survives the rollback that undid it")
                    .isNotEmpty();
            assertThat(log).extracting(DdlLogEntryStatus::of).contains(DdlStatus.ROLLED_BACK, DdlStatus.FAILED);
            assertThat(log.getLast().getStatus()).isEqualTo(DdlStatus.FAILED);
            assertThat(log.getLast().getError()).isNotBlank();
        }
    }

    /** Small indirection so the assertion above reads as a status extraction rather than a lambda. */
    private interface DdlLogEntryStatus {
        static DdlStatus of(dev.rekall.meta.domain.DdlLog entry) {
            return entry.getStatus();
        }
    }
}
