<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import { useConsoleStore } from '@/stores/console.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { useSettledOrder } from '@/composables/useSettledOrder'
import AppDialog from '@/components/ui/AppDialog.vue'
import AssignTaskColumn from '@/components/console/AssignTaskColumn.vue'
import AssignWalkColumn, { type WalkRow } from '@/components/console/AssignWalkColumn.vue'
import { identityHue } from '@/common/identity'
import { scopeAdmits, scopeName } from '@/model/note-scope'
import type { CompanyId, ProjectId, TaskId } from '@/model/branded'

const emit = defineEmits<{ close: [] }>()

const store = useConsoleStore()
const { companies, projects, tasks, selectedDocument, selectedTaskId } = storeToRefs(store)
const { run } = useAsyncAction()
const taskColumn = ref<InstanceType<typeof AssignTaskColumn> | null>(null)
const taskFilter = ref('')
const showAllTasks = ref(false)

const originTaskId = computed<TaskId | null>(() => {
  const doc = selectedDocument.value
  if (!doc) return null
  const here = doc.tasks.find((task) => task.id === selectedTaskId.value)
  return (here ?? doc.tasks[0])?.id ?? null
})

const pickedCompany = ref<CompanyId | null>(null)
const pickedProject = ref<ProjectId | null>(null)

const taskById = (id: TaskId | null) => tasks.value.find((task) => task.id === id) ?? null
const projectById = (id: ProjectId | null) => projects.value.find((p) => p.id === id) ?? null

onMounted(async () => {
  const origin = taskById(originTaskId.value)
  const originProject = projectById(origin?.projectId ?? null)
  const onlyProject = scopedProjects.value.length === 1 ? scopedProjects.value[0]! : null
  pickedProject.value = origin?.projectId ?? onlyProject?.id ?? null
  pickedCompany.value =
    originProject?.companyId ??
    onlyProject?.companyId ??
    (scopedCompanies.value.length === 1 ? scopedCompanies.value[0]!.id : null)

  await nextTick()
  taskColumn.value?.focusFilter()
})

// The columns offer only what the note's scope admits: a project note walks one project.
const scopedProjects = computed(() => {
  const scope = selectedDocument.value?.scope
  return scope ? projects.value.filter((project) => scopeAdmits(scope, project)) : []
})

const scopedCompanies = computed(() =>
  companies.value.filter((company) => scopedProjects.value.some((project) => project.companyId === company.id))
)

const scopeLabel = computed(() =>
  selectedDocument.value ? scopeName(selectedDocument.value.scope, projects.value, companies.value) : ''
)

const settledCompanies = useSettledOrder(() => scopedCompanies.value, (company) => company.id)

const companyProjects = computed(() =>
  pickedCompany.value === null
    ? []
    : scopedProjects.value.filter((project) => project.companyId === pickedCompany.value)
)

const listedProjects = useSettledOrder(() => companyProjects.value, (project) => project.id)

