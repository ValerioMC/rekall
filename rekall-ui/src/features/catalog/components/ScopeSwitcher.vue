<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import CompanyMark from '@/components/ui/CompanyMark.vue'
import ProjectMark from '@/components/ui/ProjectMark.vue'
import RecordDialog from '@/features/catalog/components/RecordDialog.vue'
import ScopeProjectOption from '@/features/catalog/components/ScopeProjectOption.vue'
import { switcherSections } from '@/features/catalog/lib/switcher-sections'
import { companyDraft, projectDraft, type RecordDraft } from '@/features/catalog/model/record-draft'
import { useRecentProjectsStore } from '@/features/catalog/store/recentProjects.store'
import { useConsoleStore } from '@/stores/console.store'
import type { Company, Project } from '@/features/catalog/model/catalog'
import type { ScopeTarget } from '@/features/catalog/model/scope-target'
import type { CompanyId, ProjectId } from '@/model/branded'

/**
 * The project in view, as the page's title, and the list that changes it: recent projects, then
 * every company with its projects. On Work it narrows the list (all work, a company or a project);
 * on a project page it only moves between projects. The page decides what a choice does.
 */
const props = defineProps<{ mode: 'work' | 'project'; companyId: CompanyId | null; projectId: ProjectId | null }>()

const emit = defineEmits<{ select: [target: ScopeTarget]; 'open-page': [projectId: ProjectId] }>()

interface Entry {
  readonly key: string
  readonly target: ScopeTarget
}

const store = useConsoleStore()
const recent = useRecentProjectsStore()
const { companies, projects, tasks } = storeToRefs(store)

const isOpen = ref(false)
const query = ref('')
const cursor = ref(0)
const editing = ref<RecordDraft | null>(null)
const trigger = ref<HTMLButtonElement | null>(null)
const search = ref<HTMLInputElement | null>(null)
const list = ref<HTMLElement | null>(null)

const isWork = computed(() => props.mode === 'work')
const searching = computed(() => query.value.trim() !== '')
const markedProject = computed(() => projects.value.find((project) => project.id === props.projectId) ?? null)
const markedCompany = computed(() => {
  const companyId = markedProject.value?.companyId ?? props.companyId
  return companies.value.find((company) => company.id === companyId) ?? null
})
const sections = computed(() => switcherSections(projects.value, recent.ids, query.value, props.projectId))
const showsAll = computed(() => isWork.value && !searching.value)
const hasMatch = computed(() => showsAll.value || sections.value.recent.length > 0 || sections.value.companies.length > 0)
const everyone = computed(() => props.companyId === null && props.projectId === null)

function projectTarget(project: Project): ScopeTarget {
  return { kind: 'project', companyId: project.companyId, projectId: project.id }
}

const entries = computed<Entry[]>(() => [
  ...(showsAll.value ? [{ key: 'all', target: { kind: 'all' } as ScopeTarget }] : []),
  ...sections.value.recent.map((project) => ({ key: `recent:${project.id}`, target: projectTarget(project) })),
  ...sections.value.companies.flatMap((group) =>
    group.projects.map((project) => ({ key: `list:${project.id}`, target: projectTarget(project) }))
  )
])

function companyOf(companyId: CompanyId): Company | null {
  return companies.value.find((company) => company.id === companyId) ?? null
}

function isActive(key: string): boolean {
  return entries.value[cursor.value]?.key === key
}

function isCurrentEntry(entry: Entry): boolean {
  if (entry.target.kind === 'all') return everyone.value
  return entry.target.kind === 'project' && entry.target.projectId === props.projectId
}

async function open(): Promise<void> {
  query.value = ''
  isOpen.value = true
  cursor.value = Math.max(0, entries.value.findIndex(isCurrentEntry))
  await nextTick()
  search.value?.focus()
}

function close(returnFocus: boolean): void {
  isOpen.value = false
  if (returnFocus) trigger.value?.focus()
}

function toggle(): void {
  if (isOpen.value) close(false)
  else void open()
}

function pick(target: ScopeTarget): void {
  if (target.kind === 'project') recent.remember(target.projectId)
  emit('select', target)
  close(true)
}

function openPage(projectId: ProjectId): void {
  emit('open-page', projectId)
  close(false)
}

function moveCursor(step: number): void {
  if (entries.value.length === 0) return
  cursor.value = (cursor.value + step + entries.value.length) % entries.value.length
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    event.stopPropagation()
    close(true)
  } else if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
    event.preventDefault()
    moveCursor(event.key === 'ArrowDown' ? 1 : -1)
  } else if (event.key === 'Enter') {
    const entry = entries.value[cursor.value]
    if (!entry) return
    event.preventDefault()
    pick(entry.target)
  }
}

watch(query, () => {
  cursor.value = 0
})

