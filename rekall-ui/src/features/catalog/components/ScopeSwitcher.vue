<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import CompanyMark from '@/components/ui/CompanyMark.vue'
import ProjectMark from '@/components/ui/ProjectMark.vue'
import RecordDialog from '@/features/catalog/components/RecordDialog.vue'
import ScopeProjectOption from '@/features/catalog/components/ScopeProjectOption.vue'
import { switcherSections } from '@/features/catalog/lib/switcher-sections'
import { companyDraft, projectDraft, type RecordDraft } from '@/features/catalog/model/record-draft'
import { usePinnedProjectsStore, PINNED_LIMIT } from '@/features/catalog/store/pinnedProjects.store'
import { useRecentProjectsStore } from '@/features/catalog/store/recentProjects.store'
import { useConsoleStore } from '@/stores/console.store'
import { useToastStore } from '@/stores/toast.store'
import type { Company, Project } from '@/features/catalog/model/catalog'
import type { ScopeTarget } from '@/features/catalog/model/scope-target'
import type { CompanyId, ProjectId } from '@/model/branded'

/**
 * The project in view and the list that changes it: pinned projects, recent ones, then every
 * company with its projects, folded except the open project's. As the title of a project page or as
 * the header key it moves between projects. The page decides what a choice does.
 */
const props = withDefaults(
  defineProps<{ companyId: CompanyId | null; projectId: ProjectId | null; variant?: 'title' | 'header' }>(),
  { variant: 'title' }
)

const emit = defineEmits<{ select: [target: ScopeTarget] }>()

interface Entry {
  readonly key: string
  readonly target: ScopeTarget
}

const store = useConsoleStore()
const recent = useRecentProjectsStore()
const pinned = usePinnedProjectsStore()
const toast = useToastStore()
const { companies, projects } = storeToRefs(store)

const isOpen = ref(false)
const query = ref('')
const cursor = ref(0)
const unfolded = ref<ReadonlySet<CompanyId>>(new Set())
const editing = ref<RecordDraft | null>(null)
const trigger = ref<HTMLButtonElement | null>(null)
const search = ref<HTMLInputElement | null>(null)
const list = ref<HTMLElement | null>(null)

const searching = computed(() => query.value.trim() !== '')
const markedProject = computed(() => projects.value.find((project) => project.id === props.projectId) ?? null)
const markedCompany = computed(() => {
  const companyId = markedProject.value?.companyId ?? props.companyId
  return companies.value.find((company) => company.id === companyId) ?? null
})
const isHeader = computed(() => props.variant === 'header')
const sections = computed(() => switcherSections(projects.value, recent.ids, query.value, props.projectId, pinned.ids))
const hasMatch = computed(
  () =>
    sections.value.pinned.length > 0 ||
    sections.value.recent.length > 0 ||
    sections.value.companies.length > 0
)

function projectTarget(project: Project): ScopeTarget {
  return { companyId: project.companyId, projectId: project.id }
}

function showsCompany(companyId: CompanyId): boolean {
  return searching.value || unfolded.value.has(companyId)
}

const entries = computed<Entry[]>(() => [
  ...sections.value.pinned.map((project) => ({ key: `pinned:${project.id}`, target: projectTarget(project) })),
  ...sections.value.recent.map((project) => ({ key: `recent:${project.id}`, target: projectTarget(project) })),
  ...sections.value.companies
    .filter((group) => showsCompany(group.companyId))
    .flatMap((group) => group.projects.map((project) => ({ key: `list:${project.id}`, target: projectTarget(project) })))
])

function companyOf(companyId: CompanyId): Company | null {
  return companies.value.find((company) => company.id === companyId) ?? null
}

function isActive(key: string): boolean {
  return entries.value[cursor.value]?.key === key
}

function isCurrentEntry(entry: Entry): boolean {
  return entry.target.projectId === props.projectId
}

function companiesToUnfold(): ReadonlySet<CompanyId> {
  const own = markedCompany.value?.id
  if (own) return new Set([own])
  const hasShortcuts = pinned.ids.length > 0 || recent.ids.length > 0
  return new Set(hasShortcuts ? [] : companies.value.map((company) => company.id))
}

function toggleCompany(companyId: CompanyId): void {
  const next = new Set(unfolded.value)
  if (!next.delete(companyId)) next.add(companyId)
  unfolded.value = next
}

function togglePin(project: Project): void {
  if (pinned.toggle(project.id) === 'full') {
    toast.notifyError(new Error(`Pin at most ${PINNED_LIMIT} projects. Unpin one first.`))
  }
}

