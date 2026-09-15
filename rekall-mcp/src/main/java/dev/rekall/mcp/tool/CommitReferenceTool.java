package dev.rekall.mcp.tool;

import dev.rekall.domain.commit.CommitReferenceService;
import dev.rekall.domain.commit.CommitReferenceView;
import dev.rekall.domain.context.AmbiguousAnchorException;
import dev.rekall.domain.context.UnknownAnchorException;
import dev.rekall.mcp.protocol.McpTool;
import dev.rekall.mcp.protocol.ToolSchema;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Component;
import tools.jackson.databind.JsonNode;

import java.util.Map;

@Component
@RequiredArgsConstructor
public class CommitReferenceTool implements McpTool {

    private final CommitReferenceService commitReferences;

    @Override
    public String name() {
        return "rekall_record_commit";
    }

    @Override
    public boolean writes() {
        return true;
    }

    @Override
    public String description() {
        return """
               Log the commit you just made against this task, or one of its steps. Call it right \
               after `git commit`, once, with nothing but the anchor: it reads the tip of the \
               project's own repo folder itself, so there is no hash or message to pass.

               `commit` is optional: the hash of an earlier commit, abbreviated or full, when the \
               one to log is not the tip any more. Leave it out for the commit you just made.

               Anchor the task the way `rekall_context` does: `project:vega task:report-builder`. \
               It has to name exactly one task.

               `step` is optional: the step's one-based position in the checklist (`"3"`) or its \
               exact title. Leave it out to log the commit against the task itself.

               The comment stored is the commit's own subject line, so write that commit message \
               as the one line you would want to see next to this task later. Logging the same \
               commit against the same task and step twice is a no-op, not a duplicate.
               """;
    }

    @Override
    public Map<String, Object> inputSchema() {
        return ToolSchema.object()
                .requiredString(
                        "anchors",
                        "The task to log this commit against, e.g. `project:vega task:report-builder`. "
                                + "Labels, never titles. It has to name exactly one task.")
                .optionalString(
                        "step",
                        "Which step, if any: its one-based position in the checklist (`\"3\"`), or its "
                                + "exact title. Omitted logs the commit against the task itself.")
                .optionalString(
                        "commit",
                        "The hash of the commit to log, abbreviated or full, when it is not the tip of "
                                + "the repo. Omitted logs the tip.")
                .build();
    }

    @Override
    public String execute(JsonNode arguments) {
        Arguments args = Arguments.of(arguments);
        AnchoredTask target = AnchoredTask.from(Anchor.parseAll(args.requiredString("anchors")));
        String step = args.optionalString("step");
        String commit = args.optionalString("commit");

        CommitReferenceView logged;
        try {
            logged = commitReferences.recordCommit(target.projectLabel(), target.taskLabel(), step, commit);
        } catch (UnknownAnchorException e) {
            throw new ToolFailure(e.getMessage());
        } catch (AmbiguousAnchorException e) {
            throw new ToolFailure(e.getMessage() + ". Qualify it with `project:<label>`.");
        } catch (IllegalArgumentException e) {
            throw new ToolFailure(e.getMessage());
        }

        String where = logged.stepTitle() == null ? "the task" : "\"%s\"".formatted(logged.stepTitle());
        return "Logged `%s` — %s — against %s.".formatted(
                logged.commitHash().substring(0, Math.min(7, logged.commitHash().length())),
                logged.comment(),
                where);
    }
}
