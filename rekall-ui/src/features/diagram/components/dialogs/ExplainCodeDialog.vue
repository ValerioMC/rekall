<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppDialog from '@/components/ui/AppDialog.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import type { GenerationRequest } from '@/features/diagram/store/diagram.store'
import type { Project, Task } from '@/features/catalog/model/catalog'
import type { CompanyId, ProjectId, TaskId } from '@/model/branded'

/**
 * Asks for a walkthrough: which company's project, optionally for which task, and which
 * functionality to follow from its entry point. A session writes it as sequences the console
 * plays step by step, with each call's code, input and output.
 */
const props = defineProps<{
  projects: readonly Project[]
  tasks: readonly Task[]
  initialProjectId: ProjectId | null
  initialTaskId: TaskId | null
  sending: boolean
}>()

const emit = defineEmits<{ cancel: []; explain: [request: GenerationRequest] }>()

/** The task select's value for "no task": the walkthrough then belongs to the project alone. */
const NO_TASK = ''

const field = ref<HTMLTextAreaElement | null>(null)
const initialProject = props.projects.find((candidate) => candidate.id === props.initialProjectId) ?? props.projects[0] ?? null
const companyId = ref<string | null>(initialProject?.companyId ?? null)
const projectId = ref<string | null>(initialProject?.id ?? null)
const taskId = ref<string | null>(props.initialTaskId ?? NO_TASK)
const request = ref('')

const companies = computed(() => {
  const byId = new Map<CompanyId, string>(props.projects.map((candidate) => [candidate.companyId, candidate.companyName]))
  return [...byId].map(([id, name]) => ({ value: id, label: name })).sort((a, b) => a.label.localeCompare(b.label))
})
const companyProjects = computed(() => props.projects.filter((candidate) => candidate.companyId === companyId.value))
const project = computed(() => companyProjects.value.find((candidate) => candidate.id === projectId.value) ?? null)
const projectTasks = computed(() =>
  props.tasks
    .filter((task) => task.projectId === projectId.value)
    .sort((a, b) => Number(a.status === 'DONE') - Number(b.status === 'DONE') || a.title.localeCompare(b.title))
)
const task = computed(() => projectTasks.value.find((candidate) => candidate.id === taskId.value) ?? null)

watch(companyId, () => {
  if (!companyProjects.value.some((candidate) => candidate.id === projectId.value)) projectId.value = companyProjects.value[0]?.id ?? null
})
watch(projectId, () => {
  if (!projectTasks.value.some((candidate) => candidate.id === taskId.value)) taskId.value = NO_TASK
})

const missingFolder = computed(() => project.value !== null && !project.value.repoFolder)
const ready = computed(() => project.value !== null && !missingFolder.value && request.value.trim().length > 0 && !props.sending)

const preview = computed(() => {
  const anchors = [project.value ? `project:${project.value.label}` : 'project:…', task.value ? `task:${task.value.label}` : null].filter(Boolean).join(' ')
  const said = request.value.trim().replace(/\s+/g, ' ').replace(/"/g, "'") || '…'
  return `/rk ${anchors} explain "${said}"`
})

const projectOptions = computed(() => companyProjects.value.map((candidate) => ({ value: candidate.id, label: candidate.title })))
const taskOptions = computed(() => [
  { value: NO_TASK, label: 'None: the project alone' },
  ...projectTasks.value.map((candidate) => ({ value: candidate.id, label: candidate.title }))
])

function submit(): void {
  const chosen = project.value
  if (!ready.value || !chosen) return
  emit('explain', { projectId: chosen.id, taskId: task.value?.id ?? null, request: request.value.trim(), kinds: ['sequence'], walkthrough: true })
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
  <AppDialog label="Explain the code" panel-class="max-w-[640px]" panel-testid="explain-code-dialog" :dismissible="!sending" @dismiss="emit('cancel')">
    <header class="flex items-center gap-3 border-b border-border px-5 py-4">
      <span class="min-w-0 flex-1">
        <span class="block text-title font-semibold tracking-[-0.01em] text-text">Explain the code</span>
        <span class="mt-0.5 block text-small leading-relaxed text-text-muted">A session follows the functionality from its entry point and writes a walkthrough you can play step by step.</span>
      </span>
      <button class="focus-ring grid size-7 shrink-0 place-items-center rounded-control text-text-subtle transition-colors hover:bg-surface-raised hover:text-text" aria-label="Close" :disabled="sending" @click="emit('cancel')">
        <CloseGlyph />
      </button>
    </header>

    <div class="flex flex-col gap-4 px-5 py-4">
      <div class="grid grid-cols-2 gap-3">
        <label class="block min-w-0">
          <span class="eyebrow mb-1.5 block text-caption">Company</span>
          <AppSelect v-model="companyId" :options="companies" />
        </label>
        <label class="block min-w-0">
          <span class="eyebrow mb-1.5 block text-caption">Project</span>
          <AppSelect v-model="projectId" :options="projectOptions" />
        </label>
      </div>

      <label class="block">
        <span class="eyebrow mb-1.5 block text-caption">Which functionality?</span>
        <textarea
          ref="field"
          v-model="request"
          rows="4"
          class="field w-full resize-y rounded-control px-3 py-2.5 text-body leading-relaxed text-text"
          placeholder="What happens when a client places an order, from the POST /orders handler down to the database"
          data-testid="explain-request"
        />
        <span class="mt-1.5 block text-caption leading-relaxed text-text-subtle">Name where it starts if you know it: a controller, a command, a scheduled job.</span>
      </label>

      <label class="block min-w-0">
        <span class="eyebrow mb-1.5 block text-caption">For task <span class="font-normal normal-case tracking-normal text-text-subtle">optional</span></span>
        <AppSelect v-model="taskId" :options="taskOptions" />
      </label>

      <p v-if="missingFolder" class="rounded-control border border-warn/40 bg-warn-soft px-3 py-2 text-small leading-relaxed text-warn" role="alert">
        {{ project?.title }} has no folder yet. Set it on the project page: the session reads the code there.
      </p>

      <p class="anchor-chip flex items-center gap-2 truncate px-2.5 py-1.5 text-caption" data-testid="explain-preview">
        <span class="shrink-0 opacity-60">&gt;</span>
        <span class="min-w-0 flex-1 truncate">{{ preview }}</span>
      </p>
    </div>

    <footer class="flex items-center justify-between gap-2 border-t border-border bg-canvas px-5 py-3">
      <span class="text-caption text-text-subtle"><kbd>⌘</kbd><kbd>↵</kbd> to explain</span>
      <span class="flex gap-2">
        <AppButton variant="ghost" size="sm" :disabled="sending" @click="emit('cancel')">Cancel</AppButton>
        <AppButton variant="primary" size="sm" :disabled="!ready" :loading="sending" data-testid="explain-submit" @click="submit">Explain</AppButton>
      </span>
    </footer>
  </AppDialog>
</template>
