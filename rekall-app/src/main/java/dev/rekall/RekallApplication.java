package dev.rekall;

import dev.rekall.bootstrap.DatabaseEntry;
import dev.rekall.bootstrap.DatabaseRegistry;
import dev.rekall.bootstrap.SettingsController;
import dev.rekall.domain.Company;
import dev.rekall.domain.Document;
import dev.rekall.domain.Project;
import dev.rekall.domain.Task;
import dev.rekall.domain.TimeEntry;
import dev.rekall.domain.Wrapup;
import dev.rekall.claude.ClaudeApiDtos;
import dev.rekall.claude.ClaudeUsageView;
import dev.rekall.domain.claude.ClaudeMessageView;
import dev.rekall.domain.claude.ClaudeSessionView;
import dev.rekall.domain.step.StepStreamEvent;
import dev.rekall.domain.step.TaskStepView;
import org.springframework.aot.hint.annotation.RegisterReflectionForBinding;
import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.context.ConfigurableApplicationContext;

@SpringBootApplication
@RegisterReflectionForBinding({
        DatabaseRegistry.class, DatabaseEntry.class, DatabaseEntry[].class,
        Company.class, Project.class, Task.class, TimeEntry.class, Wrapup.class, Document.class,
        StepStreamEvent.class, TaskStepView.class, TaskStepView[].class,
        ClaudeSessionView.class, ClaudeSessionView[].class,
        ClaudeMessageView.class, ClaudeMessageView[].class,
        ClaudeUsageView.class, ClaudeUsageView.Limit.class, ClaudeUsageView.Limit[].class,
        ClaudeApiDtos.StartSessionRequest.class, ClaudeApiDtos.PromptRequest.class,
        SettingsController.DatabaseView.class, SettingsController.DatabaseView[].class,
        SettingsController.StatusResponse.class,
        SettingsController.AddRequest.class, SettingsController.AddResponse.class,
        SettingsController.RenameRequest.class, SettingsController.CheckResponse.class
})
public class RekallApplication {

    public static void main(String[] args) {
        ConfigurableApplicationContext context = SpringApplication.run(RekallApplication.class, args);
        ApplicationRestarter.register(context, args);
    }
}
