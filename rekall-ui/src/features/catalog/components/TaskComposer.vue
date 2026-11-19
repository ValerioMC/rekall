<script setup lang="ts">
import { computed, ref } from 'vue'
import AppSplitButton, { type SplitButtonOption } from '@/components/ui/AppSplitButton.vue'
import PopoverMenu, { type PopoverMenuEntry } from '@/components/ui/PopoverMenu.vue'
import { useQuickTask } from '@/features/catalog/composables/useQuickTask'
import { QUICK_TASK_HELP, folderTail } from '@/features/catalog/lib/quick-task'
import SessionProviderMark from '@/features/claude/components/SessionProviderMark.vue'
import { useSessionProviderChoice } from '@/features/claude/composables/useSessionProviderChoice'
import { RUN_WITH_PROVIDER_LABEL, SESSION_PROVIDERS, type SessionProvider } from '@/features/claude/model/session-provider'
import type { Project } from '@/features/catalog/model/catalog'
import type { ProjectId } from '@/model/branded'

/**
 * The one way to start work: the text becomes a quick task and a session opens on it, accepted by
 * itself once the wrapup is written; "Plan a task" opens the full dialog with the text as its title.
 * Across projects, `choices` lets the reader pick which project it goes to.
 */
const props = defineProps<{ project: Project; choices: readonly Project[] }>()
const emit = defineEmits<{ plan: [title: string]; choose: [projectId: ProjectId] }>()

const MAX_ROWS = 6

type ComposerAction = 'plan' | SessionProvider

const PLAN_OPTION: SplitButtonOption<'plan'> = {
  id: 'plan', label: 'Plan a task…', hint: 'Title, description, steps, epic, tags and skills.'
}
const PROVIDER_OPTIONS: readonly SplitButtonOption<SessionProvider>[] = SESSION_PROVIDERS.map(
  (id) => ({ id, label: RUN_WITH_PROVIDER_LABEL[id] })
)

const projectId = computed(() => props.project.id)
const quick = useQuickTask(projectId)
const canChooseProvider = useSessionProviderChoice()

const text = ref('')
const isFocused = ref(false)
const isPickingProject = ref(false)
const hasText = computed(() => text.value.trim() !== '')
const reason = computed(() =>
  props.project.repoFolder ? null : "Set this project's folder on its page to open a session from it."
)
const canStart = computed(() => hasText.value && reason.value === null && !quick.busy.value)
const isExpanded = computed(() => isFocused.value || hasText.value)
const rows = computed(() => Math.min(MAX_ROWS, text.value.split('\n').length))
const menuOptions = computed<readonly SplitButtonOption<ComposerAction>[]>(() => [
  ...(canStart.value && canChooseProvider.value ? PROVIDER_OPTIONS : []),
  PLAN_OPTION
])
const projectEntries = computed<readonly PopoverMenuEntry[]>(() =>
  props.choices.map((choice) => ({ kind: 'item', id: choice.id, label: choice.title, current: choice.id === props.project.id }))
)

async function start(provider?: SessionProvider): Promise<void> {
  if (!canStart.value) return
  const sent = text.value
  if (await quick.start(sent, provider)) text.value = ''
}

function chooseFromMenu(action: ComposerAction): void {
  if (action === 'plan') emit('plan', text.value.trim())
  else void start(action)
}

function onEnter(event: KeyboardEvent): void {
  if (event.shiftKey || event.isComposing) return
  event.preventDefault()
  void start()
}

function chooseProject(id: string): void {
  emit('choose', id as ProjectId)
}
</script>

<template>
  <section class="field rounded-card" aria-label="New task" data-testid="quick-task-bar">
    <div class="flex items-start gap-2.5 py-1.5 pl-3.5 pr-1.5">
      <svg class="mt-2.5 size-3.5 shrink-0 text-text-subtle" viewBox="0 0 14 14" fill="none" aria-hidden="true">
        <path d="M7 2.5v9M2.5 7h9" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
      </svg>
      <textarea
        v-model="text"
        class="min-w-0 flex-1 resize-none bg-transparent py-1.5 font-sans text-body leading-5 text-text outline-none placeholder:text-text-subtle"
        :rows="rows"
        :aria-describedby="`quick-task-help-${project.id}`"
        aria-label="What should the session do?"
        placeholder="Describe a change, or a task to plan"
        data-testid="quick-task-input"
        @keydown.enter="onEnter"
        @focus="isFocused = true"
        @blur="isFocused = false"
      />
      <PopoverMenu
        v-if="choices.length > 0"
        v-model:open="isPickingProject"
        label="Project for this task"
        align="right"
        :entries="projectEntries"
        @pick="chooseProject"
      >
        <template #trigger="{ toggle }">
          <button
            type="button"
            class="focus-ring flex h-8 cursor-pointer items-center gap-1.5 rounded-control px-2 text-small text-text-muted transition-colors duration-(--motion-quick) hover:bg-surface-raised hover:text-text"
            aria-label="Project for this task"
            data-popover-trigger
            data-testid="composer-project"
            @click="toggle"
          >
            {{ project.title }}
            <svg class="size-3.5" viewBox="0 0 16 16" fill="none" aria-hidden="true">
              <path d="m4 6 4 4 4-4" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </button>
        </template>
      </PopoverMenu>
      <span
        v-else-if="project.repoFolder"
        class="flex h-8 max-w-72 items-center truncate font-mono text-caption text-text-subtle"
        :title="project.repoFolder"
        data-testid="composer-folder"
      >
        {{ folderTail(project.repoFolder) }}
      </span>
      <AppSplitButton
        :variant="canStart ? 'primary' : 'secondary'"
        menu-label="More ways to create"
        keeps-arrow
        :options="menuOptions"
        :disabled="!canStart"
        :loading="quick.busy.value"
        data-testid="quick-task-start"
        @click="start()"
        @choose="chooseFromMenu"
      >
        Start quick task
        <template #option-mark="{ id }">
          <SessionProviderMark v-if="id !== 'plan'" :provider="id" />
        </template>
      </AppSplitButton>
    </div>
    <p
      v-show="isExpanded || reason !== null"
      :id="`quick-task-help-${project.id}`"
      class="px-3.5 pb-2.5 pl-9 text-caption"
      :class="reason === null ? 'text-text-subtle' : 'text-warn'"
      data-testid="quick-task-help"
    >
      {{ reason ?? QUICK_TASK_HELP }}
    </p>
  </section>
</template>
