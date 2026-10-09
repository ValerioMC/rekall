<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import ProjectIcon from '@/components/ui/ProjectIcon.vue'
import ProjectMark from '@/components/ui/ProjectMark.vue'
import { groupProjectsByCompany, matchesProjectQuery } from '@/common/catalog/project-picker'
import { PROJECT_STATUS_LABEL } from '@/model/catalog'
import type { Project, ProjectStatus } from '@/model/catalog'

/**
 * Chooses the project a task belongs to. Closed, it is a card describing the current project; open,
 * a searchable list grouped by company unfolds in place, inside the form's own scroll area.
 */
const props = defineProps<{ modelValue: string; projects: readonly Project[]; id?: string }>()
const emit = defineEmits<{ 'update:modelValue': [value: string]; 'open-change': [open: boolean] }>()

const PROJECT_STATUS_TONE: Readonly<Record<ProjectStatus, 'accent' | 'neutral' | 'safe'>> = {
  ACTIVE: 'accent',
  PAUSED: 'neutral',
  DONE: 'safe',
  ARCHIVED: 'neutral'
}

const isOpen = ref(false)
const query = ref('')
const highlighted = ref(0)
const searchField = ref<HTMLInputElement | null>(null)
const list = ref<HTMLElement | null>(null)

const current = computed(() => props.projects.find((project) => project.id === props.modelValue) ?? null)
const matching = computed(() => props.projects.filter((project) => matchesProjectQuery(project, query.value)))
const groups = computed(() => groupProjectsByCompany(matching.value))
const flat = computed(() => groups.value.flatMap((group) => group.projects))

watch(isOpen, (open) => emit('open-change', open))
watch(query, () => {
  highlighted.value = 0
})

async function open(): Promise<void> {
  query.value = ''
  highlighted.value = Math.max(0, flat.value.findIndex((project) => project.id === props.modelValue))
  isOpen.value = true
  await nextTick()
  searchField.value?.focus()
  revealHighlighted()
}

function close(): void {
  isOpen.value = false
}

function pick(project: Project): void {
  emit('update:modelValue', project.id)
  close()
}

function revealHighlighted(): void {
  const row = list.value?.querySelector<HTMLElement>(`[data-index="${highlighted.value}"]`)
  row?.scrollIntoView?.({ block: 'nearest' })
}

function move(step: number): void {
  if (flat.value.length === 0) return
  highlighted.value = (highlighted.value + step + flat.value.length) % flat.value.length
  void nextTick(revealHighlighted)
}

function onSearchKeydown(event: KeyboardEvent): void {
  if (event.key === 'ArrowDown') {
    event.preventDefault()
    move(1)
  } else if (event.key === 'ArrowUp') {
    event.preventDefault()
    move(-1)
  } else if (event.key === 'Enter' && !event.metaKey && !event.ctrlKey) {
    event.preventDefault()
    event.stopPropagation()
    const target = flat.value[highlighted.value]
    if (target) pick(target)
  } else if (event.key === 'Escape') {
    event.preventDefault()
    close()
  }
}

function indexOf(project: Project): number {
  return flat.value.indexOf(project)
}
</script>

