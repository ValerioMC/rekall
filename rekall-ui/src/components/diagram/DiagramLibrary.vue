<script setup lang="ts">
import { computed, ref } from 'vue'
import ProjectIcon from '@/components/ui/ProjectIcon.vue'
import { relativeTime } from '@/common/format/relative-time'
import { useNow } from '@/composables/useNow'
import type { DiagramSummary } from '@/model/diagram'
import type { Project, Task } from '@/model/catalog'
import type { DiagramId, ProjectId } from '@/model/branded'
import type { PendingGeneration } from '@/stores/diagram.store'

/**
 * Every diagram, grouped by project, newest first. A generation still being worked on sits at
 * the top of its project as a live row until the diagram it promised arrives.
 */
const props = defineProps<{
  summaries: readonly DiagramSummary[]
  projects: readonly Project[]
  tasks: readonly Task[]
  pending: readonly PendingGeneration[]
  selectedId: DiagramId | null
  arrivedId: DiagramId | null
  loaded: boolean
}>()

const emit = defineEmits<{ select: [id: DiagramId]; openTerminal: [generation: PendingGeneration]; dismiss: [key: number] }>()

const now = useNow(60_000)
const filter = ref('')

const projectById = computed(() => new Map(props.projects.map((project) => [project.id, project])))
const taskById = computed(() => new Map(props.tasks.map((task) => [task.id, task])))

interface Group {
  readonly projectId: ProjectId
  readonly project: Project | null
  readonly diagrams: DiagramSummary[]
  readonly pending: PendingGeneration[]
}

const groups = computed<Group[]>(() => {
  const needle = filter.value.trim().toLowerCase()
  const matches = (summary: DiagramSummary): boolean =>
    !needle || summary.title.toLowerCase().includes(needle) || summary.question.toLowerCase().includes(needle)
  const byProject = new Map<ProjectId, Group>()
  const groupOf = (projectId: ProjectId): Group => {
    let group = byProject.get(projectId)
    if (!group) {
      group = { projectId, project: projectById.value.get(projectId) ?? null, diagrams: [], pending: [] }
      byProject.set(projectId, group)
    }
    return group
  }
  for (const generation of props.pending) groupOf(generation.projectId).pending.push(generation)
  for (const summary of props.summaries) if (matches(summary)) groupOf(summary.projectId).diagrams.push(summary)
  return [...byProject.values()].filter((group) => group.diagrams.length + group.pending.length > 0)
})
</script>

