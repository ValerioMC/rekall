package dev.rekall.engine.plan;

import java.util.List;
import java.util.Set;
import java.util.UUID;

/**
 * An ordered, reviewable set of schema changes.
 *
 * <p>A plan is produced, shown, and only then applied. It is never applied as a side effect of
 * being computed, which is what lets the UI render the exact SQL before anything happens.
 *
 * @param droppedEntities physical names whose tables this plan removes. Carried explicitly
 *     because their documents live in {@code rekall_meta} behind a soft reference that the
 *     database cannot cascade, so the executor has to delete them by hand.
 */
public record DdlPlan(UUID planId, List<DdlStatement> statements, Set<String> droppedEntities) {

    public DdlPlan {
        statements = List.copyOf(statements);
        droppedEntities = Set.copyOf(droppedEntities);
    }

    public static DdlPlan empty() {
        return new DdlPlan(UUID.randomUUID(), List.of(), Set.of());
    }

    public List<DdlStatement> blocked() {
        return statements.stream().filter(s -> s.changeClass() == ChangeClass.BLOCKED).toList();
    }

    public List<DdlStatement> awaitingInput() {
        return statements.stream().filter(s -> s.changeClass() == ChangeClass.NEEDS_INPUT).toList();
    }

    /** Statements that would actually run, in order. */
    public List<DdlStatement> executable() {
        return statements.stream().filter(DdlStatement::isExecutable).toList();
    }

    public boolean isEmpty() {
        return statements.isEmpty();
    }

    /**
     * A plan can only be applied when nothing is refused and nothing is still waiting on the
     * user. {@code NEEDS_INPUT} steps that the user has answered are re-planned as
     * {@link ChangeClass#SAFE}, so their presence here means the answer is still missing.
     */
    public boolean isApplicable() {
        return !isEmpty() && blocked().isEmpty() && awaitingInput().isEmpty();
    }
}
