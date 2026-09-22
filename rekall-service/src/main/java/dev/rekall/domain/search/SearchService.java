package dev.rekall.domain.search;

import dev.rekall.domain.Document;
import dev.rekall.domain.Task;
import dev.rekall.domain.TaskStep;
import dev.rekall.domain.Wrapup;
import dev.rekall.domain.repository.DocumentRepository;
import dev.rekall.domain.repository.TaskRepository;
import dev.rekall.domain.repository.TaskStepRepository;
import dev.rekall.domain.repository.WrapupRepository;
import lombok.RequiredArgsConstructor;
import org.springframework.data.domain.PageRequest;
import org.springframework.data.domain.Pageable;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Locale;

/**
 * Finds a phrase in the text the console's filter does not reach: task descriptions, steps (title
 * and detail) and wrapups, plus notes so they turn up while browsing tasks. Titles and labels are
 * left to the filter, which matches them as you type without a round trip.
 *
 * <p>The phrase is matched whole and case-insensitively, as typed; {@code %} and {@code _} in it are
 * literal. Under {@link #TERM_MIN} characters there is no search. Each kind of text returns at
 * most {@link #PER_KIND} hits, newest first, and within those a hit on a title comes before a hit
 * in the body.
 */
@Service
@RequiredArgsConstructor
public class SearchService {

    static final int TERM_MIN = 3;

    static final int PER_KIND = 8;

    /** Characters kept either side of the match in an excerpt. */
    static final int EXCERPT_RADIUS = 60;

    private final TaskRepository tasks;
    private final TaskStepRepository steps;
    private final WrapupRepository wrapups;
    private final DocumentRepository documents;

    @Transactional(readOnly = true)
    public List<SearchHit> search(String rawTerm) {
        String term = rawTerm == null ? "" : rawTerm.strip().replaceAll("\\s+", " ");
        if (term.length() < TERM_MIN) {
            return List.of();
        }
        String escaped = escapeLike(term);
        Pageable page = PageRequest.of(0, PER_KIND);

        List<SearchHit> hits = new ArrayList<>();
        tasks.searchDescriptions(escaped, page).forEach(task -> hits.add(descriptionHit(task, term)));
        steps.search(escaped, page).forEach(step -> hits.add(stepHit(step, term)));
        wrapups.search(escaped, page).forEach(wrapup -> hits.add(wrapupHit(wrapup, term)));
        documents.searchText(escaped, page).forEach(document -> hits.add(noteHit(document, term)));
        return hits.stream()
                .sorted(Comparator.comparing((SearchHit hit) -> hit.kind().ordinal())
                        .thenComparing(hit -> !contains(hit.title(), term)))
                .toList();
    }

    private static SearchHit descriptionHit(Task task, String term) {
        return new SearchHit(SearchHit.Kind.DESCRIPTION, task.getId(), null, null, task.getTitle(), anchorOf(task),
                excerpt(task.getDescription(), term));
    }

    private static SearchHit stepHit(TaskStep step, String term) {
        String body = contains(step.getBodyMarkdown(), term) ? step.getBodyMarkdown() : step.getTitle();
        return new SearchHit(SearchHit.Kind.STEP, step.getTask().getId(), step.getId(), null, step.getTitle(),
                anchorOf(step.getTask()), excerpt(body, term));
    }

    private static SearchHit wrapupHit(Wrapup wrapup, String term) {
        Task task = wrapup.getTask();
        return new SearchHit(SearchHit.Kind.WRAPUP, task.getId(), null, null, task.getTitle(), anchorOf(task),
                excerpt(wrapup.getBodyMarkdown(), term));
    }

    private static SearchHit noteHit(Document document, String term) {
        Task first = document.getTasks().stream().findFirst().orElse(null);
        String body = contains(document.getBodyMarkdown(), term) ? document.getBodyMarkdown() : document.getTitle();
        return new SearchHit(SearchHit.Kind.NOTE, first == null ? null : first.getId(), null, document.getId(),
                document.getTitle(), first == null ? "" : anchorOf(first), excerpt(body, term));
    }

    private static String anchorOf(Task task) {
        return "project:%s task:%s".formatted(task.getProject().getLabel(), task.getLabel());
    }

    private static boolean contains(String text, String term) {
        return text != null && text.toLowerCase(Locale.ROOT).contains(term.toLowerCase(Locale.ROOT));
    }

    /** The match with {@link #EXCERPT_RADIUS} characters either side, whitespace collapsed, cut at words. */
    static String excerpt(String text, String term) {
        if (text == null) {
            return "";
        }
        String flat = text.replaceAll("\\s+", " ").strip();
        int at = flat.toLowerCase(Locale.ROOT).indexOf(term.toLowerCase(Locale.ROOT));
        if (at < 0) {
            return flat.length() <= EXCERPT_RADIUS * 2 ? flat : flat.substring(0, EXCERPT_RADIUS * 2).stripTrailing() + "…";
        }
        int start = Math.max(0, at - EXCERPT_RADIUS);
        int end = Math.min(flat.length(), at + term.length() + EXCERPT_RADIUS);
        if (start > 0) {
            int space = flat.indexOf(' ', start);
            start = space >= 0 && space < at ? space + 1 : start;
        }
        if (end < flat.length()) {
            int space = flat.lastIndexOf(' ', end);
            end = space > at + term.length() ? space : end;
        }
        return (start > 0 ? "…" : "") + flat.substring(start, end) + (end < flat.length() ? "…" : "");
    }

    static String escapeLike(String term) {
        return term.replace("\\", "\\\\").replace("%", "\\%").replace("_", "\\_");
    }
}
