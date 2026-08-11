package dev.rekall.engine.execute;

import dev.rekall.engine.plan.DdlPlan;

/** A plan was submitted for execution while still refused or still waiting on the user. */
public class PlanNotApplicableException extends RuntimeException {

    public PlanNotApplicableException(DdlPlan plan) {
        super(buildMessage(plan));
    }

    private static String buildMessage(DdlPlan plan) {
        if (plan.isEmpty()) {
            return "There is nothing to apply: the schema already matches the model";
        }
        if (!plan.blocked().isEmpty()) {
            return "The plan contains %d refused change(s): %s"
                    .formatted(
                            plan.blocked().size(),
                            plan.blocked().stream()
                                    .map(statement -> statement.description() + " (" + statement.warning() + ")")
                                    .reduce((a, b) -> a + "; " + b)
                                    .orElse(""));
        }
        return "The plan is waiting on %d answer(s): %s"
                .formatted(
                        plan.awaitingInput().size(),
                        plan.awaitingInput().stream()
                                .map(statement -> statement.inputKey() + " (" + statement.warning() + ")")
                                .reduce((a, b) -> a + "; " + b)
                                .orElse(""));
    }
}
