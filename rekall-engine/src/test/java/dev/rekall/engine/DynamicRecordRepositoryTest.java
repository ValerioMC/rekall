package dev.rekall.engine;

import dev.rekall.engine.data.DynamicRecordRepository;
import dev.rekall.engine.data.Operator;
import dev.rekall.engine.data.QueryFilter;
import dev.rekall.engine.data.RecordView;
import dev.rekall.engine.data.UnknownFieldException;
import dev.rekall.engine.type.ValueCoercionException;
import dev.rekall.meta.MetaModelFixtures;
import dev.rekall.meta.domain.MetaField;
import dev.rekall.meta.domain.MetaRelation;
import dev.rekall.meta.domain.MetaTable;
import dev.rekall.meta.domain.OnDeleteAction;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;

import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.UUID;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

class DynamicRecordRepositoryTest extends AbstractEngineTest {

    @Autowired
    private DynamicRecordRepository records;

    /** project(name display, status enum, labels tags) with a required reference to environment. */
    private MetaTable projectWithEverything() {
        MetaTable environment = MetaModelFixtures.environment();
        MetaField environmentName = MetaModelFixtures.text("label", "Label");
        environment.addField(environmentName);
        environment.addField(MetaModelFixtures.text("namespace", "Namespace"));
        save(environment);
        environment.setDisplayFieldId(environmentName.getId());
        save(environment);

        MetaTable project = MetaModelFixtures.project();
        MetaField name = MetaModelFixtures.text("name", "Name");
        project.addField(name);
        project.addField(MetaModelFixtures.enumeration("status", "Status", "active", "paused", "done"));
        project.addField(MetaModelFixtures.tags("labels", "Labels"));
        MetaField environmentId = MetaModelFixtures.reference("environment_id", "Environment");
        project.addField(environmentId);
        save(project);
        project.setDisplayFieldId(name.getId());
        save(project);

        metaRelations.saveAndFlush(MetaRelation.manyToOne(
                project, environment, environmentId.getId(), OnDeleteAction.RESTRICT, "L'ambiente su cui gira"));

        applyAll();
        schemaRegistry.invalidate();
        return schemaRegistry.entity("project").orElseThrow();
    }

    @Test
    @DisplayName("a row survives a full write and read cycle with every value intact")
    void insertAndReadBack() {
        MetaTable project = projectWithEverything();

        Map<String, Object> values = new HashMap<>();
        values.put("name", "STVV");
        values.put("status", "active");
        values.put("labels", List.of("esa", "backend"));
        UUID id = records.insert(project, values);

        RecordView view = records.findById(project, id, 0).orElseThrow();

        assertThat(view.value("name")).isEqualTo("STVV");
        assertThat(view.value("status")).isEqualTo("active");
        assertThat((String[]) view.value("labels")).containsExactly("esa", "backend");
        assertThat(view.label()).as("the label comes from the display field").isEqualTo("STVV");
        assertThat(view.createdAt()).isNotNull();
    }

    @Test
    @DisplayName("naming a field the entity does not have fails loudly instead of being ignored")
    void unknownFieldIsRejected() {
        MetaTable project = projectWithEverything();

        assertThatThrownBy(() -> records.insert(project, Map.of("nome", "STVV")))
                .isInstanceOf(UnknownFieldException.class)
                .hasMessageContaining("has no field 'nome'")
                .hasMessageContaining("name");
    }

    @Test
    @DisplayName("a value outside the enum is rejected before it reaches the database")
    void enumValueIsValidated() {
        MetaTable project = projectWithEverything();

        assertThatThrownBy(() -> records.insert(project, Map.of("status", "cancelled")))
                .isInstanceOf(ValueCoercionException.class)
                .hasMessageContaining("active, paused, done");
    }

    @Test
    @DisplayName("system columns cannot be assigned by hand")
    void systemColumnsAreIgnored() {
        MetaTable project = projectWithEverything();
        UUID forced = UUID.randomUUID();

        UUID id = records.insert(project, Map.of("name", "STVV", "id", forced));

        assertThat(id).isNotEqualTo(forced);
    }

    @Test
    @DisplayName("a reference is resolved into the full target record, not left as a uuid")
    void referencesAreResolved() {
        MetaTable project = projectWithEverything();
        MetaTable environment = schemaRegistry.entity("environment").orElseThrow();

        UUID environmentId = records.insert(
                environment, Map.of("label", "kmaster14 / stvv-dev", "namespace", "stvv-dev"));
        UUID projectId = records.insert(project, Map.of("name", "STVV", "environment_id", environmentId));

        RecordView resolved = records.findById(project, projectId, 1).orElseThrow();

        assertThat(resolved.value("environment_id")).isInstanceOf(RecordView.class);
        RecordView environmentView = (RecordView) resolved.value("environment_id");
        assertThat(environmentView.label()).isEqualTo("kmaster14 / stvv-dev");
        assertThat(environmentView.value("namespace"))
                .as("this is the point: the task carries the environment config, not an id")
                .isEqualTo("stvv-dev");
    }

