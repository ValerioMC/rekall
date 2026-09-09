package dev.rekall.domain;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

/**
 * The four-state line a step moves along, and the three timestamps that have to keep saying
 * what they say as it does.
 */
class TaskStepStateTest {

    private TaskStep newStep() {
        return new TaskStep(new Task("report-builder", "Report builder"), "Aggregate the rows", 0);
    }

    @Test
    @DisplayName("a step opens DRAFT, with nothing behind it")
    void startsDraft() {
        TaskStep step = newStep();

        assertThat(step.getState()).isEqualTo(TaskStepState.DRAFT);
        assertThat(step.isDone()).isFalse();
        assertThat(step.getRunningAt()).isNull();
        assertThat(step.getClaimedAt()).isNull();
        assertThat(step.getDoneAt()).isNull();
        assertThat(step.completedAt()).isNull();
    }

    @Test
    @DisplayName("a draft is not reachable by a session; every later state but DONE is")
    void draftIsOutOfASessionsReach() {
        assertThat(TaskStepState.DRAFT.reachableBySession()).isFalse();
        assertThat(TaskStepState.OPEN.reachableBySession()).isTrue();
        assertThat(TaskStepState.RUNNING.reachableBySession()).isTrue();
        assertThat(TaskStepState.CLAIMED.reachableBySession()).isTrue();
        assertThat(TaskStepState.DONE.reachableBySession()).isFalse();
    }

    @Test
    @DisplayName("promoting a draft to open leaves nothing behind it")
    void promotingADraftKeepsItClean() {
        TaskStep step = newStep();

        step.markState(TaskStepState.OPEN);

        assertThat(step.getState()).isEqualTo(TaskStepState.OPEN);
        assertThat(step.getRunningAt()).isNull();
        assertThat(step.getClaimedAt()).isNull();
        assertThat(step.getDoneAt()).isNull();
    }

    @Test
    @DisplayName("running, then claimed, then done: each move stamps its own moment")
    void walksTheLine() {
        TaskStep step = newStep();

        step.markState(TaskStepState.RUNNING);
        assertThat(step.getRunningAt()).isNotNull();
        assertThat(step.getClaimedAt()).isNull();
        assertThat(step.getDoneAt()).isNull();

        step.markState(TaskStepState.CLAIMED);
        assertThat(step.getClaimedAt()).isNotNull();
        assertThat(step.getDoneAt()).isNull();
        assertThat(step.completedAt())
                .as("the work was finished when it was claimed, not when it is later ticked")
                .isEqualTo(step.getClaimedAt());

        step.markState(TaskStepState.DONE);
        assertThat(step.isDone()).isTrue();
        assertThat(step.getDoneAt()).isNotNull();
        assertThat(step.completedAt())
                .as("still measured from the claim")
                .isEqualTo(step.getClaimedAt());
    }

    @Test
    @DisplayName("claiming straight from open still records that it must have been running")
    void claimingFromOpenBackfillsRunning() {
        TaskStep step = newStep();

        step.markState(TaskStepState.CLAIMED);

        assertThat(step.getRunningAt()).isNotNull();
        assertThat(step.getClaimedAt()).isNotNull();
    }

    @Test
    @DisplayName("reopening clears every moment: a reopened step has no history of being anything")
    void reopeningClearsEverything() {
        TaskStep step = newStep();
        step.markState(TaskStepState.RUNNING);
        step.markState(TaskStepState.CLAIMED);
        step.markState(TaskStepState.DONE);

        step.markState(TaskStepState.OPEN);

        assertThat(step.getState()).isEqualTo(TaskStepState.OPEN);
        assertThat(step.getRunningAt()).isNull();
        assertThat(step.getClaimedAt()).isNull();
        assertThat(step.getDoneAt()).isNull();
        assertThat(step.completedAt()).isNull();
    }

    @Test
    @DisplayName("marking a step the state it is already in changes nothing")
    void sameStateIsANoOp() {
        TaskStep step = newStep();
        step.markState(TaskStepState.RUNNING);
        var runningAt = step.getRunningAt();

        step.markState(TaskStepState.RUNNING);

        assertThat(step.getRunningAt()).isEqualTo(runningAt);
    }
}
