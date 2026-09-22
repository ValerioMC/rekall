package dev.rekall.api.controller;

import dev.rekall.domain.search.SearchHit;
import dev.rekall.domain.search.SearchService;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RequestParam;
import org.springframework.web.bind.annotation.RestController;

import java.util.List;

@RestController
@RequestMapping("/api/search")
@RequiredArgsConstructor
public class SearchController {

    private final SearchService search;

    @GetMapping
    public List<SearchHit> search(@RequestParam(name = "q", defaultValue = "") String term) {
        return search.search(term);
    }
}
