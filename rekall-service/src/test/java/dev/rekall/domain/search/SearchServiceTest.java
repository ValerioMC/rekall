package dev.rekall.domain.search;

import dev.rekall.domain.repository.DocumentRepository;
import dev.rekall.domain.repository.TaskRepository;
import dev.rekall.domain.repository.TaskStepRepository;
import dev.rekall.domain.repository.WrapupRepository;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verifyNoInteractions;

class SearchServiceTest {

    private final TaskRepository tasks = mock(TaskRepository.class);
    private final TaskStepRepository steps = mock(TaskStepRepository.class);
    private final WrapupRepository wrapups = mock(WrapupRepository.class);
    private final DocumentRepository documents = mock(DocumentRepository.class);
    private final SearchService service = new SearchService(tasks, steps, wrapups, documents);

    @Test
    @DisplayName("under three characters there is no search, and the database is not asked")
    void aShortTermSearchesNothing() {
        assertThat(service.search(" ab ")).isEmpty();
        assertThat(service.search(null)).isEmpty();
        verifyNoInteractions(tasks, steps, wrapups, documents);
    }

    @Test
    @DisplayName("the LIKE wildcards in a term are matched literally")
    void wildcardsAreEscaped() {
        assertThat(SearchService.escapeLike("100%_done\\")).isEqualTo("100\\%\\_done\\\\");
    }

    @Test
    @DisplayName("an excerpt is the match with words either side, on one line, cut at a word")
    void anExcerptSurroundsTheMatch() {
        String text = "word ".repeat(30) + "the\nsettlement   batch runs nightly " + "tail ".repeat(30);

        String excerpt = SearchService.excerpt(text, "Settlement batch");

        assertThat(excerpt).startsWith("…word").endsWith("…").contains("the settlement batch runs nightly");
        assertThat(excerpt).doesNotContain("\n").doesNotContain("  ");
        assertThat(excerpt.length()).isLessThanOrEqualTo(2 * SearchService.EXCERPT_RADIUS + "settlement batch".length() + 2);
    }

    @Test
    @DisplayName("a short text is returned whole, with no ellipsis")
    void aShortTextIsWhole() {
        assertThat(SearchService.excerpt("Wire the export", "export")).isEqualTo("Wire the export");
    }
}
