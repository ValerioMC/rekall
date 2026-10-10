<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppDialog from '@/components/ui/AppDialog.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import KindPicker from './KindPicker.vue'
import type { GenerationRequest } from '@/stores/diagram.store'
import type { DiagramKind } from '@/model/diagram'
import type { Project, Task } from '@/model/catalog'
import type { ProjectId, TaskId } from '@/model/branded'

/**
 * Asks for a diagram: of which project, optionally for which of its tasks, as which types, and
 * what it should make clear. The session finds the code in the project folder by itself.
 */
const props = defineProps<{
  projects: readonly Project[]
  tasks: readonly Task[]
  initialProjectId: ProjectId | null
  initialTaskId: TaskId | null
  sending: boolean
}>()

const emit = defineEmits<{ cancel: []; generate: [request: GenerationRequest] }>()

/** The task select's value for "no task": the diagram then belongs to the project alone. */
const NO_TASK = ''

const field = ref<HTMLTextAreaElement | null>(null)
const projectId = ref<string | null>(props.initialProjectId ?? props.projects[0]?.id ?? null)
const taskId = ref<string | null>(props.initialTaskId ?? NO_TASK)
const kinds = ref<DiagramKind[]>(['sequence'])
const request = ref('')

const project = computed(() => props.projects.find((candidate) => candidate.id === projectId.value) ?? null)
const projectTasks = computed(() =>
  props.tasks
    .filter((task) => task.projectId === projectId.value)
    .sort((a, b) => Number(a.status === 'DONE') - Number(b.status === 'DONE') || a.title.localeCompare(b.title))
)
const task = computed(() => projectTasks.value.find((candidate) => candidate.id === taskId.value) ?? null)

watch(projectId, () => {
  if (!projectTasks.value.some((candidate) => candidate.id === taskId.value)) taskId.value = NO_TASK
})

const missingFolder = computed(() => project.value !== null && !project.value.repoFolder)
const ready = computed(() => project.value !== null && !missingFolder.value && kinds.value.length > 0 && request.value.trim().length > 0 && !props.sending)

const preview = computed(() => {
  const anchors = [project.value ? `project:${project.value.label}` : 'project:…', task.value ? `task:${task.value.label}` : null].filter(Boolean).join(' ')
  const said = request.value.trim().replace(/\s+/g, ' ').replace(/"/g, "'") || '…'
  return `/rk ${anchors} generate kinds:${kinds.value.join(',') || '…'} "${said}"`
})

const projectOptions = computed(() => props.projects.map((candidate) => ({ value: candidate.id, label: candidate.title })))
const taskOptions = computed(() => [
  { value: NO_TASK, label: 'None: the project alone' },
  ...projectTasks.value.map((candidate) => ({ value: candidate.id, label: candidate.title }))
])

function submit(): void {
  const chosen = project.value
  if (!ready.value || !chosen) return
  emit('generate', { projectId: chosen.id, taskId: task.value?.id ?? null, request: request.value.trim(), kinds: kinds.value })
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
  <AppDialog label="Generate a diagram" panel-class="max-w-[640px]" panel-testid="generate-diagram-dialog" :dismissible="!sending" @dismiss="emit('cancel')">
    <header class="flex items-center gap-3 border-b border-border px-5 py-4">
      <span class="min-w-0 flex-1">
        <span class="block text-title font-semibold tracking-[-0.01em] text-text">Generate a diagram</span>
        <span class="mt-0.5 block text-small leading-relaxed text-text-muted">A session reads the project's code and draws your answer as Mermaid.</span>
      </span>
      <button class="focus-ring grid size-7 shrink-0 place-items-center rounded-control text-text-subtle transition-colors hover:bg-surface-raised hover:text-text" aria-label="Close" :disabled="sending" @click="emit('cancel')">
        <CloseGlyph />
      </button>
    </header>

    <div class="flex flex-col gap-4 px-5 py-4">
      <label class="block">
        <span class="eyebrow mb-1.5 block text-caption">What should it make clear?</span>
        <textarea
          ref="field"
          v-model="request"
          rows="3"
          class="field w-full resize-y rounded-control px-3 py-2.5 text-body leading-relaxed text-text"
          placeholder="How an order moves from checkout to shipped, including what happens when payment fails"
          data-testid="generate-request"
        />
      </label>

      <fieldset>
        <legend class="eyebrow mb-1.5 flex items-baseline gap-2 text-caption">
          Draw it as
          <span class="font-normal normal-case tracking-normal text-text-subtle">one sheet per type</span>
        </legend>
        <KindPicker v-model="kinds" />
        <p v-if="kinds.length === 0" class="mt-1.5 text-caption text-warn" role="alert">Pick at least one type.</p>
      </fieldset>

      <div class="grid grid-cols-2 gap-3">
        <label class="block min-w-0">
          <span class="eyebrow mb-1.5 block text-caption">Project</span>
          <AppSelect v-model="projectId" :options="projectOptions" />
        </label>
        <label class="block min-w-0">
          <span class="eyebrow mb-1.5 block text-caption">For task <span class="font-normal normal-case tracking-normal text-text-subtle">optional</span></span>
          <AppSelect v-model="taskId" :options="taskOptions" />
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
