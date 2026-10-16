<script setup lang="ts">
import { computed } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import DiagramCard from './DiagramCard.vue'
import GenerationCard from './GenerationCard.vue'
import { identityHue } from '@/common/identity'
import type { PendingGeneration } from '@/features/diagram/store/diagram.store'
import type { DiagramSummary } from '@/features/diagram/model/diagram'
import type { Project, Task } from '@/features/catalog/model/catalog'
import type { DiagramId } from '@/model/branded'

/**
 * Every diagram of the projects in scope, one section per project, newest first; above them,
 * what sessions are drawing now. Opening a card gives the whole screen to that diagram.
 */
const props = defineProps<{
  summaries: readonly DiagramSummary[]
  projects: readonly Project[]
  tasks: readonly Task[]
  pending: readonly PendingGeneration[]
  endedTerminals: ReadonlySet<string>
  arrivedId: DiagramId | null
  loaded: boolean
}>()

const emit = defineEmits<{ open: [id: DiagramId]; generate: []; import: []; watch: [generation: PendingGeneration]; dismiss: [key: number] }>()

const projectIds = computed(() => new Set(props.projects.map((project) => project.id)))

const sections = computed(() =>
  props.projects
    .map((project) => ({ project, diagrams: props.summaries.filter((summary) => summary.projectId === project.id) }))
    .filter((section) => section.diagrams.length > 0)
)

const drawing = computed(() => props.pending.filter((generation) => projectIds.value.has(generation.projectId)))

function taskAnchorOf(taskId: string | null): string | null {
  const task = taskId ? props.tasks.find((candidate) => candidate.id === taskId) : null
  return task ? `task:${task.label}` : null
}

function anchorsOf(generation: PendingGeneration): string {
  const project = props.projects.find((candidate) => candidate.id === generation.projectId)
  const task = taskAnchorOf(generation.taskId)
  return [project ? `project:${project.label}` : null, task].filter((part) => part !== null).join(' ')
}
</script>

<template>
  <div class="min-h-0 flex-1 overflow-y-auto" data-testid="diagram-gallery">
    <div class="mx-auto flex max-w-[1240px] flex-col gap-8 px-6 pb-16 pt-6">
      <section v-if="drawing.length > 0" aria-labelledby="gallery-drawing">
        <h2 id="gallery-drawing" class="section-label mb-3">Being drawn</h2>
        <div class="grid grid-cols-[repeat(auto-fill,minmax(280px,1fr))] gap-3">
          <GenerationCard
            v-for="generation in drawing"
            :key="generation.key"
            :generation="generation"
            :anchors="anchorsOf(generation)"
            :ended="endedTerminals.has(generation.terminalId)"
            @watch="emit('watch', generation)"
            @dismiss="emit('dismiss', generation.key)"
          />
        </div>
      </section>

      <section v-for="{ project, diagrams } in sections" :key="project.id" :aria-labelledby="`gallery-${project.id}`">
        <h2 :id="`gallery-${project.id}`" class="mb-3 flex items-center gap-2">
          <span class="size-2 shrink-0 rounded-full" :style="{ backgroundColor: identityHue(project.id).base }" aria-hidden="true" />
          <span class="text-body font-semibold text-text">{{ project.title }}</span>
          <span class="anchor-chip px-1.5 py-px text-micro">project:{{ project.label }}</span>
          <span class="text-caption tabular-nums text-text-subtle">{{ diagrams.length }}</span>
        </h2>
        <div class="grid grid-cols-[repeat(auto-fill,minmax(260px,1fr))] gap-3">
          <DiagramCard
            v-for="summary in diagrams"
            :key="summary.id"
            :summary="summary"
            :task-anchor="taskAnchorOf(summary.taskId)"
            :arrived="summary.id === arrivedId"
            @open="emit('open', summary.id)"
          />
        </div>
      </section>

      <div v-if="loaded && sections.length === 0 && drawing.length === 0" class="grid place-items-center pt-10">
        <AppEmptyState
          class="max-w-[540px]"
          title="Draw what the code does"
          description="A session reads the project's code and answers your question as Mermaid diagrams: who calls whom, a lifecycle, the decisions inside a function. Every box can open the code behind it."
        >
          <template #icon>
            <svg viewBox="0 0 20 20" fill="none" class="size-5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
              <path d="M5 3v14M15 3v14" stroke-dasharray="1.5 2" />
              <path d="M5 7h8.5M11.5 5.2 13.5 7l-2 1.8M15 13H6.5M8.5 11.2 6.5 13l2 1.8" />
            </svg>
          </template>
          <div class="flex gap-2">
            <AppButton size="sm" variant="secondary" @click="emit('import')">Import Mermaid</AppButton>
            <AppButton size="sm" variant="primary" @click="emit('generate')">Generate a diagram</AppButton>
          </div>
        </AppEmptyState>
      </div>
      <div v-else-if="!loaded" class="grid grid-cols-[repeat(auto-fill,minmax(260px,1fr))] gap-3" role="status" aria-busy="true">
        <span class="sr-only">Loading the diagrams</span>
        <span v-for="index in 6" :key="index" class="skeleton h-[196px] rounded-card" />
      </div>
    </div>
  </div>
</template>
