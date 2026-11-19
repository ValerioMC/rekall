<script setup lang="ts">
import { computed, onMounted } from 'vue'
import ConnectionKindIcon from '@/features/connection/components/ConnectionKindIcon.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import { currentTaskManager, lacksTarget, taskManagerCandidates } from '@/features/project/lib/task-manager'
import type { Project } from '@/features/catalog/model/catalog'
import type { ProjectConnection } from '@/features/connection/model/connection'

/**
 * Where the project's tasks are written: one of its GitLab, GitHub or Jira bindings. A task
 * created here creates the issue there, and a task edited here edits it. A project with none chosen
 * keeps its tasks in Rekall alone.
 */
const props = defineProps<{ project: Project }>()

const store = useConnectionsStore()
const { run, isRunning } = useAsyncAction()

const bindings = computed(() => store.projectConnectionsOf(props.project.id))
const candidates = computed(() => taskManagerCandidates(bindings.value))
const chosen = computed(() => currentTaskManager(bindings.value))

onMounted(() => {
  if (!store.isProjectLoaded(props.project.id)) void run(() => store.loadProject(props.project.id))
})

function choose(binding: ProjectConnection | null): void {
  if (binding === null) {
    const released = chosen.value
    if (released) void write(released, false)
    return
  }
  void write(binding, true)
}

async function write(binding: ProjectConnection, taskManager: boolean): Promise<void> {
  await run(() =>
    store.updateBinding(props.project.id, binding.id, { label: binding.label, target: binding.target, taskManager })
  )
  await run(() => store.loadProject(props.project.id))
}
</script>

<template>
  <section aria-labelledby="project-task-manager-heading" data-testid="settings-task-manager">
    <h3 id="project-task-manager-heading" class="text-body font-semibold text-text">Task manager</h3>
    <p class="mb-3 mt-0.5 text-small text-text-subtle">
      The GitLab, GitHub or Jira project this project's tasks are written to. A new task creates its issue and a
      change to a task's title or description updates it. Only one connection is the task manager. With one chosen, a
      queued task works on a branch of its own.
    </p>

    <p v-if="candidates.length === 0" class="text-small text-text-muted" data-testid="task-manager-empty">
      This project uses no GitLab, GitHub or Jira connection yet. Bind one on its Connections page.
    </p>

    <fieldset v-else class="flex flex-col gap-1" :disabled="isRunning">
      <legend class="sr-only">Task manager</legend>
      <label class="flex min-h-(--spacing-row) cursor-pointer items-center gap-2 rounded-control px-2 hover:bg-surface-raised">
        <input
          type="radio"
          name="task-manager"
          :checked="chosen === null"
          data-testid="task-manager-none"
          @change="choose(null)"
        />
        <span class="text-body text-text">None</span>
        <span class="text-caption text-text-subtle">Tasks stay in Rekall</span>
      </label>
      <label
        v-for="binding in candidates"
        :key="binding.id"
        class="flex min-h-(--spacing-row) items-center gap-2 rounded-control px-2 hover:bg-surface-raised"
        :class="lacksTarget(binding) ? 'cursor-not-allowed opacity-70' : 'cursor-pointer'"
      >
        <input
          type="radio"
          name="task-manager"
          :checked="chosen?.id === binding.id"
          :disabled="lacksTarget(binding)"
          :data-testid="`task-manager-${binding.label}`"
          @change="choose(binding)"
        />
        <ConnectionKindIcon :kind="binding.kind" :size="14" />
        <span class="text-body text-text">{{ binding.title }}</span>
        <span class="font-mono text-caption text-anchor">{{ binding.label }}</span>
        <span v-if="binding.target" class="truncate font-mono text-caption text-text-subtle">{{ binding.target }}</span>
        <span v-else class="text-caption text-warn" role="status">Set its target on the Connections page first</span>
        <span v-if="!binding.credentialSetAt" class="text-caption text-warn" role="status">No credential stored</span>
      </label>
    </fieldset>
  </section>
</template>
