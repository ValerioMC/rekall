package dev.rekall.api.controller;

import dev.rekall.engine.execute.DdlApplyResult;
import dev.rekall.engine.execute.DdlExecutor;
import dev.rekall.engine.plan.ChangeClass;
import dev.rekall.engine.plan.DdlPhase;
import dev.rekall.engine.plan.DdlPlan;
import dev.rekall.engine.plan.DdlPlanner;
import dev.rekall.engine.plan.PlanOptions;
import lombok.RequiredArgsConstructor;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.UUID;

/**
 * Plan and apply, as two separate calls.
 *
 * <p>{@code GET /plan} has no side effect by design: the UI renders the exact SQL, the user
 * reads it, and only then does {@code POST /apply} run anything.
 */
@RestController
@RequestMapping("/api/meta")
@RequiredArgsConstructor
public class PlanController {

    private final DdlPlanner planner;
    private final DdlExecutor executor;

    @GetMapping("/plan")
    public PlanResponse plan() {
        return PlanResponse.from(planner.plan(PlanOptions.empty()));
    }

    /** Re-plans with the user's answers, so the preview and the execution are the same plan. */
    @PostMapping("/plan")
    public PlanResponse planWithOptions(@RequestBody ApplyRequest request) {
        return PlanResponse.from(planner.plan(request.toOptions()));
    }

    @PostMapping("/apply")
    public ApplyResponse apply(@RequestBody ApplyRequest request) {
        DdlPlan plan = planner.plan(request.toOptions());
        DdlApplyResult result = executor.apply(plan);
        return new ApplyResponse(result.planId(), result.statementCount(), result.documentsDeleted(), result.statements());
    }

    public record ApplyRequest(Map<String, String> backfillDefaults, Set<String> confirmedDrops) {

        PlanOptions toOptions() {
            return new PlanOptions(backfillDefaults, confirmedDrops);
        }
    }

    public record PlanResponse(
            UUID planId, boolean applicable, int blockedCount, int awaitingInputCount, List<StatementResponse> statements) {

        static PlanResponse from(DdlPlan plan) {
            return new PlanResponse(
                    plan.planId(),
                    plan.isApplicable(),
                    plan.blocked().size(),
                    plan.awaitingInput().size(),
                    plan.statements().stream().map(StatementResponse::from).toList());
        }
    }

    public record StatementResponse(
            DdlPhase phase, ChangeClass changeClass, String sql, String description, String warning, String inputKey) {

        static StatementResponse from(dev.rekall.engine.plan.DdlStatement statement) {
            return new StatementResponse(
                    statement.phase(),
                    statement.changeClass(),
                    statement.sql(),
                    statement.description(),
                    statement.warning(),
                    statement.inputKey());
        }
    }

    public record ApplyResponse(UUID planId, int statementCount, int documentsDeleted, List<String> statements) {
    }
}
