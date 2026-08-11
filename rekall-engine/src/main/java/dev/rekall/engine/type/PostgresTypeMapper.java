package dev.rekall.engine.type;

import dev.rekall.meta.domain.MetaField;
import dev.rekall.meta.domain.MetaFieldType;
import org.jooq.DataType;
import org.jooq.impl.SQLDataType;
import org.springframework.stereotype.Component;

import java.math.BigDecimal;
import java.time.LocalDate;
import java.time.OffsetDateTime;
import java.util.UUID;

/**
 * The single place where a user-chosen {@link MetaFieldType} becomes a physical Postgres type.
 *
 * <p>Nothing else in the engine is allowed to decide a column type, which is what keeps the
 * set of types genuinely closed: adding one requires a mapping here, a comparison rule in
 * {@code PhysicalTypeComparator} and a coercion in {@link #toJavaValue}.
 */
@Component
public class PostgresTypeMapper {

    /** Physical column type for a field, including its nullability. */
    public DataType<?> toDataType(MetaField field) {
        return baseType(field).nullable(field.isNullable());
    }

    /** Physical column type ignoring nullability, used when only the type is being altered. */
    public DataType<?> baseType(MetaField field) {
        return switch (field.getType()) {
            // ENUM shares varchar with TEXT. What constrains it is a generated CHECK, not a
            // Postgres enum type, because altering a Postgres enum is not transactional.
            case TEXT, ENUM -> SQLDataType.VARCHAR(field.effectiveLength());
            case LONG_TEXT, MARKDOWN -> SQLDataType.CLOB;
            case INTEGER -> SQLDataType.BIGINT;
            case DECIMAL -> SQLDataType.NUMERIC(field.effectivePrecision(), field.effectiveScale());
            case BOOLEAN -> SQLDataType.BOOLEAN;
            case DATE -> SQLDataType.LOCALDATE;
            case TIMESTAMP -> SQLDataType.TIMESTAMPWITHTIMEZONE;
            case TAGS -> SQLDataType.CLOB.getArrayDataType();
            case REFERENCE -> SQLDataType.UUID;
        };
    }

    /** Java type a value of this field is bound as. */
    public Class<?> toJavaType(MetaFieldType type) {
        return switch (type) {
            case TEXT, LONG_TEXT, MARKDOWN, ENUM -> String.class;
            case INTEGER -> Long.class;
            case DECIMAL -> BigDecimal.class;
            case BOOLEAN -> Boolean.class;
            case DATE -> LocalDate.class;
            case TIMESTAMP -> OffsetDateTime.class;
            case TAGS -> String[].class;
            case REFERENCE -> UUID.class;
        };
    }

    /**
     * Coerces a loosely typed inbound value, typically parsed from JSON, into the Java type the
     * column expects.
     *
     * @throws ValueCoercionException if the value cannot represent the field's type
     */
    public Object toJavaValue(MetaField field, Object raw) {
        if (raw == null) {
            return null;
        }
        try {
            return switch (field.getType()) {
                case TEXT, LONG_TEXT, MARKDOWN, ENUM -> coerceEnum(field, String.valueOf(raw));
                case INTEGER -> raw instanceof Number n ? n.longValue() : Long.parseLong(raw.toString().trim());
                case DECIMAL -> raw instanceof BigDecimal b ? b : new BigDecimal(raw.toString().trim());
                case BOOLEAN -> raw instanceof Boolean b ? b : parseBoolean(raw.toString().trim());
                case DATE -> raw instanceof LocalDate d ? d : LocalDate.parse(raw.toString().trim());
                case TIMESTAMP -> raw instanceof OffsetDateTime t ? t : OffsetDateTime.parse(raw.toString().trim());
                case TAGS -> toStringArray(raw);
                case REFERENCE -> raw instanceof UUID u ? u : UUID.fromString(raw.toString().trim());
            };
        } catch (ValueCoercionException e) {
            throw e;
        } catch (RuntimeException e) {
            throw new ValueCoercionException(
                    "Value '%s' is not a valid %s for field '%s'".formatted(raw, field.getType(), field.getColumnName()),
                    e);
        }
    }

    private String coerceEnum(MetaField field, String value) {
        if (field.getType() != MetaFieldType.ENUM) {
            return value;
        }
        String[] allowed = field.getEnumValues();
        for (String candidate : allowed) {
            if (candidate.equals(value)) {
                return value;
            }
        }
        throw new ValueCoercionException(
                "Value '%s' is not one of the allowed values %s for field '%s'"
                        .formatted(value, String.join(", ", allowed), field.getColumnName()),
                null);
    }

    private boolean parseBoolean(String value) {
        if ("true".equalsIgnoreCase(value) || "false".equalsIgnoreCase(value)) {
            return Boolean.parseBoolean(value);
        }
        throw new ValueCoercionException("Value '%s' is not a boolean".formatted(value), null);
    }

    private String[] toStringArray(Object raw) {
        if (raw instanceof String[] array) {
            return array;
        }
        if (raw instanceof Iterable<?> iterable) {
            java.util.List<String> values = new java.util.ArrayList<>();
            iterable.forEach(item -> values.add(String.valueOf(item)));
            return values.toArray(String[]::new);
        }
        if (raw instanceof Object[] array) {
            String[] values = new String[array.length];
            for (int i = 0; i < array.length; i++) {
                values[i] = String.valueOf(array[i]);
            }
            return values;
        }
        throw new ValueCoercionException("Value '%s' is not a list of tags".formatted(raw), null);
    }
}
