import { preferredModel } from '@/common/config/claude-launch'
import { codexPreferences, launchChoices } from '@/common/config/session-launch'
import type { SelectOption } from '@/components/ui/AppSelect.vue'
import { claudeModelOptions } from '@/features/claude/lib/claude-model-choices'
import {
  CLAUDE_EFFORT_CHOICES,
  claudeEffortChoiceLabel,
  claudeEffortLabel,
  claudeModelLabel
} from '@/features/claude/model/claude'
import { SESSION_PROVIDER_LABEL, sessionViewLabel, type SessionProvider } from '@/features/claude/model/session-provider'
import type { Terminal } from '@/features/claude/model/terminal'
import { codexEffortLabel, type CodexModel } from '@/features/codex/model/codex'
import type { QueueTaskRequest, RunQueueItem } from '@/features/queue/model/runQueue'
import type { TaskId } from '@/model/branded'

export interface QueueLaunchDraft {
  readonly provider: SessionProvider
  readonly model: string
  readonly effort: string
}

const ACCOUNT_DEFAULT: SelectOption = { value: 'default', label: 'Account default' }
const CLAUDE_EFFORTS = CLAUDE_EFFORT_CHOICES.map((value) => ({ value, label: claudeEffortChoiceLabel(value) }))

export function queueLaunchDefaults(provider: SessionProvider): QueueLaunchDraft {
  const { model, effort } = launchChoices(provider)
  return { provider, model, effort }
}

export function queueModelOptions(provider: SessionProvider, models: readonly CodexModel[]): readonly SelectOption[] {
  if (provider === 'CLAUDE') return claudeModelOptions(null, preferredModel())
  return [ACCOUNT_DEFAULT, ...models.map(({ model, displayName }) => ({ value: model, label: displayName }))]
}

export function queueEffortOptions(draft: QueueLaunchDraft, models: readonly CodexModel[]): readonly SelectOption[] {
  if (draft.provider === 'CLAUDE') return CLAUDE_EFFORTS
  const supported = models.find(({ model }) => model === draft.model)?.supportedReasoningEfforts ?? []
  return [ACCOUNT_DEFAULT, ...supported.map(({ reasoningEffort: value }) => ({ value, label: codexEffortLabel(value) }))]
}

export function queueLaunchRequest(taskId: TaskId, draft: QueueLaunchDraft): QueueTaskRequest {
  const options = {
    model: draft.model === 'default' ? null : draft.model,
    effort: draft.effort === 'default' ? null : draft.effort
  }
  if (draft.provider === 'CLAUDE') return { taskId, provider: 'CLAUDE', claudeOptions: options }
  return { taskId, provider: 'CODEX', codexOptions: { ...options, skipPermissions: codexPreferences().skipPermissions } }
}

export function queueSessionBlocker(session: Pick<Terminal, 'provider' | 'kind'> | null): string | null {
  if (!session) return null
  return `Close the live ${SESSION_PROVIDER_LABEL[session.provider]} ${sessionViewLabel(session.kind)} before queueing this task.`
}

export function queueLaunchLabels(item: RunQueueItem): { model: string; effort: string } {
  if (item.provider === 'CLAUDE') {
    if (!item.claudeOptions) return { model: 'Queue model', effort: 'Queue effort' }
    return {
      model: claudeModelLabel(item.claudeOptions.model) ?? ACCOUNT_DEFAULT.label,
      effort: claudeEffortLabel(item.claudeOptions.effort) ?? ACCOUNT_DEFAULT.label
    }
  }

  return {
    model: item.codexOptions?.model ?? ACCOUNT_DEFAULT.label,
    effort: item.codexOptions?.effort ? codexEffortLabel(item.codexOptions.effort) : ACCOUNT_DEFAULT.label
  }
}