async function open(): Promise<void> {
  query.value = ''
  unfolded.value = companiesToUnfold()
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
  recent.remember(target.projectId)
  emit('select', target)
  close(true)
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

function newProject(): void {
  const target = markedCompany.value ?? companies.value[0]
  if (target) edit(projectDraft(target.id))
}
</script>

<template>
  <div class="relative" :class="isHeader && 'min-w-0'" data-testid="scope-switcher">
    <button
      v-if="isHeader"
      ref="trigger"
      type="button"
      data-testid="scope-trigger"
      class="focus-ring flex h-8 w-full max-w-60 cursor-pointer items-center gap-2 rounded-control border border-border bg-surface px-2 text-left text-body font-medium text-text transition-colors duration-(--motion-quick) hover:bg-surface-raised"
      :class="isOpen && 'bg-surface-raised'"
      aria-haspopup="listbox"
      :aria-expanded="isOpen"
      aria-label="Switch project"
      @click="toggle"
    >
      <ProjectMark v-if="markedProject" :project-id="markedProject.id" :icon="markedProject.icon" :image-version="markedProject.iconImageVersion" :size="20" />
      <span v-else class="mx-1.5 size-2 shrink-0 rotate-45 rounded-key bg-accent" aria-hidden="true" />
      <span class="min-w-0 truncate" data-testid="scope-trigger-title">{{ markedProject?.title ?? 'Projects' }}</span>
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
    <button
      v-else
      ref="trigger"
      type="button"
      data-testid="scope-trigger"
      class="focus-ring -ml-2.5 flex h-10 cursor-pointer items-center gap-2.5 rounded-card px-2.5 text-left transition-colors duration-(--motion-quick) hover:bg-surface-raised"
      :class="isOpen && 'bg-surface-raised'"
      aria-haspopup="listbox"
      :aria-expanded="isOpen"
      aria-label="Switch project"
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
        <span v-else class="truncate font-medium text-text" data-testid="scope-trigger-title">{{ markedCompany?.name ?? 'Projects' }}</span>
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
        <template v-if="sections.pinned.length > 0">
          <p class="px-3 pb-1 pt-4 text-caption font-medium text-text-subtle first:pt-2" data-testid="scope-pinned-heading">Pinned</p>
          <ScopeProjectOption
            v-for="project in sections.pinned"
            :key="`pinned:${project.id}`"
            :project="project"
            :active="isActive(`pinned:${project.id}`)"
            :current="project.id === projectId"
            :pinned="true"
            @pick="pick(projectTarget(project))"
            @toggle-pin="togglePin(project)"
          />
        </template>

        <template v-if="sections.recent.length > 0">
          <p class="px-3 pb-1 pt-4 text-caption font-medium text-text-subtle first:pt-2">Recent</p>
          <ScopeProjectOption
            v-for="project in sections.recent"
            :key="`recent:${project.id}`"
            :project="project"
            :active="isActive(`recent:${project.id}`)"
            :current="project.id === projectId"
            :pinned="pinned.isPinned(project.id)"
            @pick="pick(projectTarget(project))"
            @toggle-pin="togglePin(project)"
          />
        </template>

        <template v-for="group in sections.companies" :key="group.companyId">
          <div class="group/company mt-3 flex items-center gap-1">
            <button
              type="button"
              class="focus-ring flex h-8 min-w-0 flex-1 cursor-pointer items-center gap-2 rounded-control px-2 text-left text-caption font-medium text-text-subtle transition-colors duration-(--motion-quick) hover:text-text disabled:cursor-default"
              :aria-label="`${showsCompany(group.companyId) ? 'Fold' : 'Unfold'} ${group.companyName}`"
              :aria-expanded="showsCompany(group.companyId)"
              :disabled="searching"
              data-testid="scope-company-fold"
              @click="toggleCompany(group.companyId)"
            >
              <svg class="size-3 shrink-0 transition-transform duration-(--motion-quick)" :class="showsCompany(group.companyId) && 'rotate-90'" viewBox="0 0 16 16" fill="none" aria-hidden="true">
                <path d="m6.5 4 4 4-4 4" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" />
              </svg>
              <CompanyMark
                :company-id="group.companyId"
                :name="group.companyName"
                :image-version="companyOf(group.companyId)?.logoImageVersion ?? null"
                :size="16"
              />
              <span class="min-w-0 flex-1 truncate">{{ group.companyName }}</span>
              <span class="shrink-0 font-normal tabular-nums">{{ group.projects.length }}</span>
            </button>
          </div>
          <template v-if="showsCompany(group.companyId)">
            <ScopeProjectOption
              v-for="project in group.projects"
              :key="`list:${project.id}`"
              :project="project"
              :active="isActive(`list:${project.id}`)"
              :current="project.id === projectId"
              :pinned="pinned.isPinned(project.id)"
                @pick="pick(projectTarget(project))"
              @toggle-pin="togglePin(project)"
              />
          </template>
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
