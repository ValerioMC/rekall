package dev.rekall.engine.plan;

import dev.rekall.engine.schema.PhysicalColumn;
import dev.rekall.meta.domain.MetaField;
import org.springframework.stereotype.Component;

/**
 * Decides whether a declared field still matches the column in the database, and if not,
 * whether moving to it can lose data.
 *
 * <p>The rule is deliberately conservative: only changes that strictly add capacity count as
 * safe. Everything else is refused rather than attempted with a cast.
 */
@Component
public class PhysicalTypeComparator {

    public TypeComparison compare(MetaField field, PhysicalColumn column) {
        return switch (field.getType()) {
            case TEXT, ENUM -> compareVarchar(field.effectiveLength(), column);
            case LONG_TEXT, MARKDOWN -> compareText(column);
            case INTEGER -> compareInteger(column);
            case DECIMAL -> compareNumeric(field.effectivePrecision(), field.effectiveScale(), column);
            case BOOLEAN -> exact("boolean", column);
            case DATE -> exact("date", column);
            case TIMESTAMP -> exact("timestamp with time zone", column);
            case REFERENCE -> exact("uuid", column);
            case TAGS -> compareTextArray(column);
        };
    }

    private TypeComparison compareVarchar(int declaredLength, PhysicalColumn column) {
        if (!"character varying".equals(column.dataType())) {
            return TypeComparison.UNSAFE;
        }
        Integer actual = column.characterMaximumLength();
        if (actual == null) {
            // varchar with no length holds anything, so narrowing it to a bounded varchar could
            // truncate. Only the reverse direction is safe.
            return TypeComparison.UNSAFE;
        }
        if (actual == declaredLength) {
            return TypeComparison.MATCHES;
        }
        return actual < declaredLength ? TypeComparison.SAFE_WIDENING : TypeComparison.UNSAFE;
    }

    private TypeComparison compareText(PhysicalColumn column) {
        if ("text".equals(column.dataType())) {
            return TypeComparison.MATCHES;
        }
        // Every varchar value fits in text, so this direction never truncates.
        return "character varying".equals(column.dataType()) ? TypeComparison.SAFE_WIDENING : TypeComparison.UNSAFE;
    }

    private TypeComparison compareInteger(PhysicalColumn column) {
        if ("bigint".equals(column.dataType())) {
            return TypeComparison.MATCHES;
        }
        return switch (column.dataType()) {
            case "integer", "smallint" -> TypeComparison.SAFE_WIDENING;
            default -> TypeComparison.UNSAFE;
        };
    }

    private TypeComparison compareNumeric(int declaredPrecision, int declaredScale, PhysicalColumn column) {
        if (!"numeric".equals(column.dataType())) {
            return TypeComparison.UNSAFE;
        }
        Integer precision = column.numericPrecision();
        Integer scale = column.numericScale();
        if (precision == null || scale == null) {
            return TypeComparison.UNSAFE;
        }
        if (precision == declaredPrecision && scale == declaredScale) {
            return TypeComparison.MATCHES;
        }
        // Both the integral part and the fractional part must keep at least the room they had.
        boolean integralRoomKept = declaredPrecision - declaredScale >= precision - scale;
        boolean fractionalRoomKept = declaredScale >= scale;
        return integralRoomKept && fractionalRoomKept ? TypeComparison.SAFE_WIDENING : TypeComparison.UNSAFE;
    }

    private TypeComparison compareTextArray(PhysicalColumn column) {
        boolean isTextArray = "ARRAY".equals(column.dataType()) && "_text".equals(column.udtName());
        return isTextArray ? TypeComparison.MATCHES : TypeComparison.UNSAFE;
    }

    private TypeComparison exact(String expectedDataType, PhysicalColumn column) {
        return expectedDataType.equals(column.dataType()) ? TypeComparison.MATCHES : TypeComparison.UNSAFE;
    }
}