<template>
  <div class="overflow-hidden rounded-control border bg-canvas transition-colors" :class="isOpen ? 'border-accent' : 'border-border-strong'">
    <button
      :id="id"
      type="button"
      class="focus-ring flex w-full items-center gap-3 px-3 py-2.5 text-left transition-colors hover:bg-surface"
      data-testid="project-picker-trigger"
      aria-haspopup="listbox"
      :aria-expanded="isOpen"
      @click="isOpen ? close() : open()"
    >
      <ProjectMark
        v-if="current"
        :project-id="current.id"
        :icon="current.icon"
        :image-version="current.iconImageVersion"
        :size="36"
      />
      <span
        v-else
        class="grid size-9 shrink-0 place-items-center rounded-[22.37%] border border-border-strong text-text-subtle"
        aria-hidden="true"
      >
        <ProjectIcon icon="folder" :size="17" />
      </span>
      <span class="min-w-0 flex-1">
        <template v-if="current">
          <span class="flex items-center gap-2">
            <span class="truncate text-body font-medium text-text" data-testid="project-picker-title">{{ current.title }}</span>
            <AppBadge :tone="PROJECT_STATUS_TONE[current.status]" dot data-testid="record-parent-status">
              {{ PROJECT_STATUS_LABEL[current.status] }}
            </AppBadge>
          </span>
          <span class="mt-0.5 flex items-center gap-2 text-caption text-text-subtle">
            <span class="truncate">{{ current.companyName }}</span>
            <span class="anchor-chip shrink-0 px-1.5 py-px text-micro leading-[14px]">{{ current.anchor }}</span>
          </span>
        </template>
        <span v-else class="text-body text-text-subtle">Choose a project</span>
      </span>
      <span class="shrink-0 eyebrow text-micro text-accent">{{ isOpen ? 'Close' : 'Change' }}</span>
    </button>

    <div v-if="isOpen" class="border-t border-border">
      <div class="flex items-center gap-2 border-b border-border px-3">
        <svg class="size-3.5 shrink-0 text-text-subtle" viewBox="0 0 24 24" fill="none" aria-hidden="true">
          <circle cx="11" cy="11" r="6.5" stroke="currentColor" stroke-width="2" />
          <path d="m16 16 4.5 4.5" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
        </svg>
        <input
          ref="searchField"
          v-model="query"
          type="text"
          data-testid="project-picker-search"
          class="h-10 min-w-0 flex-1 bg-transparent text-body text-text outline-none placeholder:text-text-subtle"
          placeholder="Find by project, company or label"
          spellcheck="false"
          autocomplete="off"
          role="combobox"
          aria-expanded="true"
          aria-controls="project-picker-list"
          @keydown="onSearchKeydown"
        />
        <span class="shrink-0 font-mono text-micro text-text-subtle">{{ flat.length }}/{{ projects.length }}</span>
      </div>

      <div id="project-picker-list" ref="list" class="max-h-[220px] overflow-y-auto p-1.5" role="listbox">
        <section v-for="group in groups" :key="group.companyId" class="mb-1 last:mb-0">
          <p class="eyebrow px-2 pb-1 pt-1.5 text-micro">{{ group.companyName }}</p>
          <button
            v-for="project in group.projects"
            :key="project.id"
            type="button"
            role="option"
            data-testid="project-picker-option"
            :data-index="indexOf(project)"
            :aria-selected="project.id === modelValue"
            class="focus-ring flex w-full items-center gap-2.5 rounded-control px-2 py-1.5 text-left transition-colors"
            :class="[
              indexOf(project) === highlighted ? 'bg-surface-raised' : 'hover:bg-surface',
              project.id === modelValue && 'selected-row'
            ]"
            @mousemove="highlighted = indexOf(project)"
            @click="pick(project)"
          >
            <ProjectMark :project-id="project.id" :icon="project.icon" :image-version="project.iconImageVersion" :size="22" />
            <span class="min-w-0 flex-1">
              <span class="block truncate text-body" :class="project.id === modelValue ? 'text-text' : 'text-text-muted'">{{ project.title }}</span>
              <span class="block truncate font-mono text-micro text-anchor/70">{{ project.anchor }}</span>
            </span>
            <span v-if="project.status !== 'ACTIVE'" class="shrink-0 rounded-key border border-border-strong px-1 py-px text-micro text-text-subtle">
              {{ PROJECT_STATUS_LABEL[project.status] }}
            </span>
            <span class="shrink-0 font-mono text-caption text-text-subtle">{{ project.taskCount }}</span>
          </button>
        </section>
        <p v-if="flat.length === 0" class="px-2 py-3 text-small text-text-subtle" data-testid="project-picker-empty">
          No project matches “{{ query.trim() }}”.
        </p>
      </div>
    </div>
  </div>
</template>
