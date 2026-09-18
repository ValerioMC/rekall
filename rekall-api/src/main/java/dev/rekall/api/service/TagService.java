package dev.rekall.api.service;

import dev.rekall.api.dto.ApiDtos.TagRequest;
import dev.rekall.api.dto.ApiDtos.TagResponse;
import dev.rekall.common.ConflictException;
import dev.rekall.common.NotFoundException;
import dev.rekall.domain.Tag;
import dev.rekall.domain.repository.TagRepository;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.List;
import java.util.UUID;

@Service
@RequiredArgsConstructor
public class TagService {

    private final TagRepository tags;

    @Transactional(readOnly = true)
    public List<TagResponse> list() {
        return tags.findAllByOrderByNameAsc().stream().map(TagResponse::of).toList();
    }

    @Transactional
    public TagResponse create(TagRequest request) {
        requireNameFree(request.name(), null);
        Tag tag = new Tag(request.name().trim(), request.icon(), request.color());
        return TagResponse.of(tags.saveAndFlush(tag));
    }

    @Transactional
    public TagResponse update(UUID id, TagRequest request) {
        Tag tag = require(id);
        requireNameFree(request.name(), id);
        tag.setName(request.name().trim());
        tag.setIcon(request.icon());
        tag.setColor(request.color());
        return TagResponse.of(tags.saveAndFlush(tag));
    }

    @Transactional
    public void delete(UUID id) {
        tags.delete(require(id));
    }

    private void requireNameFree(String name, UUID exceptId) {
        boolean taken = exceptId == null
                ? tags.existsByNameIgnoreCase(name.trim())
                : tags.existsByNameIgnoreCaseAndIdNot(name.trim(), exceptId);
        if (taken) {
            throw new ConflictException("A tag named \"%s\" already exists".formatted(name.trim()));
        }
    }

    private Tag require(UUID id) {
        return tags.findById(id).orElseThrow(() -> new NotFoundException("Tag", id));
    }
}
