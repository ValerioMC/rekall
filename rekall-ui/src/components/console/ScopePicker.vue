<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { useRouter } from 'vue-router'
import RecordDialog from '@/components/console/RecordDialog.vue'
import { useConsoleStore } from '@/stores/console.store'
import { companyDraft, projectDraft } from '@/model/record-draft'
import type { RecordDraft } from '@/model/record-draft'
import type { Company, Project } from '@/model/catalog'
import type { CompanyId, ProjectId } from '@/model/branded'

/**
 * Four levels of hierarchy in one control.
 *
 * A select per level would put two or three controls above the list and make choosing a project
 * a two-step act. One popover showing companies with their projects nested is the same
 * information in one gesture, and it doubles as the map: you can see the whole shape of the
 * work without navigating into it.
 *
 * Editing lives here too, on the row of the thing being edited. A settings screen listing the
 * same tree a second time is the arrangement this replaces.
 */
const store = useConsoleStore()
const router = useRouter()
const {
  companies,
  projects,
  scopeCompany,
  scopeProject,
  scopePath,
  scopeAnchor,
  tasks,
  isLoading
} = storeToRefs(store)

const isOpen = ref(false)
const editing = ref<RecordDraft | null>(null)

const projectsOf = (companyId: CompanyId) =>
  projects.value.filter((project) => project.companyId === companyId)

const taskTotal = computed(() => tasks.value.length)

/** The company a new project would land in: the scope, or the only company there is. */
const targetCompany = computed(
  () => companies.value.find((c) => c.id === scopeCompany.value) ?? companies.value[0] ?? null
)

function pick(company: CompanyId | null, project: ProjectId | null = null): void {
  store.setScope(company, project)
  isOpen.value = false
}

function edit(draft: RecordDraft): void {
  editing.value = draft
  isOpen.value = false
}

function newCompany(): void {
  edit(companyDraft())
}

function newProject(): void {
  if (targetCompany.value) edit(projectDraft(targetCompany.value.id))
}

function editCompany(company: Company): void {
  edit(companyDraft(company))
}

function editProject(project: Project): void {
  isOpen.value = false
  router.push(`/projects/${project.id}`)
}

/** Every other overlay in the console closes on Escape; this one is no exception. */
function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape' && isOpen.value) {
    event.stopPropagation()
    isOpen.value = false
  }
}

defineExpose({ newCompany, newProject })
</script>