<template>
  <aside class="flex w-(--spacing-nav) shrink-0 flex-col border-r border-border bg-surface/60" aria-label="Diagram library">
    <div class="border-b border-border p-3">
      <label class="relative block">
        <span class="sr-only">Filter diagrams</span>
        <svg viewBox="0 0 16 16" class="pointer-events-none absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-text-subtle" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" aria-hidden="true"><circle cx="7" cy="7" r="4.2" /><path d="m10.2 10.2 3.3 3.3" /></svg>
        <input v-model="filter" type="search" placeholder="Filter diagrams" class="field h-8 w-full rounded-[var(--radius-control)] pl-8 pr-2.5 text-[12.5px] text-text" data-testid="diagram-filter" />
      </label>
    </div>

    <div class="min-h-0 flex-1 overflow-y-auto px-2 pb-16 pt-2">
      <div v-if="!loaded" class="flex flex-col gap-2 p-1" role="status" aria-busy="true">
        <span class="sr-only">Loading diagrams</span>
        <span v-for="index in 4" :key="index" class="skeleton h-[62px] rounded-[var(--radius-control)]" />
      </div>

      <p v-else-if="groups.length === 0 && filter" class="px-3 py-6 text-center text-[12px] text-text-subtle">Nothing matches “{{ filter }}”.</p>

      <p v-else-if="groups.length === 0" class="px-3 py-6 text-center text-[12px] leading-relaxed text-text-subtle">
        No diagrams yet. Generate one from a task, or import a Semantic Graph.
      </p>

      <section v-for="group in groups" :key="group.projectId" class="mb-3" :aria-label="group.project?.title ?? 'Project'">
        <h3 class="flex h-7 items-center gap-2 px-2.5 text-[11.5px] font-medium text-text-subtle">
          <ProjectIcon :icon="group.project?.icon ?? 'folder'" :size="13" />
          <span class="min-w-0 flex-1 truncate">{{ group.project?.title ?? 'Unknown project' }}</span>
          <span class="font-mono tabular-nums">{{ group.diagrams.length }}</span>
        </h3>

        <div
          v-for="generation in group.pending"
          :key="`pending-${generation.key}`"
          class="pending-row mb-1 rounded-[var(--radius-control)] border border-dashed border-accent/35 px-3 py-2.5"
          role="status"
          data-testid="diagram-pending"
        >
          <div class="flex items-center gap-2">
            <span class="generating-orbit" aria-hidden="true" />
            <span class="text-[12px] font-medium text-accent-strong">Generating</span>
            <button class="focus-ring ml-auto grid size-5 place-items-center rounded text-text-subtle hover:text-text" aria-label="Stop waiting for this diagram" @click="emit('dismiss', generation.key)">
              <svg viewBox="0 0 12 12" class="size-3" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" aria-hidden="true"><path d="m3 3 6 6M9 3 3 9" /></svg>
            </button>
          </div>
          <p class="mt-1 line-clamp-2 text-[12px] leading-snug text-text-muted">{{ generation.request }}</p>
          <button class="focus-ring mt-1.5 rounded text-[11.5px] text-text-subtle underline-offset-2 hover:text-text hover:underline" @click="emit('openTerminal', generation)">
            In a terminal on {{ taskById.get(generation.taskId)?.title ?? 'the task' }} · open it
          </button>
        </div>

        <button
          v-for="summary in group.diagrams"
          :key="summary.id"
          class="focus-ring mb-0.5 block w-full rounded-[var(--radius-control)] px-3 py-2 text-left transition-colors"
          :class="[
            selectedId === summary.id ? 'selected-row' : 'hover:bg-surface-raised',
            arrivedId === summary.id ? 'row-arrived' : ''
          ]"
          :aria-current="selectedId === summary.id"
          data-testid="diagram-row"
          @click="emit('select', summary.id)"
        >
          <span class="flex items-baseline gap-2">
            <span class="min-w-0 flex-1 truncate text-[13px] font-medium" :class="selectedId === summary.id ? 'text-text' : 'text-text'">{{ summary.title }}</span>
            <span class="shrink-0 text-[11px] tabular-nums text-text-subtle">{{ relativeTime(summary.updatedAt, now) }}</span>
          </span>
          <span v-if="summary.question" class="mt-0.5 block truncate text-[12px] text-text-muted">{{ summary.question }}</span>
          <span class="mt-1 block font-mono text-[10.5px] tabular-nums text-text-subtle">{{ summary.nodeCount }} elements · {{ summary.edgeCount }} relations</span>
        </button>
      </section>
    </div>
  </aside>
</template>

<style scoped>
/* A comet on a ring: the one moving thing on this screen, and only while a session is working. */
.generating-orbit {
  position: relative;
  width: 12px;
  height: 12px;
  border-radius: 999px;
  border: 1.4px solid color-mix(in srgb, var(--color-accent) 28%, transparent);
}

.generating-orbit::after {
  content: '';
  position: absolute;
  inset: -1.4px;
  border-radius: 999px;
  border: 1.4px solid transparent;
  border-top-color: var(--color-accent);
  animation: orbit 1.1s linear infinite;
  filter: drop-shadow(0 0 3px var(--color-accent));
}

@keyframes orbit {
  to {
    transform: rotate(360deg);
  }
}

.pending-row {
  background: color-mix(in srgb, var(--color-accent) 4%, transparent);
}

/* A diagram that just arrived from a session announces itself once, then is a row like any other. */
.row-arrived {
  animation: row-arrived 1.4s ease-out 1;
}

@keyframes row-arrived {
  0% {
    box-shadow: 0 0 0 1px var(--color-accent), 0 0 18px -4px var(--color-accent);
  }
  100% {
    box-shadow: 0 0 0 1px transparent, 0 0 18px -4px transparent;
  }
}
</style>
