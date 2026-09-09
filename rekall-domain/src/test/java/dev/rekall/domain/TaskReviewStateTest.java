package dev.rekall.domain;

import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

import static org.assertj.core.api.Assertions.assertThat;

/**
 * The task-scoped review line: the same four values a step walks, read at task
 * scope, and the two moments that have to keep saying what they say as a stepless
 * task moves along it.
 */
class TaskReviewStateTest {

    private Task newTask() {
        return new Task("in-app-claude", "In-app Claude execution");
    }

    @Test
    @DisplayName("a fresh task opens OPEN, review active, with nothing behind it")
    void startsOpen() {
        Task task = newTask();

        assertThat(task.getReviewState()).isEqualTo(TaskStepState.OPEN);
        assertThat(task.reviewActive()).isTrue();
        assertThat(task.getClaimedAt()).isNull();
        assertThat(task.getAcceptedAt()).isNull();
        assertThat(task.getReviewNote()).isNull();
    }

    @Test
    @DisplayName("running, then claimed, then accepted: each move stamps its own moment")
    void walksTheLine() {
        Task task = newTask();

        task.markReviewState(TaskStepState.RUNNING);
        assertThat(task.getClaimedAt()).isNull();
        assertThat(task.getAcceptedAt()).isNull();

        task.markReviewState(TaskStepState.CLAIMED);
        assertThat(task.getClaimedAt()).isNotNull();
        assertThat(task.getAcceptedAt()).isNull();

        task.markReviewState(TaskStepState.DONE);
        assertThat(task.getAcceptedAt()).isNotNull();
        assertThat(task.getClaimedAt())
                .as("the claim moment is kept when the work is later accepted")
                .isNotNull();
    }

    @Test
    @DisplayName("accepting straight from open still records that it must have been claimed")
    void acceptingFromOpenBackfillsClaimed() {
        Task task = newTask();

        task.markReviewState(TaskStepState.DONE);

        assertThat(task.getClaimedAt()).isNotNull();
        assertThat(task.getAcceptedAt()).isNotNull();
    }

    @Test
    @DisplayName("a send-back note is kept while OPEN and dropped the moment it is claimed again")
    void theSendBackNoteDoesNotOutliveOpen() {
        Task task = newTask();
        task.markReviewState(TaskStepState.OPEN);
        task.setReviewNote("the export column is still wrong");

        assertThat(task.getReviewNote()).isEqualTo("the export column is still wrong");

        task.markReviewState(TaskStepState.CLAIMED);
        assertThat(task.getReviewNote()).isNull();
    }

    @Test
    @DisplayName("sending back clears every moment: a reopened task has no history of being anything")
    void sendingBackClearsEverything() {
        Task task = newTask();
        task.markReviewState(TaskStepState.RUNNING);
        task.markReviewState(TaskStepState.CLAIMED);
        task.markReviewState(TaskStepState.DONE);

        task.markReviewState(TaskStepState.OPEN);

        assertThat(task.getReviewState()).isEqualTo(TaskStepState.OPEN);
        assertThat(task.getClaimedAt()).isNull();
        assertThat(task.getAcceptedAt()).isNull();
    }

    @Test
    @DisplayName("marking the state it is already in changes nothing")
    void sameStateIsANoOp() {
        Task task = newTask();
        task.markReviewState(TaskStepState.CLAIMED);
        var claimedAt = task.getClaimedAt();

        task.markReviewState(TaskStepState.CLAIMED);

        assertThat(task.getClaimedAt()).isEqualTo(claimedAt);
    }
}
