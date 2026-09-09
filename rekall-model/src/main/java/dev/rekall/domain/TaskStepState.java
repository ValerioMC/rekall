package dev.rekall.domain;

public enum TaskStepState {

    DRAFT,

    OPEN,

    RUNNING,

    CLAIMED,

    DONE;

    public boolean complete() {
        return this == CLAIMED || this == DONE;
    }

    public boolean running() {
        return this == RUNNING;
    }

    public boolean draft() {
        return this == DRAFT;
    }

    public boolean reachableBySession() {
        return this != DRAFT && this != DONE;
    }
}
