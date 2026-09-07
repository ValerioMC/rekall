package dev.rekall.domain;

/**
 * Where one step of a task has got to.
 *
 * <p>Four states on one line, and the line is what a session moves along. A step opens closed,
 * is picked up when the work on it starts, and is claimed by the session when that work is
 * finished. The last move, from {@link #CLAIMED} to {@link #DONE}, is the one a session cannot
 * make: it is a person saying they looked at the work and accept it.
 *
 * <p>The split between {@link #CLAIMED} and {@link #DONE} is the whole reason this is an enum
 * rather than the boolean it replaced. A session can now say "I am finished with this" over MCP
 * and drive the checklist forward on its own, and the navigator's progress count still means
 * "work a person accepted" because only a console tick reaches {@link #DONE}.
 */
public enum TaskStepState {

    /** Not started. The default a step is created in. */
    OPEN,

    /** A session is working on it now. Set over MCP, and what the console animates around. */
    RUNNING,

    /** A session has finished it and is waiting for the work to be accepted. Set over MCP. */
    CLAIMED,

    /** A person reviewed the work and accepted it. Reached only from the console. */
    DONE;

    /** Whether the work of this step is finished, whoever still has to sign off on it. */
    public boolean complete() {
        return this == CLAIMED || this == DONE;
    }

    /** Whether a session is on this step right now. */
    public boolean running() {
        return this == RUNNING;
    }

    /**
     * Whether a session may move a step into this state.
     *
     * <p>Everything except {@link #DONE}. A session can open a step, pick it up, claim it, and
     * put it back to open if it has to abandon it, but the last tick is the console's.
     */
    public boolean reachableBySession() {
        return this != DONE;
    }
}