const pickedProjectTasks = computed(() =>
  pickedProject.value === null ? [] : tasks.value.filter((task) => task.projectId === pickedProject.value)
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

const companyRows = computed<WalkRow<CompanyId>[]>(() =>
  settledCompanies.value.map((company) => ({
    id: company.id,
    title: company.name,
    attached: companyAttachedCount(company.id),
    taskCount: company.taskCount
  }))
)

const projectRows = computed<WalkRow<ProjectId>[]>(() =>
  listedProjects.value.map((project) => ({
    id: project.id,
    title: project.title,
    attached: projectAttachedCount(project.id),
    taskCount: project.taskCount,
    hue: identityHue(project.id).base
  }))
)

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

function revealTask(id: TaskId): void {
  const task = taskById(id)
  if (!task) return
  pickedProject.value = task.projectId
  pickedCompany.value = projectById(task.projectId)?.companyId ?? pickedCompany.value
  taskFilter.value = ''
  showAllTasks.value = task.status === 'DONE'
}

async function toggle(taskId: TaskId): Promise<void> {
  const doc = selectedDocument.value
  if (!doc) return
  const current = doc.tasks.map((task) => task.id)
  if (attachedIds.value.has(taskId)) {
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

const membership = useSettledOrder(() => selectedDocument.value?.tasks ?? [], (task) => task.id)

watch(selectedDocument, (doc) => {
  if (!doc) emit('close')
})
</script>

<template>
  <AppDialog
    label="Choose the tasks this note belongs to"
    panel-class="flex flex-col h-[600px] max-h-[86vh] max-w-[760px]"
    panel-testid="note-assignment"
    @dismiss="emit('close')"
  >
    <header class="flex shrink-0 items-start gap-3 border-b border-border px-5 py-4">
      <span class="min-w-0 flex-1">
        <span class="block truncate text-title font-semibold tracking-[-0.01em] text-text">
          Tasks for {{ selectedDocument?.title }}
        </span>
        <span class="mt-0.5 block text-small leading-relaxed text-text-muted">
          It lives in {{ scopeLabel }} and can sit on any of its tasks. Open one and the note comes along.
        </span>
      </span>
      <button
        class="focus-ring grid size-7 shrink-0 place-items-center rounded-control text-text-subtle transition-colors hover:bg-surface-raised hover:text-text"
        aria-label="Close"
        @click="emit('close')"
      >
        <CloseGlyph />
      </button>
    </header>

    <div class="flex h-11 shrink-0 items-center gap-2 border-b border-border bg-canvas px-5">
      <span class="eyebrow shrink-0">On</span>
      <div class="chip-track flex min-w-0 flex-1 items-center gap-1.5 overflow-x-auto">
        <span
          v-for="member in membership"
          :key="member.id"
          class="inline-flex shrink-0 items-center gap-1.5 rounded-full border py-0.5 pl-2.5 pr-1 font-mono text-caption"
          :class="
            member.id === originTaskId
              ? 'border-accent bg-accent-soft text-accent'
              : 'border-anchor-line bg-anchor-soft text-anchor'
          "
        >
          <button
            class="focus-ring whitespace-nowrap rounded-key"
            :title="`Show ${member.title} in the columns`"
            @click="revealTask(member.id)"
          >
            {{ member.projectLabel }}/{{ member.label }}
          </button>
          <button
            class="focus-ring hit-target relative -m-0.5 grid size-5 shrink-0 place-items-center rounded-full text-text-subtle transition-colors hover:bg-danger-soft hover:text-danger"
            :aria-label="`Remove this note from ${member.title}`"
            @click="toggle(member.id)"
          >
            <CloseGlyph small />
          </button>
        </span>
        <span
          v-if="membership.length === 0"
          class="shrink-0 whitespace-nowrap pl-1 text-caption text-text-subtle"
        >
          no task yet: it waits in {{ scopeLabel }}
        </span>
      </div>
    </div>

    <div class="flex min-h-0 flex-1 flex-col sm:flex-row">
      <AssignWalkColumn
        heading="Company"
        region-label="Companies"
        :rows="companyRows"
        :picked="pickedCompany"
        empty-text="No company yet."
        testid="assign-company"
        @pick="pickCompany"
      />
      <AssignWalkColumn
        heading="Project"
        region-label="Projects"
        :rows="projectRows"
        :picked="pickedProject"
        :empty-text="pickedCompany === null ? 'Pick a company to see its projects.' : 'No project in this company yet.'"
        testid="assign-project"
        @pick="pickProject"
      />

      <AssignTaskColumn
        ref="taskColumn"
        v-model:filter="taskFilter"
        v-model:show-all="showAllTasks"
        :project-id="pickedProject"
        :tasks="pickedProjectTasks"
        :attached-ids="attachedIds"
        @toggle="toggle"
      />
    </div>

    <footer
      class="flex shrink-0 items-center justify-between gap-2 border-t border-border bg-canvas px-5 py-3"
    >
      <span class="font-mono text-caption text-text-subtle">
        on {{ membership.length }} task{{ membership.length === 1 ? '' : 's' }}
      </span>
      <AppButton variant="primary" size="sm" @click="emit('close')">Done</AppButton>
    </footer>
  </AppDialog>
</template>

<style scoped>
.chip-track {
  scrollbar-width: none;
}

.chip-track::-webkit-scrollbar {
  display: none;
}
</style>
