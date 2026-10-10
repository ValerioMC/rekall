<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppDialog from '@/components/ui/AppDialog.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import type { DiagramDraft } from '@/model/diagram'
import type { Project, Task } from '@/model/catalog'
import type { ProjectId, TaskId } from '@/model/branded'

/**
 * Stores Mermaid written elsewhere as a one-sheet diagram of a project. Its boxes open no code:
 * only a session links them, when it generates a diagram.
 */
const props = defineProps<{
  projects: readonly Project[]
  tasks: readonly Task[]
  initialProjectId: ProjectId | null
  sending: boolean
  refusal: string | null
}>()

const emit = defineEmits<{ cancel: []; import: [draft: DiagramDraft] }>()

const NO_TASK = ''

const field = ref<HTMLInputElement | null>(null)
const projectId = ref<string | null>(props.initialProjectId ?? props.projects[0]?.id ?? null)
const taskId = ref<string | null>(NO_TASK)
const title = ref('')
const mermaid = ref('')

const projectTasks = computed(() => props.tasks.filter((task) => task.projectId === projectId.value))
watch(projectId, () => (taskId.value = NO_TASK))

const projectOptions = computed(() => props.projects.map((project) => ({ value: project.id, label: project.title })))
const taskOptions = computed(() => [{ value: NO_TASK, label: 'None' }, ...projectTasks.value.map((task) => ({ value: task.id, label: task.title }))])
const ready = computed(() => projectId.value !== null && title.value.trim().length > 0 && mermaid.value.trim().length > 0 && !props.sending)

function submit(): void {
  if (!ready.value || !projectId.value) return
  emit('import', {
    projectId: projectId.value as ProjectId,
    taskId: taskId.value ? (taskId.value as TaskId) : null,
    title: title.value.trim(),
    question: '',
    sheets: [{ title: null, mermaid: mermaid.value }]
  })
}

onMounted(async () => {
  await nextTick()
  field.value?.focus()
})
</script>

<template>
  <AppDialog label="Import Mermaid" panel-class="max-w-[640px]" panel-testid="import-diagram-dialog" :dismissible="!sending" @dismiss="emit('cancel')">
    <header class="flex items-center gap-3 border-b border-border px-5 py-4">
      <span class="min-w-0 flex-1">
        <span class="block text-title font-semibold tracking-[-0.01em] text-text">Import Mermaid</span>
        <span class="mt-0.5 block text-small leading-relaxed text-text-muted">Paste one diagram. It is drawn like a generated one, without links to the code.</span>
      </span>
      <button class="focus-ring grid size-7 shrink-0 place-items-center rounded-control text-text-subtle transition-colors hover:bg-surface-raised hover:text-text" aria-label="Close" :disabled="sending" @click="emit('cancel')">
        <CloseGlyph />
      </button>
    </header>

    <div class="flex flex-col gap-4 px-5 py-4">
      <label class="block">
        <span class="eyebrow mb-1.5 block text-caption">Title</span>
        <input ref="field" v-model="title" class="field h-9 w-full rounded-control px-3 text-body text-text" placeholder="Order checkout" data-testid="import-title" />
      </label>
      <div class="grid grid-cols-2 gap-3">
        <label class="block min-w-0">
          <span class="eyebrow mb-1.5 block text-caption">Project</span>
          <AppSelect v-model="projectId" :options="projectOptions" />
        </label>
        <label class="block min-w-0">
          <span class="eyebrow mb-1.5 block text-caption">Task <span class="font-normal normal-case tracking-normal text-text-subtle">optional</span></span>
          <AppSelect v-model="taskId" :options="taskOptions" />
        </label>
      </div>
      <label class="block">
        <span class="eyebrow mb-1.5 block text-caption">Mermaid</span>
        <textarea
          v-model="mermaid"
          rows="10"
          spellcheck="false"
          class="field w-full resize-y rounded-control px-3 py-2.5 font-mono text-caption leading-relaxed text-text"
          placeholder="sequenceDiagram&#10;  participant A&#10;  A->>B: hello"
          data-testid="import-mermaid"
        />
      </label>
      <p v-if="refusal" class="whitespace-pre-line rounded-control border border-danger/40 bg-danger-soft px-3 py-2 font-mono text-caption leading-relaxed text-danger" role="alert">{{ refusal }}</p>
    </div>

    <footer class="flex justify-end gap-2 border-t border-border bg-canvas px-5 py-3">
      <AppButton variant="ghost" size="sm" :disabled="sending" @click="emit('cancel')">Cancel</AppButton>
      <AppButton variant="primary" size="sm" :disabled="!ready" :loading="sending" data-testid="import-submit" @click="submit">Import</AppButton>
    </footer>
  </AppDialog>
</template>