<template>
  <div class="relative" @keydown="onKeydown">
    <button
      data-testid="scope-trigger"
      class="focus-ring flex h-10 w-full items-center gap-2.5 rounded-[var(--radius-control)] border border-border-strong bg-canvas px-2.5 text-left transition-colors hover:border-accent"
      :class="isOpen && 'border-accent'"
      aria-haspopup="true"
      :aria-expanded="isOpen"
      @click="isOpen = !isOpen"
    >
      <span class="size-2 shrink-0 rotate-45 rounded-[1px] bg-accent" aria-hidden="true" />
      <span class="min-w-0 flex-1">
        <span class="block truncate text-[13px] text-text">
          <template v-if="scopePath.length === 0">All work</template>
          <template v-else>{{ scopePath[scopePath.length - 1] }}</template>
        </span>
        <span class="block truncate font-mono text-[10.5px] text-anchor">
          {{ isLoading ? 'Loading…' : scopeAnchor || `${taskTotal} tasks` }}
        </span>
      </span>
      <span class="shrink-0 text-[10px] text-text-subtle" aria-hidden="true">&#9662;</span>
    </button>

    <div
      v-if="isOpen"
      class="rise absolute inset-x-0 top-[calc(100%+6px)] z-(--z-overlay) max-h-[64vh] overflow-y-auto rounded-[var(--radius-card)] border border-border-strong bg-surface-raised p-1.5 shadow-modal"
    >
      <button
        class="focus-ring flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-left text-[12.5px] transition-colors hover:bg-surface"
        :class="scopeCompany === null ? 'text-accent' : 'text-text-muted'"
        @click="pick(null)"
      >
        <span class="flex-1">All work</span>
        <span class="font-mono text-[11px] text-text-subtle">{{ taskTotal }}</span>
      </button>

      <div class="my-1 h-px bg-border" />

      <template v-for="company in companies" :key="company.id">
        <div class="group/company flex items-center gap-0.5">
          <button
            data-testid="scope-company"
            class="focus-ring flex min-w-0 flex-1 items-center gap-2 rounded-md px-2.5 py-1.5 text-left transition-colors hover:bg-surface"
            :class="
              scopeCompany === company.id && scopeProject === null
                ? 'selected-row text-text'
                : 'text-text'
            "
            @click="pick(company.id)"
          >
            <span
              class="min-w-0 flex-1 truncate text-[11px] font-semibold uppercase tracking-[0.09em]"
            >
              {{ company.name }}
            </span>
            <span class="font-mono text-[11px] text-text-subtle">{{ company.taskCount }}</span>
          </button>
          <button
            data-testid="edit-company"
            class="focus-ring mr-1 grid size-6 shrink-0 place-items-center rounded text-text-subtle opacity-0 transition hover:bg-surface hover:text-accent focus-visible:opacity-100 group-hover/company:opacity-100"
            :aria-label="`Edit ${company.name}`"
            @click="editCompany(company)"
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

        <div
          v-for="project in projectsOf(company.id)"
          :key="project.id"
          class="group/project flex items-center gap-0.5"
        >
          <button
            data-testid="scope-project"
            class="focus-ring flex min-w-0 flex-1 items-center gap-2 rounded-md py-1.5 pl-6 pr-2 text-left transition-colors hover:bg-surface"
            :class="scopeProject === project.id ? 'selected-row' : ''"
            @click="pick(company.id, project.id)"
          >
            <span class="min-w-0 flex-1">
              <span
                class="block truncate text-[12.5px]"
                :class="scopeProject === project.id ? 'text-text' : 'text-text-muted'"
              >
                {{ project.title }}
              </span>
              <span class="mt-0.5 block truncate">
                <span class="anchor-chip px-1.5 py-px text-[9px] leading-[14px]">{{ project.anchor }}</span>
              </span>
            </span>
            <span
              v-if="project.status !== 'ACTIVE'"
              class="shrink-0 rounded border border-border-strong px-1 py-px text-[9.5px] uppercase tracking-[0.06em] text-text-subtle"
            >
              {{ project.status }}
            </span>
            <span class="shrink-0 font-mono text-[11px] text-text-subtle">
              {{ project.taskCount }}
            </span>
          </button>
          <button
            data-testid="edit-project"
            class="focus-ring mr-1 grid size-6 shrink-0 place-items-center rounded text-text-subtle opacity-0 transition hover:bg-surface hover:text-accent focus-visible:opacity-100 group-hover/project:opacity-100"
            :aria-label="`Edit ${project.title}`"
            @click="editProject(project)"
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

        <p
          v-if="!projectsOf(company.id).length"
          class="py-1 pl-6 pr-2.5 text-[11.5px] italic text-text-subtle"
        >
          no projects yet
        </p>
      </template>

      <p v-if="!companies.length" class="px-2.5 py-2 text-[12.5px] leading-relaxed text-text-subtle">
        Start with a company: it holds the projects, and those hold the work.
      </p>

      <div class="my-1 h-px bg-border" />

      <button
        data-testid="new-company"
        class="focus-ring flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-left text-[12.5px] text-text-muted transition-colors hover:bg-surface hover:text-text"
        @click="newCompany"
      >
        <span class="text-accent">+</span> New company
      </button>
      <button
        v-if="targetCompany"
        data-testid="new-project"
        class="focus-ring flex w-full items-center gap-2 rounded-md px-2.5 py-1.5 text-left text-[12.5px] text-text-muted transition-colors hover:bg-surface hover:text-text"
        @click="newProject"
      >
        <span class="text-accent">+</span>
        <span class="min-w-0 truncate">New project in {{ targetCompany.name }}</span>
      </button>
    </div>

    <!-- Clicking anywhere else closes the popover, without a listener on the document. -->
    <button
      v-if="isOpen"
      class="fixed inset-0 z-(--z-sticky) cursor-default"
      tabindex="-1"
      aria-hidden="true"
      @click="isOpen = false"
    />

    <RecordDialog v-if="editing" :draft="editing" @close="editing = null" />
  </div>
</template>
