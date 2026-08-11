package dev.rekall.engine.plan;

import java.util.Map;
import java.util.Optional;
import java.util.Set;

/**
 * The answers the user gave to the {@link ChangeClass#NEEDS_INPUT} steps of a previous plan.
 *
 * <p>Planning is pure and repeatable: the same meta-model and the same options always produce
 * the same plan. That is what makes the preview the user approves and the plan that actually
 * runs the same thing.
 *
 * @param backfillDefaults literal to write into existing rows, keyed by {@code table.column}
 * @param confirmedDrops keys the user explicitly agreed to destroy, {@code table} or
 *     {@code table.column}
 */
public record PlanOptions(Map<String, String> backfillDefaults, Set<String> confirmedDrops) {

    public PlanOptions {
        backfillDefaults = backfillDefaults == null ? Map.of() : Map.copyOf(backfillDefaults);
        confirmedDrops = confirmedDrops == null ? Set.of() : Set.copyOf(confirmedDrops);
    }

    public static PlanOptions empty() {
        return new PlanOptions(Map.of(), Set.of());
    }

    public Optional<String> backfillFor(String key) {
        return Optional.ofNullable(backfillDefaults.get(key));
    }

    public boolean isDropConfirmed(String key) {
        return confirmedDrops.contains(key);
    }
}
