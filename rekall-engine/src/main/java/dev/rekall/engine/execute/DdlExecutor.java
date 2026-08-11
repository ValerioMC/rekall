package dev.rekall.engine.execute;

import dev.rekall.engine.plan.DdlPlan;
import dev.rekall.engine.plan.DdlStatement;
import dev.rekall.engine.schema.SchemaRegistry;
import dev.rekall.meta.domain.MetaTable;
import dev.rekall.meta.domain.MetaTableStatus;
import dev.rekall.meta.repository.DocumentRepository;
import dev.rekall.meta.repository.MetaTableRepository;
import org.jooq.DSLContext;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Component;
import org.springframework.transaction.annotation.Transactional;

import java.util.ArrayList;
import java.util.List;

/**
 * Applies a plan, all of it or none of it.
 *
 * <p>PostgreSQL has transactional DDL, which is the reason this design is safe at all: the
 * whole plan runs inside one transaction, so a statement failing halfway leaves no partially
 * migrated schema behind. It is also why PostgreSQL is a hard requirement rather than a
 * preference.
 */
@Component
public class DdlExecutor {

    private static final Logger log = LoggerFactory.getLogger(DdlExecutor.class);

    private final DSLContext dsl;
    private final DdlLogWriter logWriter;
    private final MetaTableRepository metaTables;
    private final DocumentRepository documents;
    private final SchemaRegistry schemaRegistry;

    public DdlExecutor(
            DSLContext dsl,
            DdlLogWriter logWriter,
            MetaTableRepository metaTables,
            DocumentRepository documents,
            SchemaRegistry schemaRegistry) {
        this.dsl = dsl;
        this.logWriter = logWriter;
        this.metaTables = metaTables;
        this.documents = documents;
        this.schemaRegistry = schemaRegistry;
    }

    /**
     * @throws PlanNotApplicableException if the plan is refused or still waiting on the user
     */
    @Transactional
    public DdlApplyResult apply(DdlPlan plan) {
        if (!plan.isApplicable()) {
            throw new PlanNotApplicableException(plan);
        }

        List<String> executed = new ArrayList<>();
        for (DdlStatement statement : plan.executable()) {
            try {
                dsl.execute(statement.sql());
                executed.add(statement.sql());
            } catch (RuntimeException e) {
                // Logged on its own transaction so the record outlives the rollback below.
                logWriter.recordFailure(plan.planId(), executed, statement.sql(), e.getMessage());
                log.error(
                        "DDL plan {} failed at statement {} of {}: {}",
                        plan.planId(),
                        executed.size() + 1,
                        plan.executable().size(),
                        statement.sql(),
                        e);
                throw e;
            }
        }

        int documentsDeleted = deleteDocumentsOfDroppedEntities(plan);
        markApplied();
        logWriter.recordSuccess(plan.planId(), executed);
        schemaRegistry.invalidate();

        log.info(
                "DDL plan {} applied: {} statement(s), {} orphaned document(s) removed",
                plan.planId(),
                executed.size(),
                documentsDeleted);

        return new DdlApplyResult(plan.planId(), executed, documentsDeleted);
    }

    /**
     * The database cannot do this. {@code document} points into {@code rekall_data} through a
     * soft reference, because those tables do not exist when the migration that creates
     * {@code document} runs, so dropping an entity has to take its documents with it here.
     */
    private int deleteDocumentsOfDroppedEntities(DdlPlan plan) {
        int deleted = 0;
        for (String entityName : plan.droppedEntities()) {
            deleted += documents.deleteByEntityName(entityName);
        }
        return deleted;
    }

    private void markApplied() {
        List<MetaTable> tables = metaTables.findAllWithFields();
        tables.forEach(table -> table.setStatus(MetaTableStatus.APPLIED));
        metaTables.saveAll(tables);
    }
}
