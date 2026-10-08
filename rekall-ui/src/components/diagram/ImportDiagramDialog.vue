<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppSelect from '@/components/ui/AppSelect.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import AppDialog from '@/components/ui/AppDialog.vue'
import { SemanticGraphSchema } from '@/api/schemas/diagram.schema'
import type { DiagramDraft } from '@/model/diagram'
import type { Project, Task } from '@/model/catalog'
import type { ProjectId, TaskId } from '@/model/branded'

/**
 * Stores a Semantic Graph written elsewhere: pasted or dropped as JSON. The shape is checked
 * here as you type; the rules (ids, relations, containment) are the server's, and every rule
 * it refuses is listed.
 */
const props = defineProps<{
  projects: readonly Project[]
  tasks: readonly Task[]
  initialProjectId: ProjectId | null
  sending: boolean
  refusal: string | null
}>()

const emit = defineEmits<{ cancel: []; import: [draft: DiagramDraft] }>()

const titleField = ref<HTMLInputElement | null>(null)
const picker = ref<HTMLInputElement | null>(null)

const projectId = ref<string | null>(props.initialProjectId ?? props.projects[0]?.id ?? null)
const taskId = ref<string>('')
const title = ref('')
const question = ref('')
const json = ref('')
const dragging = ref(false)

const taskOptions = computed(() => [
  { value: '', label: 'None' },
  ...props.tasks.filter((task) => task.projectId === projectId.value).map((task) => ({ value: task.id, label: task.title }))
])
const projectOptions = computed(() => props.projects.map((project) => ({ value: project.id, label: project.title })))

watch(projectId, () => (taskId.value = ''))

const parsed = computed<{ graph: unknown; problem: string | null }>(() => {
  if (!json.value.trim()) return { graph: null, problem: null }
  let value: unknown
  try {
    value = JSON.parse(json.value)
  } catch (caught) {
    return { graph: null, problem: caught instanceof Error ? caught.message : 'Not JSON' }
  }
  const shape = SemanticGraphSchema.safeParse(value)
  if (!shape.success) {
    const issue = shape.error.issues[0]
    return { graph: null, problem: issue ? `${issue.path.join('.') || 'document'}: ${issue.message}` : 'Not a Semantic Graph' }
  }
  return { graph: value, problem: null }
})

const summary = computed(() => {
  const shape = SemanticGraphSchema.safeParse(parsed.value.graph)
  return shape.success ? `${shape.data.nodes.length} elements · ${shape.data.edges.length} relations` : null
})

const refusalLines = computed(() => (props.refusal ?? '').split('\n').filter((line) => line.trim().length > 0))
const ready = computed(() => projectId.value !== null && title.value.trim().length > 0 && parsed.value.graph !== null && !props.sending)

function submit(): void {
  if (!ready.value || !projectId.value) return
  emit('import', {
    projectId: projectId.value as ProjectId,
    taskId: taskId.value ? (taskId.value as TaskId) : null,
    title: title.value.trim(),
    question: question.value.trim(),
    graph: parsed.value.graph
  })
}

async function readFile(file: File | undefined): Promise<void> {
  if (!file) return
  json.value = await file.text()
  if (!title.value) title.value = file.name.replace(/\.json$/i, '').replace(/[-_]+/g, ' ')
}

function onDrop(event: DragEvent): void {
  dragging.value = false
  void readFile(event.dataTransfer?.files[0])
}

onMounted(async () => {
  await nextTick()
  titleField.value?.focus()
})
</script>

<template>
  <AppDialog
    label="Import a diagram"
    panel-class="flex flex-col max-h-[90vh] max-w-[620px]"
    panel-testid="import-diagram-dialog"
    :dismissible="!sending"
    @dismiss="emit('cancel')"
  >
    <header class="flex items-center gap-3 border-b border-border px-5 py-4">
      <span class="min-w-0 flex-1">
        <span class="block text-title font-semibold tracking-[-0.01em] text-text">Import a diagram</span>
        <span class="mt-0.5 block text-small leading-relaxed text-text-muted">A Semantic Graph document, format <code>rekall.semantic-graph</code> version 1.</span>
      </span>
      <button class="focus-ring grid size-7 shrink-0 place-items-center rounded-control text-text-subtle transition-colors hover:bg-surface-raised hover:text-text" aria-label="Close" :disabled="sending" @click="emit('cancel')">
        <CloseGlyph />
      </button>
    </header>

    <div class="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto px-5 py-4">
      <label class="block">
        <span class="eyebrow mb-1.5 block text-caption">Title</span>
        <input ref="titleField" v-model="title" type="text" class="field h-9 w-full rounded-control px-3 text-body text-text" data-testid="import-title" />
      </label>
      <label class="block">
        <span class="eyebrow mb-1.5 block text-caption">The question it answers</span>
        <input v-model="question" type="text" class="field h-9 w-full rounded-control px-3 text-body text-text" placeholder="Optional" />
      </label>
      <div class="grid grid-cols-2 gap-3">
        <label class="block min-w-0">
          <span class="eyebrow mb-1.5 block text-caption">Project</span>
          <AppSelect v-model="projectId" :options="projectOptions" />
        </label>
        <label class="block min-w-0">
          <span class="eyebrow mb-1.5 block text-caption">From task</span>
          <AppSelect v-model="taskId" :options="taskOptions" />
        </label>
      </div>

      <div>
        <div class="mb-1.5 flex items-center justify-between">
          <span class="eyebrow text-caption">Graph</span>
          <button class="focus-ring rounded-key px-1 text-caption text-text-subtle hover:text-text" @click="picker?.click()">Choose a file…</button>
          <input ref="picker" type="file" accept=".json,application/json" class="hidden" @change="readFile(($event.target as HTMLInputElement).files?.[0])" />
        </div>
        <textarea
          v-model="json"
          rows="10"
          spellcheck="false"
          class="field w-full resize-y rounded-control p-3 font-mono text-caption leading-relaxed text-text"
          :class="dragging ? 'border-accent' : ''"
          placeholder='Paste the JSON, or drop a .json file here: { "format": "rekall.semantic-graph", "version": 1, "nodes": [...], "edges": [...] }'
          :aria-invalid="parsed.problem !== null"
          data-testid="import-json"
          @dragover.prevent="dragging = true"
          @dragleave="dragging = false"
          @drop.prevent="onDrop"
        />
        <p v-if="parsed.problem" class="mt-1.5 font-mono text-caption text-danger" role="alert">{{ parsed.problem }}</p>
        <p v-else-if="summary" class="mt-1.5 font-mono text-caption text-text-subtle">{{ summary }}</p>
      </div>

      <div v-if="refusalLines.length > 0" class="rounded-control border border-danger/50 bg-danger-soft px-3 py-2.5" role="alert" data-testid="import-refusal">
        <p class="text-small font-medium text-danger">{{ refusalLines[0] }}</p>
        <ul v-if="refusalLines.length > 1" class="mt-1.5 flex flex-col gap-0.5">
          <li v-for="line in refusalLines.slice(1)" :key="line" class="font-mono text-caption text-text-muted">{{ line }}</li>
        </ul>
      </div>
    </div>

    <footer class="flex justify-end gap-2 border-t border-border bg-canvas px-5 py-3">
      <AppButton variant="ghost" size="sm" :disabled="sending" @click="emit('cancel')">Cancel</AppButton>
      <AppButton variant="primary" size="sm" :disabled="!ready" :loading="sending" data-testid="import-submit" @click="submit">Import</AppButton>
    </footer>
  </AppDialog>
</template>
