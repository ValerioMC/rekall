package dev.rekall.bootstrap;

import org.springframework.aot.hint.MemberCategory;
import org.springframework.aot.hint.RuntimeHints;
import org.springframework.aot.hint.RuntimeHintsRegistrar;
import org.springframework.aot.hint.TypeReference;

import java.util.List;

/**
 * Native-image reflection hints for every Liquibase {@code Change} type. Liquibase reflects over
 * every change getter on each startup to regenerate checksums, and the bundled GraalVM metadata
 * misses the checksum path, so an unregistered type fails the context with
 * {@code MissingReflectionRegistrationError}. Registering the full set keeps future migrations safe.
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
