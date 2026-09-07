package dev.rekall.mcp.tool;

import java.util.ArrayList;
import java.util.List;

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
