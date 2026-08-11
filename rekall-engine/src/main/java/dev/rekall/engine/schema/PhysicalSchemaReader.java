package dev.rekall.engine.schema;

import dev.rekall.engine.RekallSchemas;
import org.jooq.DSLContext;
import org.jooq.Record;
import org.jooq.Result;
import org.springframework.stereotype.Component;
import org.springframework.transaction.annotation.Transactional;

import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.Map;

/**
 * Reads the live state of {@code rekall_data}.
 *
 * <p>Queries {@code information_schema} and {@code pg_catalog} directly rather than going
 * through jOOQ's {@code Meta} API: the diff needs character lengths, numeric scale and the
 * referential action of each foreign key, and those are only available at this level of
 * detail. Every query is a constant string with bound parameters.
 */
@Component
public class PhysicalSchemaReader {

    private final DSLContext dsl;

    public PhysicalSchemaReader(DSLContext dsl) {
        this.dsl = dsl;
    }

    @Transactional(readOnly = true)
    public PhysicalSchema read() {
        Map<String, Map<String, PhysicalColumn>> columnsByTable = readColumns();
        Map<String, Map<String, PhysicalForeignKey>> foreignKeysByTable = readForeignKeys();

        Map<String, PhysicalTable> tables = new LinkedHashMap<>();
        columnsByTable.forEach((tableName, columns) -> tables.put(
                tableName,
                new PhysicalTable(
                        tableName, columns, foreignKeysByTable.getOrDefault(tableName, Map.of()))));

        return new PhysicalSchema(tables);
    }

    private Map<String, Map<String, PhysicalColumn>> readColumns() {
        Result<Record> rows = dsl.fetch(
                """
                SELECT c.table_name, c.column_name, c.data_type, c.udt_name,
                       c.character_maximum_length, c.numeric_precision, c.numeric_scale,
                       c.is_nullable, c.column_default
                FROM information_schema.columns c
                JOIN information_schema.tables t
                  ON t.table_schema = c.table_schema AND t.table_name = c.table_name
                WHERE c.table_schema = ? AND t.table_type = 'BASE TABLE'
                ORDER BY c.table_name, c.ordinal_position
                """,
                RekallSchemas.DATA);

        Map<String, Map<String, PhysicalColumn>> byTable = new LinkedHashMap<>();
        for (Record row : rows) {
            String tableName = row.get("table_name", String.class);
            PhysicalColumn column = new PhysicalColumn(
                    row.get("column_name", String.class),
                    row.get("data_type", String.class),
                    row.get("udt_name", String.class),
                    row.get("character_maximum_length", Integer.class),
                    row.get("numeric_precision", Integer.class),
                    row.get("numeric_scale", Integer.class),
                    "YES".equals(row.get("is_nullable", String.class)),
                    row.get("column_default", String.class));
            byTable.computeIfAbsent(tableName, key -> new LinkedHashMap<>()).put(column.name(), column);
        }
        return byTable;
    }

    private Map<String, Map<String, PhysicalForeignKey>> readForeignKeys() {
        Result<Record> rows = dsl.fetch(
                """
                SELECT con.conname          AS constraint_name,
                       src.relname          AS source_table,
                       att.attname          AS source_column,
                       tgt.relname          AS target_table,
                       con.confdeltype      AS on_delete
                FROM pg_constraint con
                JOIN pg_class src        ON src.oid = con.conrelid
                JOIN pg_class tgt        ON tgt.oid = con.confrelid
                JOIN pg_namespace ns     ON ns.oid = src.relnamespace
                JOIN pg_attribute att    ON att.attrelid = con.conrelid AND att.attnum = con.conkey[1]
                WHERE con.contype = 'f' AND ns.nspname = ?
                """,
                RekallSchemas.DATA);

        Map<String, Map<String, PhysicalForeignKey>> byTable = new HashMap<>();
        for (Record row : rows) {
            PhysicalForeignKey foreignKey = new PhysicalForeignKey(
                    row.get("constraint_name", String.class),
                    row.get("source_table", String.class),
                    row.get("source_column", String.class),
                    row.get("target_table", String.class),
                    decodeOnDelete(row.get("on_delete", String.class)));
            byTable.computeIfAbsent(foreignKey.sourceTable(), key -> new LinkedHashMap<>())
                    .put(foreignKey.constraintName(), foreignKey);
        }
        return byTable;
    }

    private String decodeOnDelete(String code) {
        return switch (code) {
            case "a" -> "NO ACTION";
            case "r" -> "RESTRICT";
            case "c" -> "CASCADE";
            case "n" -> "SET NULL";
            case "d" -> "SET DEFAULT";
            default -> "UNKNOWN";
        };
    }

    /** Row count of a generated table, used to classify whether a change needs a backfill. */
    @Transactional(readOnly = true)
    public long countRows(String tableName) {
        return dsl.fetchCount(org.jooq.impl.DSL.table(org.jooq.impl.DSL.name(RekallSchemas.DATA, tableName)));
    }
}
