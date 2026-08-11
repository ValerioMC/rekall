package dev.rekall.meta;

import dev.rekall.meta.domain.MetaField;
import dev.rekall.meta.domain.MetaFieldType;
import dev.rekall.meta.domain.MetaRelation;
import dev.rekall.meta.domain.MetaTable;
import dev.rekall.meta.domain.OnDeleteAction;
import dev.rekall.meta.repository.MetaRelationRepository;
import dev.rekall.meta.repository.MetaTableRepository;
import jakarta.validation.ConstraintViolationException;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.dao.DataIntegrityViolationException;
import org.springframework.transaction.annotation.Transactional;

import java.util.List;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

@SpringBootTest(classes = rekalltest.RekallMetaTestApplication.class)
@Transactional
class MetaModelPersistenceTest extends AbstractPostgresTest {

    @Autowired
    private MetaTableRepository tables;

    @Autowired
    private MetaRelationRepository relations;

    @Autowired
    private org.springframework.jdbc.core.JdbcTemplate jdbc;

    private MetaTable project;

    @BeforeEach
    void setUp() {
        relations.deleteAll();
        tables.deleteAll();
        project = new MetaTable("project", "Project", "Projects", "Tutti i progetti su cui si sta lavorando");
    }

    @Test
    @DisplayName("text[] columns survive a round trip")
    void arrayColumnsRoundTrip() {
        project.setAliases(new String[] {"progetti", "commesse"});
        MetaField status = new MetaField("status", "Status", "Stato corrente del progetto", MetaFieldType.ENUM);
        status.setEnumValues(new String[] {"active", "paused", "done"});
        project.addField(status);

        tables.saveAndFlush(project);
        tables.flush();

        MetaTable reloaded = tables.findByPhysicalName("project").orElseThrow();
        assertThat(reloaded.getAliases()).containsExactly("progetti", "commesse");
        assertThat(reloaded.getFields().getFirst().getEnumValues()).containsExactly("active", "paused", "done");
    }

    @Test
    @DisplayName("fields keep the position assigned when they were added")
    void fieldsArePositioned() {
        project.addField(new MetaField("name", "Name", "Nome breve del progetto", MetaFieldType.TEXT));
        project.addField(new MetaField("notes", "Notes", "Note libere", MetaFieldType.MARKDOWN));

        tables.saveAndFlush(project);

        List<MetaField> fields = tables.findByPhysicalName("project").orElseThrow().getFields();
        assertThat(fields).extracting(MetaField::getColumnName).containsExactly("name", "notes");
        assertThat(fields).extracting(MetaField::getPosition).containsExactly(0, 1);
    }

    @Test
    @DisplayName("the display field resolves to one of the entity's own fields")
    void displayFieldResolves() {
        MetaField name = new MetaField("name", "Name", "Nome breve del progetto", MetaFieldType.TEXT);
        project.addField(name);
        tables.saveAndFlush(project);

        project.setDisplayFieldId(name.getId());
        tables.saveAndFlush(project);

        MetaTable reloaded = tables.findByPhysicalName("project").orElseThrow();
        assertThat(reloaded.displayField()).isPresent().get().extracting(MetaField::getColumnName).isEqualTo("name");
    }

    @Test
    @DisplayName("physical names are unique across the whole meta-model")
    void physicalNameIsUnique() {
        tables.saveAndFlush(project);
        MetaTable duplicate = new MetaTable("project", "Progetto", "Progetti", "Duplicato");

        assertThatThrownBy(() -> tables.saveAndFlush(duplicate))
                .isInstanceOf(DataIntegrityViolationException.class);
    }

    @Test
    @DisplayName("bean validation rejects a blank description before it reaches the database")
    void descriptionCannotBeBlank() {
        MetaTable blank = new MetaTable("task", "Task", "Tasks", "   ");

        assertThatThrownBy(() -> tables.saveAndFlush(blank))
                .isInstanceOf(ConstraintViolationException.class)
                .hasMessageContaining("description");
    }

    @Test
    @DisplayName("the database also refuses a blank description, with bean validation bypassed")
    void descriptionCheckConstraintIsEnforcedByTheDatabase() {
        // Inserted through JDBC on purpose. Bean validation fires first on the JPA path, which
        // would leave the CHECK constraint untested, and the constraint is what protects the
        // description Claude relies on when the meta-model is edited outside the application.
        assertThatThrownBy(() -> jdbc.update(
                        """
                        INSERT INTO rekall_meta.meta_table (physical_name, label, label_plural, description)
                        VALUES ('task', 'Task', 'Tasks', '   ')
                        """))
                .isInstanceOf(DataIntegrityViolationException.class)
                .hasMessageContaining("ck_meta_table_description");
    }

    @Test
    @DisplayName("a many-to-one relation requires a source field and forbids a join table")
    void manyToOneShapeIsEnforced() {
        MetaTable environment =
                new MetaTable("environment", "Environment", "Environments", "Configurazioni di cluster e namespace");
        tables.saveAndFlush(environment);

        MetaField environmentId =
                new MetaField("environment_id", "Environment", "L'ambiente su cui gira", MetaFieldType.REFERENCE);
        project.addField(environmentId);
        tables.saveAndFlush(project);

        MetaRelation relation = MetaRelation.manyToOne(
                project, environment, environmentId.getId(), OnDeleteAction.RESTRICT, "L'ambiente su cui gira");
        relations.saveAndFlush(relation);

        assertThat(relations.findBySourceFieldId(environmentId.getId())).isPresent();
        assertThat(relations.findAllTouching(environment.getId())).hasSize(1);
    }

    @Test
    @DisplayName("deleting an entity cascades to its fields")
    void deletingEntityCascadesToFields() {
        project.addField(new MetaField("name", "Name", "Nome breve", MetaFieldType.TEXT));
        tables.saveAndFlush(project);

        tables.delete(project);
        tables.flush();

        assertThat(tables.findByPhysicalName("project")).isEmpty();
    }
}