watch(cursor, async () => {
  await nextTick()
  list.value?.querySelector<HTMLElement>('[data-active="true"]')?.scrollIntoView?.({ block: 'nearest' })
})

function edit(draft: RecordDraft): void {
  editing.value = draft
  isOpen.value = false
}

function newCompany(): void {
  edit(companyDraft())
}

function editCompany(companyId: CompanyId): void {
  const company = companyOf(companyId)
  if (company) edit(companyDraft(company))
}

function newProject(): void {
  const target = markedCompany.value ?? companies.value[0]
  if (target) edit(projectDraft(target.id))
}
</script>

<template>
  <div class="relative" data-testid="scope-switcher">
    <button
      ref="trigger"
      type="button"
      data-testid="scope-trigger"
      class="focus-ring -ml-2.5 flex h-10 cursor-pointer items-center gap-2.5 rounded-card px-2.5 text-left transition-colors duration-(--motion-quick) hover:bg-surface-raised"
      :class="isOpen && 'bg-surface-raised'"
      aria-haspopup="listbox"
      :aria-expanded="isOpen"
      :aria-label="isWork ? 'Choose what Work shows' : 'Switch project'"
      @click="toggle"
    >
      <span v-if="markedCompany" class="relative shrink-0" data-testid="scope-trigger-mark">
        <CompanyMark :company-id="markedCompany.id" :name="markedCompany.name" :image-version="markedCompany.logoImageVersion" :size="24" />
        <span
          v-if="markedProject"
          class="absolute -bottom-1 -right-1 rounded-key"
          :style="{ boxShadow: '0 0 0 1.5px var(--color-canvas)' }"
          data-testid="scope-trigger-project-mark"
        >
          <ProjectMark :project-id="markedProject.id" :icon="markedProject.icon" :image-version="markedProject.iconImageVersion" :size="14" />
        </span>
      </span>
      <span v-else class="mx-1 size-2.5 shrink-0 rotate-45 rounded-key bg-accent" aria-hidden="true" />
      <span class="flex min-w-0 items-baseline gap-2 text-heading">
        <template v-if="markedProject && markedCompany">
          <span class="shrink-0 text-text-subtle">{{ markedCompany.name }}</span>
          <span class="shrink-0 text-border-strong" aria-hidden="true">/</span>
          <span class="truncate font-medium text-text" data-testid="scope-trigger-title">{{ markedProject.title }}</span>
        </template>
        <span v-else class="truncate font-medium text-text" data-testid="scope-trigger-title">{{ markedCompany?.name ?? 'All work' }}</span>
      </span>
      <svg
        class="size-3.5 shrink-0 text-text-subtle transition-transform duration-(--motion-move)"
        :class="isOpen && 'rotate-180 text-text'"
        viewBox="0 0 16 16"
        fill="none"
        aria-hidden="true"
      >
        <path d="m4 6.5 4 4 4-4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
      </svg>
    </button>

    <div
      v-if="isOpen"
      class="rise absolute left-0 top-[calc(100%+6px)] z-(--z-overlay) flex max-h-[min(70vh,37.5rem)] w-[24rem] max-w-[calc(100vw-2rem)] flex-col overflow-hidden rounded-card border border-border-strong bg-surface-raised shadow-(--shadow-lift)"
      role="dialog"
      aria-label="Projects"
      data-testid="scope-popover"
      @keydown="onKeydown"
    >
      <label class="flex h-11 shrink-0 items-center gap-3 border-b border-border px-4 text-text-subtle">
        <svg class="size-4 shrink-0" viewBox="0 0 16 16" fill="none" aria-hidden="true">
          <circle cx="7" cy="7" r="4.5" stroke="currentColor" stroke-width="1.5" />
          <path d="m10.5 10.5 3.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
        <input
          ref="search"
          v-model="query"
          type="text"
          autocomplete="off"
          spellcheck="false"
          placeholder="Find a project"
          aria-label="Find a project"
          class="min-w-0 flex-1 bg-transparent text-body text-text outline-none placeholder:text-text-subtle"
          data-testid="switcher-search"
        />
      </label>

      <div ref="list" class="min-h-0 flex-1 overflow-y-auto p-2" role="listbox" aria-label="Projects">
        <div
          v-if="showsAll"
          class="flex items-center rounded-control"
          :class="isActive('all') && 'bg-surface-hover'"
          :data-active="isActive('all')"
        >
          <button
            type="button"
            role="option"
            class="focus-ring flex h-10 min-w-0 flex-1 cursor-pointer items-center gap-3 rounded-control px-3 text-left"
            :aria-selected="everyone"
            data-testid="scope-all"
            @click="pick({ kind: 'all' })"
          >
            <span class="mx-1 size-2.5 shrink-0 rotate-45 rounded-key bg-accent" aria-hidden="true" />
            <span class="min-w-0 flex-1 text-body" :class="everyone ? 'text-accent-strong' : 'text-text'">All work</span>
            <span class="shrink-0 text-small tabular-nums text-text-subtle">{{ tasks.length }}</span>
          </button>
        </div>

        <template v-if="sections.recent.length > 0">
          <p class="px-3 pb-1 pt-4 text-caption font-medium text-text-subtle first:pt-2">Recent</p>
          <ScopeProjectOption
            v-for="project in sections.recent"
            :key="`recent:${project.id}`"
            :project="project"
            :active="isActive(`recent:${project.id}`)"
            :current="project.id === projectId"
            :can-open-page="isWork"
            @pick="pick(projectTarget(project))"
            @open-page="openPage(project.id)"
          />
        </template>

        <template v-for="group in sections.companies" :key="group.companyId">
          <div v-if="isWork" class="group/company mt-3 flex items-center gap-1">
            <button
              type="button"
              role="option"
              class="focus-ring flex h-8 min-w-0 flex-1 cursor-pointer items-center gap-2 rounded-control px-3 text-left text-caption font-medium transition-colors duration-(--motion-quick)"
              :class="companyId === group.companyId && projectId === null ? 'text-accent-strong' : 'text-text-subtle hover:text-text'"
              :aria-selected="companyId === group.companyId && projectId === null"
              data-testid="scope-company"
              @click="pick({ kind: 'company', companyId: group.companyId })"
            >
              <CompanyMark
                :company-id="group.companyId"
                :name="group.companyName"
                :image-version="companyOf(group.companyId)?.logoImageVersion ?? null"
                :size="16"
              />
              <span class="min-w-0 flex-1 truncate">{{ group.companyName }}</span>
              <span class="shrink-0 font-normal tabular-nums">{{ companyOf(group.companyId)?.taskCount ?? 0 }}</span>
            </button>
            <button
              v-if="companyOf(group.companyId)"
              type="button"
              class="focus-ring mr-1 grid size-6 shrink-0 cursor-pointer place-items-center rounded-key text-text-subtle opacity-0 transition-opacity hover:bg-surface hover:text-accent focus-visible:opacity-100 group-hover/company:opacity-100"
              :aria-label="`Edit ${group.companyName}`"
              data-testid="edit-company"
              @click="editCompany(group.companyId)"
            >
              <svg class="size-3.5" viewBox="0 0 24 24" fill="none" aria-hidden="true">
                <path d="M4 20h4L20 8l-4-4L4 16v4z" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round" />
              </svg>
            </button>
          </div>
          <p v-else class="mt-3 flex h-8 items-center gap-2 px-3 text-caption font-medium text-text-subtle">
            <CompanyMark
              :company-id="group.companyId"
              :name="group.companyName"
              :image-version="companyOf(group.companyId)?.logoImageVersion ?? null"
              :size="16"
            />
            {{ group.companyName }}
          </p>
          <ScopeProjectOption
            v-for="project in group.projects"
            :key="`list:${project.id}`"
            :project="project"
            :active="isActive(`list:${project.id}`)"
            :current="project.id === projectId"
            :can-open-page="isWork"
            @pick="pick(projectTarget(project))"
            @open-page="openPage(project.id)"
          />
        </template>

        <p v-if="!hasMatch" class="px-4 py-6 text-center text-small text-text-subtle" data-testid="switcher-empty">
          {{ companies.length === 0 ? 'Start with a company: it holds the projects, and those hold the work.' : `No project matches “${query.trim()}”.` }}
        </p>
      </div>

      <div class="flex shrink-0 gap-1 border-t border-border p-2">
        <button
          type="button"
          class="focus-ring flex h-8 cursor-pointer items-center gap-2 rounded-control px-3 text-small text-text-muted transition-colors duration-(--motion-quick) hover:bg-surface-hover hover:text-text"
          data-testid="new-project"
          :disabled="companies.length === 0"
          @click="newProject"
        >
          <span class="text-accent" aria-hidden="true">+</span> New project
        </button>
        <button
          type="button"
          class="focus-ring flex h-8 cursor-pointer items-center gap-2 rounded-control px-3 text-small text-text-muted transition-colors duration-(--motion-quick) hover:bg-surface-hover hover:text-text"
          data-testid="new-company"
          @click="newCompany"
        >
          <span class="text-accent" aria-hidden="true">+</span> New company
        </button>
      </div>
    </div>

    <button v-if="isOpen" type="button" class="fixed inset-0 z-(--z-sticky) cursor-default" tabindex="-1" aria-hidden="true" @click="close(false)" />

    <Transition name="dialog">
      <RecordDialog v-if="editing" :draft="editing" @close="editing = null" />
    </Transition>
  </div>
</template>
