<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useModalGate } from '@/composables/useModalGate'
import { trapTabKey } from '@/common/a11y/focus-trap'
import { identityHue } from '@/common/identity'
import { TASK_STATUS_COLOR, TASK_STATUS_LABEL } from '@/model/catalog'
import type { CompanyId, ProjectId, TaskId } from '@/model/branded'

/**
 * Where a note belongs, chosen the way the work is actually shaped: company, then project, then
 * task.
 *
 * The flat list this replaces asked you to recognise one task title among every task there is.
 * Three linked columns cut that to the handful in one project, and the columns open already
 * standing on the task the note was written against, so the common case is one click to confirm.
 *
 * Only the task column changes what the note is attached to. Picking a company or a project just
 * walks the tree; the strip along the top is the note's membership and the only place it is
 * edited, from whichever column you happen to be in.
 */
const emit = defineEmits<{ close: [] }>()

const store = useConsoleStore()
const { companies, projects, tasks, selectedDocument, selectedTaskId } = storeToRefs(store)
const { run } = useAsyncAction()
const { open: openModal, close: closeModal } = useModalGate()

const panel = ref<HTMLElement | null>(null)
const filterField = ref<HTMLInputElement | null>(null)
const taskFilter = ref('')

/**
 * A project accumulates finished tasks that are almost never the one a note is being filed
 * against. The task pane hides them until asked; `All` brings them back.
 */
const TASK_SCOPES = [
  { value: 'open', label: 'Open' },
  { value: 'all', label: 'All' }
] as const
const showAllTasks = ref(false)

/**
 * The task the note was written against: the one in view if the note sits on it, otherwise the
 * first it is on. Its project and company are where the columns open.
 */
const originTaskId = computed<TaskId | null>(() => {
  const doc = selectedDocument.value
  if (!doc) return null
  const here = doc.tasks.find((task) => task.id === selectedTaskId.value)
  return (here ?? doc.tasks[0])?.id ?? null
})

const pickedCompany = ref<CompanyId | null>(null)
const pickedProject = ref<ProjectId | null>(null)

/** The full task carries the ids a note's lightweight ref does not. */
const taskById = (id: TaskId | null) => tasks.value.find((task) => task.id === id) ?? null
const projectById = (id: ProjectId | null) => projects.value.find((p) => p.id === id) ?? null

onMounted(async () => {
  openModal()
  window.addEventListener('keydown', onKeydown, true)

  const origin = taskById(originTaskId.value)
  const originProject = projectById(origin?.projectId ?? null)
  pickedProject.value = origin?.projectId ?? null
  pickedCompany.value =
    originProject?.companyId ?? (companies.value.length === 1 ? companies.value[0]!.id : null)

  await nextTick()
  filterField.value?.focus()
})

onUnmounted(() => {
  closeModal()
  window.removeEventListener('keydown', onKeydown, true)
})

const companyProjects = computed(() =>
  pickedCompany.value === null
    ? []
    : projects.value.filter((project) => project.companyId === pickedCompany.value)
)

const projectTasks = computed(() => {
  if (pickedProject.value === null) return []
  const needle = taskFilter.value.trim().toLowerCase()
  return tasks.value
    .filter((task) => task.projectId === pickedProject.value)
    .filter((task) => showAllTasks.value || task.status !== 'DONE')
    .filter((task) => !needle || `${task.title} ${task.label}`.toLowerCase().includes(needle))
})

/** How many finished tasks the current project is hiding, so the toggle only shows when it matters. */
const doneInProject = computed(() =>
  pickedProject.value === null
    ? 0
    : tasks.value.filter(
        (task) => task.projectId === pickedProject.value && task.status === 'DONE'
      ).length
)

const attachedIds = computed(
  () => new Set(selectedDocument.value?.tasks.map((task) => task.id) ?? [])
)

const projectAttachedCount = (projectId: ProjectId) =>
  (selectedDocument.value?.tasks ?? []).filter(
    (entry) => taskById(entry.id)?.projectId === projectId
  ).length

