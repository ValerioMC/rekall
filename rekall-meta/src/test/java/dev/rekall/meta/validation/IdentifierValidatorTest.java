package dev.rekall.meta.validation;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.ValueSource;

import static org.assertj.core.api.Assertions.assertThatCode;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

class IdentifierValidatorTest {

    @Nested
    @DisplayName("table names")
    class TableNames {

        @ParameterizedTest
        @ValueSource(strings = {"project", "task", "environment", "code_validator", "a", "t1", "a_b_c_9"})
        void acceptsWellFormedNames(String name) {
            assertThatCode(() -> IdentifierValidator.validateTableName(name)).doesNotThrowAnyException();
        }

        @ParameterizedTest
        @DisplayName("rejects anything that would need quoting to survive a hand-written query")
        @ValueSource(strings = {"Project", "1task", "my-table", "my table", "café", "task;", "task--", "_task", "TASK"})
        void rejectsMalformedNames(String name) {
            assertThatThrownBy(() -> IdentifierValidator.validateTableName(name))
                    .isInstanceOf(InvalidIdentifierException.class);
        }

        @ParameterizedTest
        @DisplayName("rejects reserved keywords even though quoting would technically work")
        @ValueSource(strings = {"select", "table", "user", "order", "group", "check", "default", "references"})
        void rejectsReservedWords(String name) {
            assertThatThrownBy(() -> IdentifierValidator.validateTableName(name))
                    .isInstanceOf(InvalidIdentifierException.class)
                    .hasMessageContaining("reserved");
        }

        @Test
        @DisplayName("rejects names longer than the 63 byte Postgres identifier limit")
        void rejectsOverlongNames() {
            String tooLong = "a".repeat(64);

            assertThatThrownBy(() -> IdentifierValidator.validateTableName(tooLong))
                    .isInstanceOf(InvalidIdentifierException.class);

            assertThatCode(() -> IdentifierValidator.validateTableName("a".repeat(63)))
                    .doesNotThrowAnyException();
        }

        @Test
        void rejectsNullAndBlank() {
            assertThatThrownBy(() -> IdentifierValidator.validateTableName(null))
                    .isInstanceOf(InvalidIdentifierException.class);
            assertThatThrownBy(() -> IdentifierValidator.validateTableName("   "))
                    .isInstanceOf(InvalidIdentifierException.class);
        }
    }

    @Nested
    @DisplayName("column names")
    class ColumnNames {

        @ParameterizedTest
        @DisplayName("rejects the system columns generated on every table")
        @ValueSource(strings = {"id", "created_at", "updated_at"})
        void rejectsSystemColumns(String name) {
            assertThatThrownBy(() -> IdentifierValidator.validateColumnName(name))
                    .isInstanceOf(InvalidIdentifierException.class)
                    .hasMessageContaining("system column");
        }

        @Test
        void acceptsOrdinaryColumns() {
            assertThatCode(() -> IdentifierValidator.validateColumnName("environment_id"))
                    .doesNotThrowAnyException();
        }

        @Test
        @DisplayName("system column names are still valid as table names")
        void systemColumnNamesAreFineAsTableNames() {
            assertThatCode(() -> IdentifierValidator.validateTableName("id")).doesNotThrowAnyException();
        }
    }

    @Test
    void isValidTableNameMirrorsTheThrowingVariant() {
        assertThatCode(() -> IdentifierValidator.validateTableName("project")).doesNotThrowAnyException();

        org.assertj.core.api.Assertions.assertThat(IdentifierValidator.isValidTableName("project")).isTrue();
        org.assertj.core.api.Assertions.assertThat(IdentifierValidator.isValidTableName("select")).isFalse();
    }
}
