package dev.rekall.api.controller;

import dev.rekall.api.dto.MetaDtos;
import dev.rekall.api.service.MetaModelService;
import jakarta.validation.Valid;
import lombok.RequiredArgsConstructor;
import org.springframework.http.HttpStatus;
import org.springframework.web.bind.annotation.DeleteMapping;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.PutMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.ResponseStatus;
import org.springframework.web.bind.annotation.RestController;

import java.util.List;
import java.util.UUID;

/** Schema designer endpoints. Editing here never touches the physical schema. */
@RestController
@RequestMapping("/api/meta")
@RequiredArgsConstructor
public class MetaController {

    private final MetaModelService metaModel;

    @GetMapping("/tables")
    public List<MetaDtos.TableResponse> listTables() {
        return metaModel.listTables().stream().map(MetaDtos.TableResponse::from).toList();
    }

    @GetMapping("/tables/{id}")
    public MetaDtos.TableResponse getTable(@PathVariable UUID id) {
        return MetaDtos.TableResponse.from(metaModel.getTable(id));
    }

    @PostMapping("/tables")
    @ResponseStatus(HttpStatus.CREATED)
    public MetaDtos.TableResponse createTable(@Valid @RequestBody MetaDtos.CreateTableRequest request) {
        return MetaDtos.TableResponse.from(metaModel.createTable(request));
    }

    @PutMapping("/tables/{id}")
    public MetaDtos.TableResponse updateTable(
            @PathVariable UUID id, @Valid @RequestBody MetaDtos.UpdateTableRequest request) {
        return MetaDtos.TableResponse.from(metaModel.updateTable(id, request));
    }

    @DeleteMapping("/tables/{id}")
    @ResponseStatus(HttpStatus.NO_CONTENT)
    public void deleteTable(@PathVariable UUID id) {
        metaModel.deleteTable(id);
    }

    @PostMapping("/tables/{tableId}/fields")
    @ResponseStatus(HttpStatus.CREATED)
    public MetaDtos.FieldResponse addField(
            @PathVariable UUID tableId, @Valid @RequestBody MetaDtos.CreateFieldRequest request) {
        return MetaDtos.FieldResponse.from(metaModel.addField(tableId, request));
    }

    @PutMapping("/fields/{fieldId}")
    public MetaDtos.FieldResponse updateField(
            @PathVariable UUID fieldId, @Valid @RequestBody MetaDtos.UpdateFieldRequest request) {
        return MetaDtos.FieldResponse.from(metaModel.updateField(fieldId, request));
    }

    @DeleteMapping("/fields/{fieldId}")
    @ResponseStatus(HttpStatus.NO_CONTENT)
    public void deleteField(@PathVariable UUID fieldId) {
        metaModel.deleteField(fieldId);
    }

    @GetMapping("/relations")
    public List<MetaDtos.RelationResponse> listRelations() {
        return metaModel.listRelations().stream().map(MetaDtos.RelationResponse::from).toList();
    }

    @PostMapping("/relations")
    @ResponseStatus(HttpStatus.CREATED)
    public MetaDtos.RelationResponse createRelation(@Valid @RequestBody MetaDtos.CreateRelationRequest request) {
        return MetaDtos.RelationResponse.from(metaModel.createRelation(request));
    }

    @DeleteMapping("/relations/{id}")
    @ResponseStatus(HttpStatus.NO_CONTENT)
    public void deleteRelation(@PathVariable UUID id) {
        metaModel.deleteRelation(id);
    }
}