    @Test
    @DisplayName("depth zero leaves the raw id, so a listing does not drag every target along")
    void depthZeroDoesNotResolve() {
        MetaTable project = projectWithEverything();
        MetaTable environment = schemaRegistry.entity("environment").orElseThrow();
        UUID environmentId = records.insert(environment, Map.of("label", "kmaster14"));
        UUID projectId = records.insert(project, Map.of("name", "STVV", "environment_id", environmentId));

        RecordView view = records.findById(project, projectId, 0).orElseThrow();

        assertThat(view.value("environment_id")).isEqualTo(environmentId);
    }

    @Test
    @DisplayName("a record can be found by the value of its display field, not only by id")
    void findsByLabel() {
        MetaTable project = projectWithEverything();
        UUID id = records.insert(project, Map.of("name", "code-validator-main-workflow"));

        assertThat(records.findByLabel(project, "code-validator-main-workflow", 0))
                .singleElement()
                .extracting(RecordView::id)
                .isEqualTo(id);
        assertThat(records.findByLabel(project, id.toString(), 0)).singleElement();
        assertThat(records.findByLabel(project, "does-not-exist", 0)).isEmpty();
    }

    @Test
    @DisplayName("filters cover equality, patterns, membership, nullity and tag containment")
    void filtersWork() {
        MetaTable project = projectWithEverything();
        records.insert(project, Map.of("name", "STVV", "status", "active", "labels", List.of("esa", "backend")));
        records.insert(project, Map.of("name", "AINABLER", "status", "paused", "labels", List.of("esa", "ai")));
        records.insert(project, Map.of("name", "Sandbox"));

        assertThat(records.query(project, QueryFilter.where(QueryFilter.Condition.eq("status", "active")), 0))
                .hasSize(1);
        assertThat(records.query(
                        project,
                        QueryFilter.where(new QueryFilter.Condition("name", Operator.ILIKE, "st%")),
                        0))
                .hasSize(1);
        assertThat(records.query(
                        project,
                        QueryFilter.where(new QueryFilter.Condition("status", Operator.IN, List.of("active", "paused"))),
                        0))
                .hasSize(2);
        assertThat(records.query(project, QueryFilter.where(QueryFilter.Condition.isNull("status")), 0))
                .hasSize(1);
        assertThat(records.query(
                        project,
                        QueryFilter.where(new QueryFilter.Condition("labels", Operator.CONTAINS, "ai")),
                        0))
                .singleElement()
                .extracting(view -> view.value("name"))
                .isEqualTo("AINABLER");
    }

    @Test
    @DisplayName("results are newest first by default, since that is the task most likely wanted")
    void defaultSortIsMostRecentlyUpdated() {
        MetaTable project = projectWithEverything();
        records.insert(project, Map.of("name", "first"));
        jdbc.execute("SELECT pg_sleep(0.01)");
        records.insert(project, Map.of("name", "second"));

        List<RecordView> results = records.query(project, QueryFilter.all(), 0);

        assertThat(results).extracting(view -> view.value("name")).containsExactly("second", "first");
    }

    @Test
    @DisplayName("an explicit sort overrides the default")
    void explicitSortIsHonoured() {
        MetaTable project = projectWithEverything();
        records.insert(project, Map.of("name", "b"));
        records.insert(project, Map.of("name", "a"));

        List<RecordView> results = records.query(
                project,
                new QueryFilter(List.of(), List.of(QueryFilter.SortSpec.asc("name")), 10, 0),
                0);

        assertThat(results).extracting(view -> view.value("name")).containsExactly("a", "b");
    }

    @Test
    @DisplayName("the limit is capped so no single read can flood a context window")
    void limitIsCapped() {
        QueryFilter filter = new QueryFilter(List.of(), List.of(), 10_000, 0);

        assertThat(filter.limit()).isEqualTo(QueryFilter.MAX_LIMIT);
    }

    @Test
    @DisplayName("update changes only the given fields and refuses a missing row")
    void updateAndDelete() {
        MetaTable project = projectWithEverything();
        UUID id = records.insert(project, Map.of("name", "STVV", "status", "active"));

        records.update(project, id, Map.of("status", "done"));
        RecordView updated = records.findById(project, id, 0).orElseThrow();

        assertThat(updated.value("status")).isEqualTo("done");
        assertThat(updated.value("name")).as("fields not mentioned are untouched").isEqualTo("STVV");

        assertThat(records.delete(project, id)).isTrue();
        assertThat(records.delete(project, id)).isFalse();
        assertThat(records.count(project, QueryFilter.all())).isZero();
    }

    @Test
    @DisplayName("counting ignores limit and offset")
    void countIgnoresPaging() {
        MetaTable project = projectWithEverything();
        records.insert(project, Map.of("name", "a"));
        records.insert(project, Map.of("name", "b"));
        records.insert(project, Map.of("name", "c"));

        long total = records.count(project, new QueryFilter(List.of(), List.of(), 1, 0));

        assertThat(total).isEqualTo(3);
    }
}
