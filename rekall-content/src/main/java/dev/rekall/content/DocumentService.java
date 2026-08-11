package dev.rekall.content;

import dev.rekall.meta.domain.Document;
import dev.rekall.meta.repository.DocumentRepository;
import org.springframework.jdbc.core.JdbcTemplate;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.List;
import java.util.UUID;

/** Markdown documents attached to records of generated entities. */
@Service
public class DocumentService {

    private final DocumentRepository documents;
    private final JdbcTemplate jdbc;

    public DocumentService(DocumentRepository documents, JdbcTemplate jdbc) {
        this.documents = documents;
        this.jdbc = jdbc;
    }

    @Transactional(readOnly = true)
    public List<Document> forRecord(String entityName, UUID recordId) {
        return documents.findByEntityNameAndRecordIdOrderByPositionAsc(entityName, recordId);
    }

    @Transactional(readOnly = true)
    public Document get(UUID id) {
        return documents.findById(id).orElseThrow(() -> new DocumentNotFoundException(id));
    }

    @Transactional
    public Document create(String entityName, UUID recordId, String title, String kind, String bodyMarkdown) {
        Document document = new Document(entityName, recordId, title, kind, bodyMarkdown);
        document.setPosition((int) documents.countByEntityNameAndRecordId(entityName, recordId));
        return documents.save(document);
    }

    @Transactional
    public Document update(UUID id, String title, String kind, String bodyMarkdown) {
        Document document = get(id);
        document.setTitle(title);
        document.setKind(kind);
        document.setBodyMarkdown(bodyMarkdown);
        return documents.save(document);
    }

    @Transactional
    public void delete(UUID id) {
        documents.deleteById(id);
    }

    @Transactional
    public int deleteForRecord(String entityName, UUID recordId) {
        return documents.deleteByEntityNameAndRecordId(entityName, recordId);
    }

    /**
     * Full-text search over titles and bodies.
     *
     * <p>{@code websearch_to_tsquery} rather than {@code plainto_tsquery} so quoted phrases and
     * {@code or} behave the way anyone typing into a search box expects. The {@code simple}
     * configuration matches the generated column: the content mixes Italian and English, and a
     * language-specific stemmer would help one at the cost of the other.
     *
     * <p>Known limit: Postgres' parser keeps URLs and paths as single tokens, so a document
     * containing {@code /api/v1/pipelines} is not found by searching {@code pipelines} alone.
     * Searching the whole path, or a host or identifier, works. That trade-off is kept because
     * these notes are searched for cluster names and endpoints far more often than for a word
     * buried inside a path.
     */
    @Transactional(readOnly = true)
    public List<DocumentMatch> search(String query, List<String> entityNames, int limit) {
        if (query == null || query.isBlank()) {
            return List.of();
        }
        boolean filterByEntity = entityNames != null && !entityNames.isEmpty();
        String sql =
                """
                SELECT id, entity_name, record_id, title, kind,
                       ts_headline('simple', body_markdown, websearch_to_tsquery('simple', ?),
                                   'MaxFragments=2, MaxWords=40, MinWords=15') AS excerpt,
                       ts_rank(search_vector, websearch_to_tsquery('simple', ?)) AS rank
                FROM rekall_meta.document
                WHERE search_vector @@ websearch_to_tsquery('simple', ?)
                %s
                ORDER BY rank DESC
                LIMIT ?
                """
                        .formatted(filterByEntity ? "AND entity_name = ANY (?)" : "");

        Object[] arguments = filterByEntity
                ? new Object[] {query, query, query, entityNames.toArray(String[]::new), cappedLimit(limit)}
                : new Object[] {query, query, query, cappedLimit(limit)};

        return jdbc.query(
                sql,
                (rs, rowNum) -> new DocumentMatch(
                        rs.getObject("id", UUID.class),
                        rs.getString("entity_name"),
                        rs.getObject("record_id", UUID.class),
                        rs.getString("title"),
                        rs.getString("kind"),
                        rs.getString("excerpt"),
                        rs.getDouble("rank")),
                arguments);
    }

    private int cappedLimit(int limit) {
        return limit <= 0 ? 20 : Math.min(limit, 100);
    }
}
