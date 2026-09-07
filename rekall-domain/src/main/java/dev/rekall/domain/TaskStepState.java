package dev.rekall.domain;

public enum TaskStepState {

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

    public boolean reachableBySession() {
        return this != DONE;
    }
}
