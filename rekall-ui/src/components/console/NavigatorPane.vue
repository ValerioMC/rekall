<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import ScopePicker from '@/components/console/ScopePicker.vue'
import RecordDialog from '@/components/console/RecordDialog.vue'
import { useConsoleStore } from '@/stores/console.store'
import { TASK_STATUS_LABEL, TASK_STATUS_ORDER } from '@/model/catalog'
import { taskDraft } from '@/model/record-draft'
import type { RecordDraft } from '@/model/record-draft'
import type { Task, TaskStatus } from '@/model/catalog'

/**
 * The navigator, and the one rule that governs its layout: every control sits above the
 * scrolling content and never moves.
 *
 * The previous sidebar listed projects underneath the task groups, so filtering changed their
 * height and the projects slid up and down. A target that moves is a target you have to find
 * again, a hundred times a day.
 */
const store = useConsoleStore()
const {
  scopeCompany,
  scopeProject,
  scopeName,
  navMode,
  visibleTasks,
  visibleDocuments,
  selectedTaskId,
  selectedDocId,
  elsewhere,
  isLoading
} = storeToRefs(store)

const STATUS_DOT: Record<TaskStatus, string> = {
  IN_PROGRESS: 'bg-accent',
  TODO: 'bg-text-subtle',
  BLOCKED: 'bg-danger',
  DONE: 'bg-safe'
}

const editing = ref<RecordDraft | null>(null)

/** A task needs a project. Scoped to one, that is the answer; otherwise it has to be picked. */
const projectChoices = computed(() => store.scopedProjects)

const grouped = computed(() =>
  TASK_STATUS_ORDER.map((status) => ({
    status,
    tasks: visibleTasks.value.filter((task) => task.status === status)
  })).filter((group) => group.tasks.length > 0)
)

/**
 * A new task opens on the project you are already in.
 *
 * When the scope does not name one, the project of the task in view is the next best answer and
 * the first in scope is the last resort. Whichever it lands on, the editor shows it as a field:
 * a guess that cannot be seen is how a task ends up in the wrong project.
 */
function beginCreate(): void {
  const projectId =
    scopeProject.value ?? store.selectedTask?.projectId ?? projectChoices.value[0]?.id
  if (!projectId) return
  editing.value = taskDraft(projectId)
}

function editTask(task: Task): void {
  editing.value = taskDraft(task.projectId, task)
}