const companyAttachedCount = (companyId: CompanyId) =>
  (selectedDocument.value?.tasks ?? []).filter((entry) => {
    const project = projectById(taskById(entry.id)?.projectId ?? null)
    return project?.companyId === companyId
  }).length

function pickCompany(id: CompanyId): void {
  pickedCompany.value = id
  if (projectById(pickedProject.value)?.companyId !== id) pickedProject.value = null
  taskFilter.value = ''
  showAllTasks.value = false
}

function pickProject(id: ProjectId): void {
  pickedProject.value = id
  taskFilter.value = ''
  showAllTasks.value = false
}

/** Walk the columns to a task the note already sits on, from a chip in the strip. */
function revealTask(id: TaskId): void {
  const task = taskById(id)
  if (!task) return
  pickedProject.value = task.projectId
  pickedCompany.value = projectById(task.projectId)?.companyId ?? pickedCompany.value
  taskFilter.value = ''
  // The strip can point at a finished task; the pane has to be showing them for the walk to land.
  showAllTasks.value = task.status === 'DONE'
}

async function toggle(taskId: TaskId): Promise<void> {
  const doc = selectedDocument.value
  if (!doc) return
  const current = doc.tasks.map((task) => task.id)
  if (attachedIds.value.has(taskId)) {
    if (current.length === 1) return
    await run(
      () => store.saveNote(doc.id, { taskIds: current.filter((id) => id !== taskId) }),
      'Removed from that task.'
    )
  } else {
    await run(
      () => store.saveNote(doc.id, { taskIds: [...current, taskId] }),
      'Added to that task.'
    )
  }
}

/** The strip stays put even as the note moves between tasks under it. */
const membership = computed(() => selectedDocument.value?.tasks ?? [])

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    event.stopPropagation()
    emit('close')
    return
  }
  if (panel.value) trapTabKey(panel.value, event)
}

// A note that loses its last task has nowhere to be reached from; the dialog closes with the
// note it was editing rather than lingering over nothing.
watch(selectedDocument, (doc) => {
  if (!doc) emit('close')
})
</script>

