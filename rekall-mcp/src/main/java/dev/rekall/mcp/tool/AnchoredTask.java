package dev.rekall.mcp.tool;

import java.util.ArrayList;
import java.util.List;

/**
 * The one task a write tool is addressing, pulled out of a list of anchors.
 *
 * <p>A read tool can take a company or a project anchor and hand back everything under it. A
 * write cannot: a project names forty tasks and none of them is the answer. So anything that
 * does not resolve to a single task is refused here, with the form that would have worked.
 *
 * @param projectLabel the {@code project:} half, or null when only a task was given
 */
record AnchoredTask(String projectLabel, String taskLabel) {

    static AnchoredTask from(List<Anchor> anchors) {
        String project = null;
        String task = null;
        List<String> bare = new ArrayList<>();

        for (Anchor anchor : anchors) {
            if (anchor.is("task")) {
                if (task != null) {
                    throw new ToolFailure("Two task anchors were given. This write belongs to one task.");
                }
                task = anchor.value();
            } else if (anchor.is("project")) {
                project = anchor.value();
            } else if (!anchor.isQualified()) {
                bare.add(anchor.value());
            } else {
                throw new ToolFailure(
                        "`%s` cannot say which task to write to. Pass `project:<label> task:<label>`."
                                .formatted(anchor));
            }
        }

        // One bare term is the positional form of a task label, the same way `/rk report-builder`
        // reads. More than one is a guess, and this is the wrong place to guess.
        if (task == null && bare.size() == 1) {
            task = bare.getFirst();
        }
        if (task == null) {
            throw new ToolFailure(
                    "No task in those anchors. This write belongs to exactly one task: "
                            + "pass `project:<label> task:<label>`.");
        }
        return new AnchoredTask(project, task);
    }
}
