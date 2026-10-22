<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppCheckbox from '@/components/ui/AppCheckbox.vue'
import AppDialog from '@/components/ui/AppDialog.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import ConnectionKindIcon from '@/features/connection/components/ConnectionKindIcon.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import { useConsoleStore } from '@/stores/console.store'
import type { JiraIssue, ProjectConnection } from '@/features/connection/model/connection'

/**
 * The open issues of the Jira project a binding names, to pick and import as the project's tasks.
 * An issue already imported is shown as such and cannot be picked again.
 */
const props = defineProps<{ binding: ProjectConnection }>()
const emit = defineEmits<{ close: [] }>()

const connections = useConnectionsStore()
const consoleStore = useConsoleStore()
const { run: load, isRunning: isLoading } = useAsyncAction()
const { run, isRunning } = useAsyncAction()
const issues = ref<readonly JiraIssue[] | null>(null)
const chosen = ref<ReadonlySet<string>>(new Set())

const heading = computed(() => `Import from ${props.binding.title}`)
const importable = computed(() => (issues.value ?? []).filter((issue) => issue.taskId === null))

function toggle(key: string, picked: boolean): void {
  const next = new Set(chosen.value)
  if (picked) next.add(key)
  else next.delete(key)
  chosen.value = next
}

async function loadIssues(): Promise<void> {
  issues.value = await load(() => connections.jiraIssues(props.binding.projectId, props.binding.connectionId))
}

async function importChosen(): Promise<void> {
  if (!chosen.value.size || isRunning.value) return
  const keys = [...chosen.value]
  const created = await run(
    () => consoleStore.importJiraIssues(props.binding.projectId, props.binding.connectionId, keys),
    `Imported ${keys.length} Jira ${keys.length === 1 ? 'issue' : 'issues'} as tasks`
  )
  if (created) emit('close')
}

onMounted(loadIssues)
</script>

<template>
  <AppDialog
    :label="heading"
    panel-class="max-w-[640px]"
    panel-testid="jira-import-dialog"
    @dismiss="emit('close')"
  >
    <header class="flex items-center gap-3 border-b border-border px-6 py-4">
      <ConnectionKindIcon kind="JIRA" :size="18" />
      <span class="min-w-0 flex-1 text-title font-semibold tracking-[-0.01em] text-text">{{ heading }}</span>
      <span class="font-mono text-caption text-text-subtle">{{ binding.target }}</span>
      <button
        class="focus-ring grid size-7 place-items-center rounded-control text-text-subtle transition-colors hover:bg-surface-raised hover:text-text"
        aria-label="Close"
        @click="emit('close')"
      >
        <CloseGlyph />
      </button>
    </header>

    <div class="max-h-[60vh] overflow-y-auto px-6 py-4">
      <p v-if="isLoading" class="text-small text-text-subtle" data-testid="jira-issues-loading">
        Asking Jira for the open issues…
      </p>
      <p v-else-if="issues === null" class="text-small text-text-subtle" data-testid="jira-issues-failed">
        Jira could not be read.
        <button class="focus-ring rounded-key text-accent hover:underline" @click="loadIssues">Try again</button>
      </p>
      <p v-else-if="!issues.length" class="text-small text-text-subtle" data-testid="jira-issues-empty">
        {{ binding.target }} has no open issue.
      </p>
      <ul v-else class="divide-y divide-border">
        <li
          v-for="issue in issues"
          :key="issue.key"
          class="flex items-center gap-3 py-2"
          data-testid="jira-issue"
        >
          <span v-if="issue.taskId" class="shrink-0 font-mono text-small text-text-subtle">{{ issue.key }}</span>
          <AppCheckbox
            v-else
            class="shrink-0 font-mono"
            :model-value="chosen.has(issue.key)"
            :label="issue.key"
            :data-testid="`pick-${issue.key}`"
            @update:model-value="toggle(issue.key, $event)"
          />
          <span class="min-w-0 flex-1 truncate text-small text-text" :title="issue.summary">{{ issue.summary }}</span>
          <AppBadge v-if="issue.taskId" tone="safe" dot data-testid="jira-issue-imported">Imported</AppBadge>
          <span v-if="issue.issueType" class="shrink-0 text-caption text-text-subtle">{{ issue.issueType }}</span>
          <AppBadge v-if="issue.status" class="shrink-0">{{ issue.status }}</AppBadge>
          <a
            :href="issue.url"
            target="_blank"
            rel="noopener noreferrer"
            class="focus-ring shrink-0 rounded-key text-caption text-text-subtle hover:text-accent"
            :aria-label="`Open ${issue.key} in Jira`"
            >&#8599;</a
          >
        </li>
      </ul>
      <p v-if="issues?.length" class="mt-3 text-caption text-text-subtle">
        The {{ issues.length }} most recently updated open issues. Each becomes a task labelled with its key,
        its description carried over with a link back to Jira.
      </p>
    </div>

    <footer class="flex items-center justify-end gap-2 border-t border-border bg-canvas px-6 py-3.5">
      <span class="flex-1 text-caption text-text-subtle">{{ chosen.size }} of {{ importable.length }} chosen</span>
      <AppButton size="sm" variant="ghost" @click="emit('close')">Cancel</AppButton>
      <AppButton
        size="sm"
        variant="primary"
        :disabled="!chosen.size"
        :loading="isRunning"
        data-testid="import-chosen-issues"
        @click="importChosen"
      >
        Import as tasks
      </AppButton>
    </footer>
  </AppDialog>
</template>