<template>
  <div
    class="fade-in fixed inset-0 z-(--z-modal) grid place-items-center bg-black/70 p-5 backdrop-blur-sm"
    @click.self="emit('close')"
  >
    <!--
      The frame never resizes. Height and width are set once; walking company -> project -> task
      and the row counts under each only move the scrollbars inside the three panes, never the
      panel's edges.
    -->
    <div
      ref="panel"
      class="rise flex h-[600px] max-h-[86vh] w-full max-w-[760px] flex-col overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface shadow-modal"
      role="dialog"
      aria-modal="true"
      aria-label="Choose the tasks this note belongs to"
      data-testid="note-assignment"
    >
      <header class="flex shrink-0 items-start gap-3 border-b border-border px-5 py-4">
        <span class="min-w-0 flex-1">
          <span class="block truncate text-[15px] font-semibold tracking-[-0.01em] text-text">
            Tasks for {{ selectedDocument?.title }}
          </span>
          <span class="mt-0.5 block text-[12.5px] leading-relaxed text-text-muted">
            A note can sit on several tasks. Open any of them and it comes along.
          </span>
        </span>
        <button
          class="focus-ring grid size-7 shrink-0 place-items-center rounded-md text-text-subtle transition-colors hover:bg-surface-raised hover:text-text"
          aria-label="Close"
          @click="emit('close')"
        >
          &times;
        </button>
      </header>

      <!--
        Where the note lives now, edited from here whichever column you are in. One line, fixed
        height: extra tasks scroll sideways rather than pushing the panes down. The note's home
        task carries the amber; every other membership is a plain cyan anchor.
      -->
      <div class="flex h-11 shrink-0 items-center gap-2 border-b border-border bg-canvas px-5">
        <span class="eyebrow shrink-0">On</span>
        <div class="chip-track flex min-w-0 flex-1 items-center gap-1.5 overflow-x-auto">
          <span
            v-for="member in membership"
            :key="member.id"
            class="inline-flex shrink-0 items-center gap-1.5 rounded-full border py-0.5 pl-2.5 pr-1 font-mono text-[11px]"
            :class="
              member.id === originTaskId
                ? 'border-accent bg-accent-soft text-accent'
                : 'border-anchor-line bg-anchor-soft text-anchor'
            "
          >
            <button
              class="focus-ring whitespace-nowrap rounded"
              :title="`Show ${member.title} in the columns`"
              @click="revealTask(member.id)"
            >
              {{ member.projectLabel }}/{{ member.label }}
            </button>
            <button
              v-if="membership.length > 1"
              class="focus-ring -m-0.5 grid size-5 shrink-0 place-items-center rounded-full text-text-subtle transition-colors hover:bg-danger-soft hover:text-danger"
              :aria-label="`Remove this note from ${member.title}`"
              @click="toggle(member.id)"
            >
              &times;
            </button>
          </span>
          <span
            v-if="membership.length === 1"
            class="shrink-0 whitespace-nowrap pl-1 text-[11px] text-text-subtle"
          >
            a note needs at least one
          </span>
        </div>
      </div>

      <!-- Company, then project, then task. Three panes on a wide screen, stacked below. -->
      <div class="flex min-h-0 flex-1 flex-col sm:flex-row">
        <section
          class="flex min-h-0 min-w-0 flex-1 basis-0 flex-col border-b border-border sm:border-b-0 sm:border-r"
          aria-label="Companies"
        >
          <div class="flex shrink-0 items-baseline justify-between px-3.5 pb-1.5 pt-3">
            <p class="eyebrow">Company</p>
            <span v-if="companies.length" class="font-mono text-[10px] text-text-subtle">
              {{ companies.length }}
            </span>
          </div>
          <div class="relative min-h-0 flex-1">
            <div
              v-if="companies.length"
              class="h-full space-y-0.5 overflow-y-auto overscroll-contain px-1.5 py-1"
            >
              <button
                v-for="company in companies"
                :key="company.id"
                data-testid="assign-company"
                class="focus-ring flex w-full items-center gap-2 rounded-[var(--radius-control)] px-2.5 py-2 text-left transition-colors"
                :class="
                  company.id === pickedCompany
                    ? 'walk-picked bg-surface-raised text-text'
                    : 'text-text-muted hover:bg-surface-raised hover:text-text'
                "
                :aria-pressed="company.id === pickedCompany"
                @click="pickCompany(company.id)"
              >
                <span
                  class="min-w-0 flex-1 truncate text-[12.5px]"
                  :title="company.name"
                >{{ company.name }}</span>
                <span
                  v-if="companyAttachedCount(company.id)"
                  class="shrink-0 rounded-full bg-accent-soft px-1.5 font-mono text-[10px] text-accent"
                  :title="`${companyAttachedCount(company.id)} of this note's tasks are here`"
                >
                  {{ companyAttachedCount(company.id) }}
                </span>
                <span
                  class="shrink-0 font-mono text-[10.5px] text-text-subtle"
                  :title="`${company.taskCount} tasks`"
                >
                  {{ company.taskCount }}
                </span>
              </button>
            </div>
            <div v-else class="grid h-full place-items-center px-5">
              <p class="text-center text-[12px] leading-relaxed text-text-subtle">No company yet.</p>
            </div>
            <div
              v-if="companies.length"
              class="pointer-events-none absolute inset-x-1.5 bottom-0 h-5 bg-gradient-to-t from-surface to-transparent"
              aria-hidden="true"
            />
          </div>
        </section>

        <section
          class="flex min-h-0 min-w-0 flex-1 basis-0 flex-col border-b border-border sm:border-b-0 sm:border-r"
          aria-label="Projects"
        >
          <div class="flex shrink-0 items-baseline justify-between px-3.5 pb-1.5 pt-3">
            <p class="eyebrow">Project</p>
            <span
              v-if="pickedCompany !== null && companyProjects.length"
              class="font-mono text-[10px] text-text-subtle"
            >
              {{ companyProjects.length }}
            </span>
          </div>
          <div class="relative min-h-0 flex-1">
            <div
              v-if="pickedCompany !== null && companyProjects.length"
              class="h-full space-y-0.5 overflow-y-auto overscroll-contain px-1.5 py-1"
            >
              <button
                v-for="project in companyProjects"
                :key="project.id"
                data-testid="assign-project"
                class="focus-ring flex w-full items-center gap-2 rounded-[var(--radius-control)] px-2.5 py-2 text-left transition-colors"
                :class="
                  project.id === pickedProject
                    ? 'walk-picked bg-surface-raised text-text'
                    : 'text-text-muted hover:bg-surface-raised hover:text-text'
                "
                :aria-pressed="project.id === pickedProject"
                @click="pickProject(project.id)"
              >
                <span
                  class="size-1.5 shrink-0 rounded-full"
                  :style="{ backgroundColor: identityHue(project.id).base }"
                  aria-hidden="true"
                />
                <span
                  class="min-w-0 flex-1 truncate text-[12.5px]"
                  :title="project.title"
                >{{ project.title }}</span>
                <span
                  v-if="projectAttachedCount(project.id)"
                  class="shrink-0 rounded-full bg-accent-soft px-1.5 font-mono text-[10px] text-accent"
                  :title="`${projectAttachedCount(project.id)} of this note's tasks are here`"
                >
                  {{ projectAttachedCount(project.id) }}
                </span>
                <span
                  class="shrink-0 font-mono text-[10.5px] text-text-subtle"
                  :title="`${project.taskCount} tasks`"
                >
                  {{ project.taskCount }}
                </span>
              </button>
            </div>
            <div v-else class="grid h-full place-items-center px-5">
              <p class="text-center text-[12px] leading-relaxed text-text-subtle">
                {{
                  pickedCompany === null
                    ? 'Pick a company to see its projects.'
                    : 'No project in this company yet.'
                }}
              </p>
            </div>
            <div
              v-if="pickedCompany !== null && companyProjects.length"
              class="pointer-events-none absolute inset-x-1.5 bottom-0 h-5 bg-gradient-to-t from-surface to-transparent"
              aria-hidden="true"
            />
          </div>
        </section>

        <section class="flex min-h-0 min-w-0 flex-1 basis-0 flex-col" aria-label="Tasks">
          <div class="shrink-0 px-3.5 pb-2 pt-3">
            <div class="flex items-center justify-between pb-1.5">
              <p class="eyebrow">Task</p>
              <div v-if="pickedProject !== null" class="flex items-center gap-2">
                <div
                  v-if="doneInProject"
                  class="flex gap-0.5 rounded-[6px] bg-canvas p-0.5"
                  role="group"
                  aria-label="Which tasks to show"
                >
                  <button
                    v-for="scope in TASK_SCOPES"
                    :key="scope.value"
                    class="focus-ring h-[18px] rounded-[4px] px-1.5 text-[10px] transition-colors"
                    :class="
                      (scope.value === 'all') === showAllTasks
                        ? 'bg-surface-raised text-text'
                        : 'text-text-subtle hover:text-text'
                    "
                    :aria-pressed="(scope.value === 'all') === showAllTasks"
                    :data-testid="`assign-scope-${scope.value}`"
                    @click="showAllTasks = scope.value === 'all'"
                  >
                    {{ scope.label }}
                  </button>
                </div>
                <span class="font-mono text-[10px] text-text-subtle">
                  {{ projectTasks.length }}
                </span>
              </div>
            </div>
            <input
              ref="filterField"
              v-model="taskFilter"
              type="search"
              data-testid="assign-task-filter"
              placeholder="Filter tasks"
              :disabled="pickedProject === null"
              class="focus-ring h-8 w-full rounded-[var(--radius-control)] border border-border bg-canvas px-2.5 text-[12px] text-text transition-colors placeholder:text-text-subtle hover:border-border-strong focus-visible:border-accent-deep disabled:opacity-40"
            />
          </div>
          <div class="relative min-h-0 flex-1">
            <div
              v-if="pickedProject !== null && projectTasks.length"
              class="h-full space-y-0.5 overflow-y-auto overscroll-contain px-1.5 py-1"
            >
              <button
                v-for="task in projectTasks"
                :key="task.id"
                data-testid="assign-task"
                class="focus-ring flex w-full items-center gap-2.5 rounded-[var(--radius-control)] px-2.5 py-2 text-left transition-colors"
                :class="
                  attachedIds.has(task.id)
                    ? 'bg-surface-raised text-text'
                    : 'text-text-muted hover:bg-surface-raised hover:text-text'
                "
                :aria-pressed="attachedIds.has(task.id)"
                @click="toggle(task.id)"
              >
                <span
                  class="grid size-4 shrink-0 place-items-center rounded-[5px] border transition-colors"
                  :class="
                    attachedIds.has(task.id)
                      ? 'border-accent-deep bg-accent-soft text-accent'
                      : 'border-border-strong text-transparent'
                  "
                  aria-hidden="true"
                >
                  <svg class="size-3" viewBox="0 0 24 24" fill="none">
                    <path
                      d="M5 12.5l4.5 4.5L19 7.5"
                      stroke="currentColor"
                      stroke-width="2.6"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                    />
                  </svg>
                </span>
                <span class="min-w-0 flex-1">
                  <span class="block truncate text-[12.5px]" :title="task.title">
                    {{ task.title }}
                  </span>
                  <span
                    class="anchor-chip mt-0.5 inline-block max-w-full truncate align-bottom px-1.5 py-px text-[9.5px] leading-[15px]"
                    :title="task.label"
                  >
                    {{ task.label }}
                  </span>
                </span>
                <span
                  class="size-[7px] shrink-0 rounded-full"
                  :class="TASK_STATUS_COLOR[task.status]"
                  :title="TASK_STATUS_LABEL[task.status]"
                  aria-hidden="true"
                />
              </button>
            </div>
            <div v-else class="grid h-full place-items-center px-5">
              <p class="text-center text-[12px] leading-relaxed text-text-subtle">
                <template v-if="pickedProject === null">Pick a project to see its tasks.</template>
                <template v-else-if="taskFilter.trim()">No task matches that filter.</template>
                <template v-else-if="!showAllTasks && doneInProject">
                  Every task here is done. Switch to All to see them.
                </template>
                <template v-else>No task in this project yet.</template>
              </p>
            </div>
            <div
              v-if="pickedProject !== null && projectTasks.length"
              class="pointer-events-none absolute inset-x-1.5 bottom-0 h-5 bg-gradient-to-t from-surface to-transparent"
              aria-hidden="true"
            />
          </div>
        </section>
      </div>

      <footer
        class="flex shrink-0 items-center justify-between gap-2 border-t border-border bg-canvas px-5 py-3"
      >
        <span class="font-mono text-[11px] text-text-subtle">
          on {{ membership.length }} task{{ membership.length === 1 ? '' : 's' }}
        </span>
        <AppButton variant="primary" size="sm" @click="emit('close')">Done</AppButton>
      </footer>
    </div>
  </div>
</template>

<style scoped>
/*
 * The walk selection in the company and project panes. Not the app's full `selected-row` lift,
 * which on a task row read as a claim the row had not made: just a quiet raised surface and a
 * short amber capsule in the gutter, so it marks where you are without competing with the
 * checked state next to it.
 */
.walk-picked {
  position: relative;
}

.walk-picked::before {
  content: '';
  position: absolute;
  left: 3px;
  top: 7px;
  bottom: 7px;
  width: 3px;
  border-radius: 999px;
  background: linear-gradient(180deg, var(--color-accent-strong), var(--color-accent-deep));
}

/* The membership line scrolls sideways when a note is on many tasks; the bar stays one line. */
.chip-track {
  scrollbar-width: none;
}

.chip-track::-webkit-scrollbar {
  display: none;
}
</style>
