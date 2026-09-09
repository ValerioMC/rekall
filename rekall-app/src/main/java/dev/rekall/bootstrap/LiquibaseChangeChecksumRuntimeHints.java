package dev.rekall.bootstrap;

import org.springframework.aot.hint.MemberCategory;
import org.springframework.aot.hint.RuntimeHints;
import org.springframework.aot.hint.RuntimeHintsRegistrar;
import org.springframework.aot.hint.TypeReference;

import java.util.List;

/**
 * Native-image reflection hints for every Liquibase {@code Change} type the changelog can use.
 *
 * <p>On every startup Liquibase's {@code ValidatingVisitor} regenerates the checksum of each
 * changeSet, and that serialises the change by reflectively calling every getter through
 * {@code ChangeParameterMetaData.getCurrentValue}. A change type whose members are not
 * registered for reflection fails the whole context with
 * {@code MissingReflectionRegistrationError}, e.g. on {@code DropDefaultValueChange.getCatalogName()}
 * once {@code 016-task-step-draft} introduced {@code dropDefaultValue}.
 *
 * <p>The bundled GraalVM reachability metadata for {@code liquibase-core} only covers the change
 * types its own trace exercised and gates them on conditions that do not all hold for the
 * checksum path, so a new migration breaks the native build until this list is regenerated.
 * Registering the full {@code liquibase.change.core.*Change} set plus the column and constraint
 * config classes keeps any future migration safe. This is picked up by {@code process-aot}
 * through {@code META-INF/spring/aot.factories} and folded into the AOT reachability metadata.
 */
public class LiquibaseChangeChecksumRuntimeHints implements RuntimeHintsRegistrar {

    private static final List<String> CHANGE_TYPES = List.of(
            "liquibase.change.core.AbstractModifyDataChange",
            "liquibase.change.core.AddAutoIncrementChange",
            "liquibase.change.core.AddColumnChange",
            "liquibase.change.core.AddDefaultValueChange",
            "liquibase.change.core.AddForeignKeyConstraintChange",
            "liquibase.change.core.AddLookupTableChange",
            "liquibase.change.core.AddNotNullConstraintChange",
            "liquibase.change.core.AddPrimaryKeyChange",
            "liquibase.change.core.AddUniqueConstraintChange",
            "liquibase.change.core.AlterSequenceChange",
            "liquibase.change.core.CreateIndexChange",
            "liquibase.change.core.CreateProcedureChange",
            "liquibase.change.core.CreateSequenceChange",
            "liquibase.change.core.CreateTableChange",
            "liquibase.change.core.CreateViewChange",
            "liquibase.change.core.DeleteDataChange",
            "liquibase.change.core.DropAllForeignKeyConstraintsChange",
            "liquibase.change.core.DropColumnChange",
            "liquibase.change.core.DropDefaultValueChange",
            "liquibase.change.core.DropForeignKeyConstraintChange",
            "liquibase.change.core.DropIndexChange",
            "liquibase.change.core.DropNotNullConstraintChange",
            "liquibase.change.core.DropPrimaryKeyChange",
            "liquibase.change.core.DropProcedureChange",
            "liquibase.change.core.DropSequenceChange",
            "liquibase.change.core.DropTableChange",
            "liquibase.change.core.DropUniqueConstraintChange",
            "liquibase.change.core.DropViewChange",
            "liquibase.change.core.EmptyChange",
            "liquibase.change.core.ExecuteShellCommandChange",
            "liquibase.change.core.InsertDataChange",
            "liquibase.change.core.LoadDataChange",
            "liquibase.change.core.LoadUpdateDataChange",
            "liquibase.change.core.MergeColumnChange",
            "liquibase.change.core.ModifyDataTypeChange",
            "liquibase.change.core.OutputChange",
            "liquibase.change.core.RawSQLChange",
            "liquibase.change.core.RenameColumnChange",
            "liquibase.change.core.RenameSequenceChange",
            "liquibase.change.core.RenameTableChange",
            "liquibase.change.core.RenameViewChange",
            "liquibase.change.core.SetColumnRemarksChange",
            "liquibase.change.core.SetTableRemarksChange",
            "liquibase.change.core.SQLFileChange",
            "liquibase.change.core.StopChange",
            "liquibase.change.core.TagDatabaseChange",
            "liquibase.change.core.UpdateDataChange",
            "liquibase.change.AbstractChange",
            "liquibase.change.AbstractSQLChange",
            "liquibase.change.AbstractTableChange",
            "liquibase.change.AddColumnConfig",
            "liquibase.change.ColumnConfig",
            "liquibase.change.ConstraintsConfig",
            "liquibase.change.core.LoadDataColumnConfig",
            "liquibase.statement.DatabaseFunction",
            "liquibase.statement.SequenceNextValueFunction",
            "liquibase.statement.SequenceCurrentValueFunction",
            "liquibase.statement.NotNullConstraint");

    @Override
    public void registerHints(RuntimeHints hints, ClassLoader classLoader) {
        for (String type : CHANGE_TYPES) {
            hints.reflection().registerType(TypeReference.of(type),
                    MemberCategory.INVOKE_DECLARED_CONSTRUCTORS,
                    MemberCategory.INVOKE_DECLARED_METHODS,
                    MemberCategory.DECLARED_FIELDS);
        }
    }
}
