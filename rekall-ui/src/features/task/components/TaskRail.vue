<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import CommitReferenceRail from '@/features/commits/components/CommitReferenceRail.vue'
import TaskLabelsStrip from '@/features/catalog/components/TaskLabelsStrip.vue'
import ContextSizeGroup from '@/features/context/components/ContextSizeGroup.vue'
import TaskRailGlyph, { type TaskRailGlyphName } from '@/features/task/components/TaskRailGlyph.vue'
import TaskRailGroup from '@/features/task/components/TaskRailGroup.vue'
import TaskRailIdentity from '@/features/task/components/TaskRailIdentity.vue'
import TaskBranchPanel from '@/features/worktree/components/TaskBranchPanel.vue'
import { useTaskBranchStore } from '@/features/worktree/store/taskBranch.store'
import { usePreferencesStore } from '@/features/settings/store/preferences.store'
import { useConsoleStore } from '@/stores/console.store'
import type { WorkRow } from '@/features/work/model/work-view'

/**
 * The task's properties beside every tab, grouped so the tabs keep their height: status and
 * anchor, labels, the context a session gets, and the git work. It folds to an icon bar, and the
 * choice is remembered per browser.
 */
const props = defineProps<{ row: WorkRow }>()

const FOLDED_GROUPS: readonly { readonly glyph: TaskRailGlyphName; readonly label: string }[] = [
  { glyph: 'labels', label: 'Labels' },
  { glyph: 'context', label: 'Context' },
  { glyph: 'git', label: 'Git' }
]

const preferences = usePreferencesStore()
const branches = useTaskBranchStore()
const { selectedTaskCommitReferences } = storeToRefs(useConsoleStore())

const task = computed(() => props.row.task)
const collapsed = computed(() => preferences.taskRailCollapsed)
const signature = computed(() => `${task.value.stepsDone}:${task.value.reviewState}:${task.value.updatedAt}`)
const commitCount = computed(() => selectedTaskCommitReferences.value.length)
const showsGit = computed(() => commitCount.value > 0 || branches.of(task.value.id) !== null)

function toggle(): void {
  preferences.taskRailCollapsed = !preferences.taskRailCollapsed
}
</script>

<template>
  <aside
    class="flex min-h-0 shrink-0 flex-col border-r border-border bg-surface transition-[width] duration-(--motion-move)"
    :class="collapsed ? 'w-13' : 'w-68'"
    aria-label="Task properties"
    data-testid="task-rail"
    :data-collapsed="collapsed"
  >
    <div class="flex h-11 shrink-0 items-center border-b border-border" :class="collapsed ? 'justify-center' : 'gap-2 pl-4 pr-3'">
      <h2 v-if="!collapsed" class="flex-1 text-small font-semibold text-text">Properties</h2>
      <button
        type="button"
        class="focus-ring grid size-7 cursor-pointer place-items-center rounded-control text-text-subtle transition-colors duration-(--motion-quick) hover:bg-surface-raised hover:text-text"
        :aria-label="collapsed ? 'Show the properties bar' : 'Fold the properties bar'"
        :aria-expanded="!collapsed"
        data-testid="task-rail-toggle"
        @click="toggle"
      >
        <TaskRailGlyph name="panel" />
      </button>
    </div>

    <div v-if="collapsed" class="flex flex-col items-center gap-1 py-2.5" data-testid="task-rail-icons">
      <button
        v-for="item in FOLDED_GROUPS"
        :key="item.glyph"
        type="button"
        class="focus-ring relative grid size-9 cursor-pointer place-items-center rounded-control text-text-subtle transition-colors duration-(--motion-quick) hover:bg-surface-raised hover:text-text"
        :title="item.label"
        :aria-label="`Show the properties bar, at ${item.label}`"
        @click="toggle"
      >
        <TaskRailGlyph :name="item.glyph" />
        <span
          v-if="item.glyph === 'git' && commitCount > 0"
          class="absolute bottom-0.5 right-0.5 min-w-4 rounded-full border border-border-strong bg-surface-raised px-1 text-center font-mono text-micro leading-3.5 text-text-muted"
        >
          {{ commitCount }}
        </span>
      </button>
    </div>

    <div v-else class="min-h-0 flex-1 overflow-y-auto overflow-x-hidden pb-16">
      <TaskRailIdentity :row="row" />
      <TaskRailGroup label="Labels" glyph="labels">
        <TaskLabelsStrip :task="task" stacked />
      </TaskRailGroup>
      <TaskRailGroup label="Context" glyph="context">
        <ContextSizeGroup :task="task" />
      </TaskRailGroup>
      <TaskRailGroup v-show="showsGit" label="Git" glyph="git">
        <template #aside>{{ commitCount || '' }}</template>
        <TaskBranchPanel :task-id="task.id" :signature="signature" />
        <CommitReferenceRail
          v-if="commitCount"
          :key="task.id"
          :references="selectedTaskCommitReferences"
          :class="{ 'mt-2.5 border-t border-border pt-2.5': branches.of(task.id) }"
        />
      </TaskRailGroup>
    </div>
  </aside>
</template>
