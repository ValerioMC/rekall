<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import ScopePicker from '@/features/catalog/components/ScopePicker.vue'
import RecordDialog from '@/features/catalog/components/RecordDialog.vue'
import NavigatorProjectGroup from '@/components/console/NavigatorProjectGroup.vue'
import NavigatorStatusGroups from '@/components/console/NavigatorStatusGroups.vue'
import NavigatorNoteList from '@/features/notes/components/NavigatorNoteList.vue'
import { groupTasksByProject } from '@/features/catalog/lib/project-task-groups'
import { revealOffset } from '@/common/scroll/reveal-offset'
import { useNavigatorShelves } from '@/composables/useNavigatorShelves'
import { useConsoleStore } from '@/stores/console.store'
import { taskDraft } from '@/features/catalog/model/record-draft'
import type { RecordDraft } from '@/features/catalog/model/record-draft'
import type { Task } from '@/features/catalog/model/catalog'

const store = useConsoleStore()
const {
  scopeCompany,
  scopeProject,
  scopeName,
  navMode,
  visibleTasks,
  visibleDocuments,
  selectedTaskId,
  selectedTask,
  noteComposerOpen,
  tasks,
  projects,
  elsewhere,
  isLoading,
  runningEntries
} = storeToRefs(store)

const runningTaskIds = computed(() => new Set(runningEntries.value.map((entry) => entry.taskId)))

const editing = ref<RecordDraft | null>(null)

const projectChoices = computed(() => store.scopedProjects)

const groupByProject = computed(() => navMode.value === 'tasks' && scopeProject.value === null)

const showNewTaskButton = computed(() => navMode.value === 'tasks' && projectChoices.value.length > 0)
const showNewNoteButton = computed(() => navMode.value === 'notes' && tasks.value.length > 0)

const groupedByProject = computed(() =>
  groupTasksByProject(visibleTasks.value, projects.value, scopeCompany.value === null)
)

const shelves = useNavigatorShelves(selectedTask)

const scroller = ref<HTMLElement | null>(null)

// Brings the selected task to the middle of the list when it is out of sight, smoothly unless
// the system asks for less motion. The sticky "new" bar covers the top of the viewport.
async function revealSelected(): Promise<void> {
  await nextTick()
  const list = scroller.value
  const row = list?.querySelector<HTMLElement>('[data-testid="task-row"][aria-current="true"]')
  if (!list || !row) return
  const stickyBar = list.querySelector<HTMLElement>('[data-nav-sticky]')
  const listBox = list.getBoundingClientRect()
  const rowBox = row.getBoundingClientRect()
  const offset = revealOffset(rowBox, {
    top: listBox.top + (stickyBar?.offsetHeight ?? 0),
    bottom: listBox.bottom
  })
  if (offset === null) return
  const reducedMotion = window.matchMedia?.('(prefers-reduced-motion: reduce)').matches ?? false
  list.scrollTo?.({ top: list.scrollTop + offset, behavior: reducedMotion ? 'auto' : 'smooth' })
}

watch([selectedTaskId, navMode, isLoading], () => {
  if (navMode.value === 'tasks' && !isLoading.value) void revealSelected()
}, { flush: 'post' })

function beginCreate(): void {
  const projectId =
    scopeProject.value ?? store.selectedTask?.projectId ?? projectChoices.value[0]?.id
  if (!projectId) return
  editing.value = taskDraft(projectId)
}

function editTask(task: Task): void {
  editing.value = taskDraft(task.projectId, task)
}

function editSelected(): void {
  const task = store.selectedTask
  if (task) editTask(task)
}

defineExpose({ beginCreate, editSelected })
</script>

