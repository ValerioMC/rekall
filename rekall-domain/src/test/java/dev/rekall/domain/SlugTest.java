package dev.rekall.domain;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.params.ParameterizedTest;
import org.junit.jupiter.params.provider.CsvSource;
import org.junit.jupiter.params.provider.ValueSource;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

/** The one rule the anchor grammar rests on: a label is a single space-free term. */
class SlugTest {

    @ParameterizedTest
    @CsvSource({
            "stvv, stvv",
            "STVV, stvv",
            "'  STVV Platform  ', stvv-platform",
            "Code Validator, code-validator",
            "code-validator, code-validator",
            "release_1.2, release_1.2",
            "'a  //  b', a-b",
            "'--edge--', edge",
            "caffè, caff",
    })
    @DisplayName("what a person types becomes a term an anchor can carry")
    void normalises(String raw, String expected) {
        // The last case is the one worth stating: an accent is dropped rather than transliterated,
        // because a label is an identifier and `task:caffè` is not something a terminal agrees on.
        assertThat(Slug.of(raw)).isEqualTo(expected);
    }

    @Test
    @DisplayName("the result is always a valid label, whatever went in")
    void alwaysMatchesThePattern() {
        for (String raw : new String[] {"STVV", "a / b", "..hidden..", "x--y", "1"}) {
            assertThat(Slug.of(raw)).matches(Slug.PATTERN);
        }
    }

    @ParameterizedTest
    @ValueSource(strings = {"", "   ", "///", "...", "---"})
    @DisplayName("a value with nothing usable in it is refused rather than silently emptied")
    void refusesWhatCannotBecomeALabel(String raw) {
        assertThatThrownBy(() -> Slug.of(raw)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void refusesNull() {
        assertThatThrownBy(() -> Slug.of(null)).isInstanceOf(IllegalArgumentException.class);
    }
}
