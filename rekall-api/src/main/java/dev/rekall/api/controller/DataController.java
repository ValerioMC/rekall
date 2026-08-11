package dev.rekall.api.controller;

import dev.rekall.api.service.DataService;
import dev.rekall.engine.data.Operator;
import dev.rekall.engine.data.QueryFilter;
import dev.rekall.engine.data.RecordView;
import lombok.RequiredArgsConstructor;
import org.springframework.http.HttpStatus;
import org.springframework.web.bind.annotation.DeleteMapping;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.PutMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.ResponseStatus;
import org.springframework.web.bind.annotation.RestController;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.UUID;

/** Data browser endpoints, generic over whatever entities exist. */
@RestController
@RequestMapping("/api/data")
@RequiredArgsConstructor
public class DataController {

    private final DataService data;

    @GetMapping("/{entity}")
    public PageResponse list(
            @PathVariable String entity,
            @RequestParam(required = false) String search,
            @RequestParam(defaultValue = "50") int limit,
            @RequestParam(defaultValue = "0") int offset) {

        QueryFilter filter = new QueryFilter(searchConditions(entity, search), List.of(), limit, offset);
        List<RecordView> records = data.list(entity, filter, 1);
        return new PageResponse(records, data.count(entity, filter), limit, offset);
    }

    @GetMapping("/{entity}/{id}")
    public RecordView get(@PathVariable String entity, @PathVariable UUID id) {
        return data.get(entity, id);
    }

    @PostMapping("/{entity}")
    @ResponseStatus(HttpStatus.CREATED)
    public RecordView create(@PathVariable String entity, @RequestBody Map<String, Object> values) {
        return data.create(entity, values);
    }

    @PutMapping("/{entity}/{id}")
    public RecordView update(
            @PathVariable String entity, @PathVariable UUID id, @RequestBody Map<String, Object> values) {
        return data.update(entity, id, values);
    }

    @DeleteMapping("/{entity}/{id}")
    @ResponseStatus(HttpStatus.NO_CONTENT)
    public void delete(@PathVariable String entity, @PathVariable UUID id) {
        data.delete(entity, id);
    }

    /**
     * Free-text search in the browser matches on the display field only. Searching every text
     * column would need an OR, which the filter model deliberately does not have, and the
     * display field is what the user is looking at in the list anyway.
     */
    private List<QueryFilter.Condition> searchConditions(String entity, String search) {
        List<QueryFilter.Condition> conditions = new ArrayList<>();
        if (search == null || search.isBlank()) {
            return conditions;
        }
        data.requireEntity(entity)
                .displayField()
                .ifPresent(field -> conditions.add(
                        new QueryFilter.Condition(field.getColumnName(), Operator.ILIKE, "%" + search.trim() + "%")));
        return conditions;
    }

    public record PageResponse(List<RecordView> records, long total, int limit, int offset) {
    }
}
