<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink } from 'vue-router'
import { epicEntries } from '@/features/work/lib/epic-entries'
import { useConsoleStore } from '@/stores/console.store'
import ProjectMark from '@/components/ui/ProjectMark.vue'
import WorkRowView from '@/features/work/components/WorkRow.vue'
import { formatTokens } from '@/common/format/token-count'
import type { WorkDensity, WorkGroup } from '@/features/work/model/work-view'
import type { TaskId } from '@/model/branded'

const props = defineProps<{
  group: WorkGroup
  density: WorkDensity
  now: number
  filedUnfolded: boolean
  foldedEpics: ReadonlySet<TaskId>
}>()

defineEmits<{ toggleFiled: []; toggleEpic: [epicId: TaskId] }>()

const store = useConsoleStore()

const entries = computed(() => epicEntries(props.group.rows, titleOfEpic))
const filedEntries = computed(() => epicEntries(props.group.filed, titleOfEpic))

function titleOfEpic(id: TaskId): string | null {
  return store.tasks.find((candidate) => candidate.id === id)?.title ?? null
}

const taskCountLabel = computed(() => `${props.group.rows.length} ${props.group.rows.length === 1 ? 'task' : 'tasks'}`)
</script>

<template>
  <section :aria-label="group.projectTitle" data-testid="work-group">
    <header class="flex h-[54px] items-center gap-2.5 px-3 pt-4">
      <ProjectMark :project-id="group.projectId" :icon="group.icon" :image-version="group.iconImageVersion" :size="20" />
      <RouterLink
        :to="`/projects/${group.projectId}`"
        class="focus-ring text-body font-semibold text-text hover:text-accent-strong"
      >
        {{ group.projectTitle }}
      </RouterLink>
      <span class="text-small text-text-muted">{{ group.companyName }}</span>
      <span class="font-mono text-caption text-text-subtle">
        {{ taskCountLabel }}
        <template v-if="group.needs > 0"> · <span class="text-accent-strong">{{ group.needs }} to review</span></template>
      </span>
      <span v-if="group.tokens > 0" class="ml-auto font-mono text-caption text-text-subtle">
        {{ formatTokens(group.tokens) }} tokens
      </span>
    </header>

    <template v-for="entry in entries" :key="entry.row.task.id">
      <WorkRowView
        v-if="entry.kind === 'epic'"
        :row="entry.row"
        :density="density"
        :now="now"
        :epic="{ members: entry.members, folded: foldedEpics.has(entry.row.task.id) }"
        @toggle-epic="$emit('toggleEpic', entry.row.task.id)"
      />
      <WorkRowView v-else :row="entry.row" :density="density" :now="now" :epic-title="entry.epicTitle" />
      <template v-if="entry.kind === 'epic' && !foldedEpics.has(entry.row.task.id)">
        <WorkRowView v-for="member in entry.members" :key="member.task.id" :row="member" :density="density" :now="now" nested />
      </template>
    </template>

    <template v-if="group.filed.length > 0">
      <button
        type="button"
        class="focus-ring flex h-[34px] w-full cursor-pointer items-center gap-2 rounded-control px-3 text-small text-filed transition-colors duration-(--motion-quick) hover:bg-filed-soft"
        :aria-expanded="filedUnfolded"
        data-testid="work-filed-fold"
        @click="$emit('toggleFiled')"
      >
        <svg
          class="size-3 transition-transform duration-(--motion-move)"
          :class="filedUnfolded && 'rotate-90'"
          viewBox="0 0 12 12"
          fill="none"
          aria-hidden="true"
        >
          <path d="m4.5 2.5 3.5 3.5-3.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        {{ group.filed.length }} filed
      </button>
      <template v-if="filedUnfolded">
        <template v-for="entry in filedEntries" :key="entry.row.task.id">
          <WorkRowView v-if="entry.kind === 'epic'" :row="entry.row" :density="density" :now="now" :epic="{ members: entry.members, folded: foldedEpics.has(entry.row.task.id) }" @toggle-epic="$emit('toggleEpic', entry.row.task.id)" />
          <WorkRowView v-else :row="entry.row" :density="density" :now="now" :epic-title="entry.epicTitle" />
          <template v-if="entry.kind === 'epic' && !foldedEpics.has(entry.row.task.id)">
            <WorkRowView v-for="member in entry.members" :key="member.task.id" :row="member" :density="density" :now="now" nested />
          </template>
        </template>
      </template>
    </template>
  </section>
</template>
