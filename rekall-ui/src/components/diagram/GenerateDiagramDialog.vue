<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import { useModalGate } from '@/composables/useModalGate'
import { trapTabKey } from '@/common/a11y/focus-trap'
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

const { open: openModal, close: closeModal } = useModalGate()
const panel = ref<HTMLElement | null>(null)
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

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    event.stopPropagation()
    if (!props.sending) emit('cancel')
    return
  }
  if (event.key === 'Enter' && (event.metaKey || event.ctrlKey)) {
    event.preventDefault()
    submit()
    return
  }
  if (panel.value) trapTabKey(panel.value, event)
}

onMounted(async () => {
  openModal()
  window.addEventListener('keydown', onKeydown, true)
  await nextTick()
  field.value?.focus()
})

onUnmounted(() => {
  closeModal()
  window.removeEventListener('keydown', onKeydown, true)
})
</script>

<template>
  <div class="fixed inset-0 z-(--z-modal) grid place-items-center bg-black/70 p-5 backdrop-blur-sm" @click.self="!sending && emit('cancel')">
    <div
      ref="panel"
      class="dialog-panel w-full max-w-[560px] overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface shadow-modal"
      role="dialog"
      aria-modal="true"
      aria-label="Generate a diagram"
      data-testid="generate-diagram-dialog"
    >
      <header class="flex items-center gap-3 border-b border-border px-5 py-4">
        <span class="min-w-0 flex-1">
          <span class="block text-[15px] font-semibold tracking-[-0.01em] text-text">Generate a diagram</span>
          <span class="mt-0.5 block text-[12.5px] leading-relaxed text-text-muted">
            Say what you want to understand. A session reads the code and draws what it does.
          </span>
        </span>
        <button class="focus-ring grid size-7 shrink-0 place-items-center rounded-md text-text-subtle transition-colors hover:bg-surface-raised hover:text-text" aria-label="Close" :disabled="sending" @click="emit('cancel')">
          <CloseGlyph />
        </button>
      </header>

      <div class="flex flex-col gap-4 px-5 py-4">
        <label class="block">
          <span class="eyebrow mb-1.5 block text-[11px]">What should it show?</span>
          <textarea
            ref="field"
            v-model="request"
            rows="4"
            class="field w-full resize-y rounded-[var(--radius-control)] px-3 py-2.5 text-[13.5px] leading-relaxed text-text"
            placeholder="How an order moves from checkout to shipped, including what happens when payment fails"
            data-testid="generate-request"
          />
        </label>

        <div v-if="taskFixed && task" class="flex min-w-0 items-center gap-2 rounded-[var(--radius-control)] border border-border bg-canvas px-3 py-2" data-testid="generate-task">
          <span class="min-w-0 flex-1">
            <span class="block truncate text-[13px] font-medium text-text">{{ task.title }}</span>
            <span class="block truncate text-[11.5px] text-text-subtle">{{ task.projectTitle }}</span>
          </span>
          <span class="anchor-chip shrink-0 px-1.5 py-px text-[10px]">{{ task.label }}</span>
        </div>

        <div v-else class="grid grid-cols-2 gap-3">
          <label class="block min-w-0">
            <span class="eyebrow mb-1.5 block text-[11px]">Project</span>
            <AppSelect v-model="projectId" :options="projectOptions" />
          </label>
          <label class="block min-w-0">
            <span class="eyebrow mb-1.5 block text-[11px]">Run on task</span>
            <AppSelect v-model="taskId" :options="taskOptions" placeholder="No task on this project" :disabled="taskOptions.length === 0" />
          </label>
        </div>

        <p v-if="missingFolder" class="rounded-[var(--radius-control)] border border-warn/40 bg-warn-soft px-3 py-2 text-[12px] leading-relaxed text-warn" role="alert">
          {{ project?.title }} has no folder yet. Set it on the project page: the session reads the code there.
        </p>

        <p class="anchor-chip flex items-center gap-2 truncate px-2.5 py-1.5 text-[11px]" data-testid="generate-preview">
          <span class="shrink-0 opacity-60">&gt;</span>
          <span class="min-w-0 flex-1 truncate">{{ preview }}</span>
        </p>
      </div>

      <footer class="flex items-center justify-between gap-2 border-t border-border bg-canvas px-5 py-3">
        <span class="text-[11px] text-text-subtle"><kbd>⌘</kbd><kbd>↵</kbd> to generate</span>
        <span class="flex gap-2">
          <AppButton variant="ghost" size="sm" :disabled="sending" @click="emit('cancel')">Cancel</AppButton>
          <AppButton variant="primary" size="sm" :disabled="!ready" :loading="sending" data-testid="generate-submit" @click="submit">Generate</AppButton>
        </span>
      </footer>
    </div>
  </div>
</template>
