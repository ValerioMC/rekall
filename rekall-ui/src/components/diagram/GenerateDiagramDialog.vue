<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import AppDialog from '@/components/ui/AppDialog.vue'
import type { Project, Task } from '@/model/catalog'
import type { ProjectId, TaskId } from '@/model/branded'

/**
 * Asks for a diagram. It names only what to show: the session opened on the task finds the
 * code in the project folder by itself and writes the graph back through Rekall.
 */
const props = defineProps<{
  projects: readonly Project[]
  tasks: readonly Task[]
  initialTaskId: TaskId | null
  /** Opened from a task: the task is not asked for again, only what to show. */
  taskFixed?: boolean
  sending: boolean
}>()

const emit = defineEmits<{ cancel: []; generate: [projectId: ProjectId, taskId: TaskId, request: string] }>()

const field = ref<HTMLTextAreaElement | null>(null)

const initialTask = props.tasks.find((task) => task.id === props.initialTaskId) ?? null
const projectId = ref<string | null>(initialTask?.projectId ?? props.projects[0]?.id ?? null)
const taskId = ref<string | null>(initialTask?.id ?? null)
const request = ref('')

const project = computed(() => props.projects.find((candidate) => candidate.id === projectId.value) ?? null)
const projectTasks = computed(() =>
  props.tasks
    .filter((task) => task.projectId === projectId.value)
    .sort((a, b) => Number(a.status === 'DONE') - Number(b.status === 'DONE') || a.title.localeCompare(b.title))
)

watch(projectId, () => {
  if (!projectTasks.value.some((task) => task.id === taskId.value)) taskId.value = projectTasks.value[0]?.id ?? null
})
if (!taskId.value) taskId.value = projectTasks.value[0]?.id ?? null

const task = computed(() => projectTasks.value.find((candidate) => candidate.id === taskId.value) ?? null)
const missingFolder = computed(() => project.value !== null && !project.value.repoFolder)
const ready = computed(() => task.value !== null && !missingFolder.value && request.value.trim().length > 0 && !props.sending)

const preview = computed(() => {
  const anchors = task.value ? `project:${task.value.projectLabel} task:${task.value.label}` : 'project:… task:…'
  const said = request.value.trim().replace(/\s+/g, ' ').replace(/"/g, "'") || '…'
  return `/rk ${anchors} generate "${said}"`
})

const projectOptions = computed(() => props.projects.map((candidate) => ({ value: candidate.id, label: candidate.title })))
const taskOptions = computed(() => projectTasks.value.map((candidate) => ({ value: candidate.id, label: candidate.title })))

function submit(): void {
  if (!ready.value || !task.value) return
  emit('generate', task.value.projectId, task.value.id, request.value.trim())
}

function submitOnModEnter(event: KeyboardEvent): void {
  if (!(event.metaKey || event.ctrlKey) || event.key !== 'Enter') return
  event.preventDefault()
  submit()
}

onMounted(async () => {
  window.addEventListener('keydown', submitOnModEnter, true)
  await nextTick()
  field.value?.focus()
})

onUnmounted(() => window.removeEventListener('keydown', submitOnModEnter, true))
</script>

<template>
  <AppDialog
    label="Generate a diagram"
    panel-class="max-w-[560px]"
    panel-testid="generate-diagram-dialog"
    :dismissible="!sending"
    @dismiss="emit('cancel')"
  >
    <header class="flex items-center gap-3 border-b border-border px-5 py-4">
      <span class="min-w-0 flex-1">
        <span class="block text-title font-semibold tracking-[-0.01em] text-text">Generate a diagram</span>
        <span class="mt-0.5 block text-small leading-relaxed text-text-muted">
          Say what you want to understand. A session reads the code and draws what it does.
        </span>
      </span>
      <button class="focus-ring grid size-7 shrink-0 place-items-center rounded-control text-text-subtle transition-colors hover:bg-surface-raised hover:text-text" aria-label="Close" :disabled="sending" @click="emit('cancel')">
        <CloseGlyph />
      </button>
    </header>

    <div class="flex flex-col gap-4 px-5 py-4">
      <label class="block">
        <span class="eyebrow mb-1.5 block text-caption">What should it show?</span>
        <textarea
          ref="field"
          v-model="request"
          rows="4"
          class="field w-full resize-y rounded-control px-3 py-2.5 text-body leading-relaxed text-text"
          placeholder="How an order moves from checkout to shipped, including what happens when payment fails"
          data-testid="generate-request"
        />
      </label>

      <div v-if="taskFixed && task" class="flex min-w-0 items-center gap-2 rounded-control border border-border bg-canvas px-3 py-2" data-testid="generate-task">
        <span class="min-w-0 flex-1">
          <span class="block truncate text-body font-medium text-text">{{ task.title }}</span>
          <span class="block truncate text-caption text-text-subtle">{{ task.projectTitle }}</span>
        </span>
        <span class="anchor-chip shrink-0 px-1.5 py-px text-micro">{{ task.label }}</span>
      </div>

      <div v-else class="grid grid-cols-2 gap-3">
        <label class="block min-w-0">
          <span class="eyebrow mb-1.5 block text-caption">Project</span>
          <AppSelect v-model="projectId" :options="projectOptions" />
        </label>
        <label class="block min-w-0">
          <span class="eyebrow mb-1.5 block text-caption">Run on task</span>
          <AppSelect v-model="taskId" :options="taskOptions" placeholder="No task on this project" :disabled="taskOptions.length === 0" />
        </label>
      </div>

      <p v-if="missingFolder" class="rounded-control border border-warn/40 bg-warn-soft px-3 py-2 text-small leading-relaxed text-warn" role="alert">
        {{ project?.title }} has no folder yet. Set it on the project page: the session reads the code there.
      </p>

      <p class="anchor-chip flex items-center gap-2 truncate px-2.5 py-1.5 text-caption" data-testid="generate-preview">
        <span class="shrink-0 opacity-60">&gt;</span>
        <span class="min-w-0 flex-1 truncate">{{ preview }}</span>
      </p>
    </div>

    <footer class="flex items-center justify-between gap-2 border-t border-border bg-canvas px-5 py-3">
      <span class="text-caption text-text-subtle"><kbd>⌘</kbd><kbd>↵</kbd> to generate</span>
      <span class="flex gap-2">
        <AppButton variant="ghost" size="sm" :disabled="sending" @click="emit('cancel')">Cancel</AppButton>
        <AppButton variant="primary" size="sm" :disabled="!ready" :loading="sending" data-testid="generate-submit" @click="submit">Generate</AppButton>
      </span>
    </footer>
  </AppDialog>
</template>