/** The keyboard's way in, so editing does not require finding a row with the pointer first. */
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
  >
    <!-- Fixed controls. Nothing here shifts when the list below changes. -->
    <div class="flex shrink-0 flex-col gap-2.5 border-b border-border p-2.5">
      <ScopePicker />

      <div class="flex gap-0.5 rounded-[8px] bg-canvas p-0.5" role="group" aria-label="Browse by">
        <button
          v-for="mode in (['tasks', 'notes'] as const)"
          :key="mode"
          class="focus-ring flex h-7 flex-1 items-center justify-center gap-1.5 rounded-[6px] text-[12px] capitalize transition-colors"
          :class="
            navMode === mode
              ? 'bg-surface-raised text-text shadow-[0_1px_2px_rgb(0_0_0/0.4)]'
              : 'text-text-subtle hover:text-text'
          "
          :aria-pressed="navMode === mode"
          @click="navMode = mode"
        >
          {{ mode }}
          <span class="font-mono text-[10.5px] text-text-subtle">
            {{ mode === 'tasks' ? visibleTasks.length : visibleDocuments.length }}
          </span>
        </button>
      </div>
    </div>

    <div class="min-h-0 flex-1 overflow-y-auto pb-4">
      <!-- Shaped like the list it stands in for, so the empty states below never get a chance
           to flash "no project here yet" while the first fetch is still in flight. -->
      <div v-if="isLoading" class="flex flex-col gap-4 p-2 pt-3" aria-hidden="true">
        <div v-for="group in 3" :key="group" class="flex flex-col gap-1.5">
          <div class="skeleton mx-1.5 mb-1 h-2.5 w-16" />
          <div v-for="row in 2" :key="row" class="skeleton h-9 w-full rounded-[var(--radius-control)]" />
        </div>
      </div>

      <template v-else>
      <button
        v-if="navMode === 'tasks' && projectChoices.length"
        data-testid="new-task"
        class="focus-ring m-2 flex w-[calc(100%-16px)] items-center gap-2 rounded-[var(--radius-control)] border border-dashed border-border-strong px-2.5 py-2 text-left text-[12.5px] text-text-muted transition-colors hover:border-solid hover:border-accent hover:bg-accent-soft hover:text-text"
        @click="beginCreate"
      >
        <span class="text-accent">+</span> New task
        <span
          v-if="scopeProject !== null"
          class="ml-auto min-w-0 truncate font-mono text-[10.5px] text-text-subtle"
        >
          in {{ scopeName }}
        </span>
      </button>

      <template v-if="navMode === 'tasks'">
        <div v-for="group in grouped" :key="group.status" class="px-2">
          <div class="flex items-center gap-2 px-1.5 pb-1 pt-3">
            <span
              class="size-[7px] shrink-0 rounded-full"
              :class="STATUS_DOT[group.status]"
              aria-hidden="true"
            />
            <span class="text-[10px] font-semibold uppercase tracking-[0.09em] text-text-subtle">
              {{ TASK_STATUS_LABEL[group.status] }}
            </span>
            <span class="ml-auto font-mono text-[11px] text-text-subtle">
              {{ group.tasks.length }}
            </span>
          </div>

          <div
            v-for="task in group.tasks"
            :key="task.id"
            class="group/task relative flex items-center"
          >
            <button
              data-testid="task-row"
              class="focus-ring flex w-full items-center gap-2.5 rounded-[var(--radius-control)] px-2 py-1.5 text-left transition-colors"
              :class="
                task.id === selectedTaskId
                  ? 'selected-row text-text'
                  : 'text-text-muted hover:bg-surface-raised hover:text-text'
              "
              :aria-current="task.id === selectedTaskId"
              @click="store.selectTask(task.id)"
            >
              <span class="min-w-0 flex-1">
                <span class="block truncate text-[13px]">{{ task.title }}</span>
                <span class="block truncate font-mono text-[10.5px]">
                  <template v-if="scopeCompany === null">
                    <span class="text-text-subtle">{{ task.companyName }}/</span>
                  </template>
                  <template v-if="scopeProject === null">
                    <span class="text-text-subtle">{{ task.projectLabel }}/</span>
                  </template>
                  <span class="text-anchor/80">{{ task.label }}</span>
                </span>
              </span>
              <span
                class="flex shrink-0 items-center gap-1.5 font-mono text-[10.5px] text-text-subtle transition-opacity group-hover/task:opacity-0"
              >
                <!-- The same glyph the wrapup card carries, so the mark means one thing. -->
                <svg
                  v-if="task.hasWrapup"
                  class="size-2.5 text-text-muted"
                  viewBox="0 0 12 12"
                  role="img"
                >
                  <title>Has a wrapup</title>
                  <path d="M6 1.2 10.8 6 6 10.8 1.2 6z" fill="currentColor" />
                </svg>
                {{ task.documentCount }}
              </span>
            </button>
            <!-- Editing is on the row, not behind it. Hidden until wanted, never further away. -->
            <button
              data-testid="edit-task"
              class="focus-ring absolute right-1.5 grid size-6 place-items-center rounded text-text-subtle opacity-0 transition hover:bg-surface-hover hover:text-accent focus-visible:opacity-100 group-hover/task:opacity-100"
              :aria-label="`Edit ${task.title}`"
              @click="editTask(task)"
            >
              <svg class="size-3.5" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                <path
                  d="M4 20h4L20 8l-4-4L4 16v4z"
                  stroke="currentColor"
                  stroke-width="1.8"
                  stroke-linejoin="round"
                />
              </svg>
            </button>
          </div>
        </div>

        <p v-if="!grouped.length" class="px-4 py-3 text-[12.5px] leading-relaxed text-text-subtle">
          <template v-if="!projectChoices.length">
            No project here yet. Add one from the picker above, then a task can live in it.
          </template>
          <template v-else>No task here yet.</template>
        </p>
      </template>

      <template v-else>
        <div class="px-2">
          <button
            v-for="document in visibleDocuments"
            :key="document.id"
            data-testid="note-row"
            class="focus-ring mt-0.5 flex w-full items-center gap-2.5 rounded-[var(--radius-control)] px-2 py-1.5 text-left transition-colors"
            :class="
              document.id === selectedDocId
                ? 'selected-row text-text'
                : 'text-text-muted hover:bg-surface-raised hover:text-text'
            "
            :aria-current="document.id === selectedDocId"
            @click="store.selectDocument(document.id)"
          >
            <span class="min-w-0 flex-1">
              <span class="block truncate text-[13px]">{{ document.title }}</span>
              <span class="block truncate font-mono text-[10.5px] text-anchor/80">
                {{ document.tasks.map((t) => t.label).join(', ') }}
              </span>
            </span>
            <span
              v-if="document.tasks.length > 1"
              class="anchor-chip shrink-0 px-1.5 py-px text-[9.5px]"
              :title="`On ${document.tasks.length} tasks`"
            >
              &#8942; {{ document.tasks.length }}
            </span>
          </button>
        </div>

        <p
          v-if="!visibleDocuments.length"
          class="px-4 py-3 text-[12.5px] leading-relaxed text-text-subtle"
        >
          No note matches.
        </p>
      </template>

      <!-- Results the project scope is hiding, rather than a silent nothing. -->
      <button
        v-if="elsewhere"
        class="focus-ring m-2 flex w-[calc(100%-16px)] flex-wrap items-center gap-2 rounded-[var(--radius-control)] border border-border-strong bg-canvas px-2.5 py-2 text-left text-[12px] text-text-muted transition-colors hover:border-accent hover:text-text"
        @click="store.setScope(null)"
      >
        <span>{{ elsewhere.count }} more in {{ elsewhere.names.join(', ') }}</span>
        <span class="ml-auto whitespace-nowrap text-[11.5px] text-accent">
          search everywhere &#8594;
        </span>
      </button>
      </template>
    </div>

    <RecordDialog v-if="editing" :draft="editing" @close="editing = null" />
  </nav>
</template>