<template>
  <nav
    class="flex min-h-0 w-(--spacing-nav) shrink-0 flex-col border-r border-border bg-surface"
    aria-label="Navigator"
    @pointerleave="revealSelected"
  >
    <div class="flex shrink-0 flex-col gap-2.5 border-b border-border p-2.5">
      <ScopePicker />

      <div class="flex gap-0.5 rounded-card bg-canvas p-0.5" role="group" aria-label="Browse by">
        <button
          v-for="mode in (['tasks', 'notes'] as const)"
          :key="mode"
          class="focus-ring flex h-7 flex-1 items-center justify-center gap-1.5 rounded-control text-small capitalize transition-colors"
          :class="
            navMode === mode
              ? 'bg-surface-raised text-text shadow-[0_1px_2px_rgb(0_0_0/0.4)]'
              : 'text-text-subtle hover:text-text'
          "
          :aria-pressed="navMode === mode"
          @click="store.setNavMode(mode)"
        >
          {{ mode }}
          <span class="font-mono text-micro text-text-subtle">
            {{ mode === 'tasks' ? visibleTasks.length : visibleDocuments.length }}
          </span>
        </button>
      </div>
    </div>

    <div ref="scroller" class="min-h-0 flex-1 overflow-y-auto pb-4">
      <div v-if="isLoading" class="flex flex-col gap-4 p-2 pt-3" aria-hidden="true">
        <div v-for="group in 3" :key="group" class="flex flex-col gap-1.5">
          <div class="skeleton mx-1.5 mb-1 h-2.5 w-16" />
          <div v-for="row in 2" :key="row" class="skeleton h-9 w-full rounded-control" />
        </div>
      </div>

      <template v-else>
      <div
        v-if="showNewTaskButton || showNewNoteButton"
        class="glass sticky top-0 z-(--z-sticky) border-b border-border/60 p-2"
        data-nav-sticky
      >
        <button
          v-if="showNewTaskButton"
          data-testid="new-task"
          class="add-row focus-ring py-2"
          @click="beginCreate"
        >
          <span class="text-accent">+</span> New task
          <span
            v-if="scopeProject !== null"
            class="ml-auto min-w-0 truncate font-mono text-micro text-text-subtle"
          >
            in {{ scopeName }}
          </span>
        </button>

        <button
          v-if="showNewNoteButton"
          data-testid="new-note-here"
          class="add-row focus-ring py-2"
          :class="noteComposerOpen && 'add-row-open'"
          :aria-pressed="noteComposerOpen"
          @click="store.openNoteComposer()"
        >
          <span class="text-accent">+</span> New note
          <kbd class="ml-auto rounded-key border border-border px-1 font-mono text-micro text-text-subtle">N</kbd>
        </button>
      </div>

      <template v-if="navMode === 'tasks' && groupByProject">
        <NavigatorProjectGroup
          v-for="group in groupedByProject"
          :key="group.projectId"
          :group="group"
          :shelves="shelves"
          :show-company="scopeCompany === null"
          :running-task-ids="runningTaskIds"
          @edit="editTask"
        />

        <p v-if="!groupedByProject.length" class="px-4 py-3 text-small leading-relaxed text-text-subtle">
          <template v-if="!projectChoices.length">
            No project here yet. Add one from the picker above, then a task can live in it.
          </template>
          <template v-else>No task here yet.</template>
        </p>
      </template>

      <NavigatorStatusGroups
        v-else-if="navMode === 'tasks'"
        :shelves="shelves"
        :running-task-ids="runningTaskIds"
        :has-projects="projectChoices.length > 0"
        @edit="editTask"
      />

      <NavigatorNoteList v-else />

      <button
        v-if="elsewhere"
        class="focus-ring m-2 flex w-[calc(100%-16px)] flex-wrap items-center gap-2 rounded-control border border-border-strong bg-canvas px-2.5 py-2 text-left text-small text-text-muted transition-colors hover:border-accent hover:text-text"
        @click="store.setScope(null)"
      >
        <span>{{ elsewhere.count }} more in {{ elsewhere.names.join(', ') }}</span>
        <span class="ml-auto whitespace-nowrap text-caption text-accent">
          search everywhere &#8594;
        </span>
      </button>
      </template>
    </div>

    <Transition name="dialog">
      <RecordDialog v-if="editing" :draft="editing" @close="editing = null" />
    </Transition>
  </nav>
</template>
