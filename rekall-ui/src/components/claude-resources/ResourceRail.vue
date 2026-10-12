<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppEmptyState from '@/components/ui/AppEmptyState.vue'
import AppSkeleton from '@/components/ui/AppSkeleton.vue'
import MemoryTree from './MemoryTree.vue'
import ResourceRow from './ResourceRow.vue'
import ReviewEntry from './ReviewEntry.vue'
import SkillShelves from './SkillShelves.vue'
import {
  RESOURCE_SECTIONS,
  SECTION_LABEL,
  matchesQuery,
  sectionCount,
  type ResourceCatalog,
  type ResourceSection,
  type SkillPlacement
} from '@/model/claude-resources'
import { buildMemoryTree, trailTo } from '@/model/memory-tree'
import { buildSkillShelves } from '@/model/skill-shelves'
import type { MemoryAnalysis } from '@/model/memory-analysis'
import type { Project } from '@/model/catalog'

const props = defineProps<{
  catalog: ResourceCatalog | null
  section: ResourceSection
  query: string
  selectedPath: string | null
  projects: readonly Project[]
  placements: readonly SkillPlacement[]
  analysis: MemoryAnalysis | null
  reviewing: boolean
}>()

const emit = defineEmits<{
  'update:section': [section: ResourceSection]
  'update:query': [query: string]
  select: [path: string]
  create: []
  review: []
  'move-skill': [path: string, group: string | null]
  'rename-group': [from: string, to: string]
}>()

const expanded = ref<ReadonlySet<string>>(new Set())

const memoryTree = computed(() => buildMemoryTree(props.catalog?.memoryGroups ?? [], props.projects, props.query))
const shelves = computed(() => buildSkillShelves(props.catalog?.skills ?? [], props.placements, props.query))

const flat = computed(() => {
  if (!props.catalog || props.section === 'memories' || props.section === 'skills') return []
  return props.catalog[props.section].filter((entry) => matchesQuery(entry, props.query))
})

const instructions = computed(() => props.catalog?.instructions ?? null)
const searching = computed(() => props.query.trim() !== '')
const nothingShown = computed(() => {
  if (props.section === 'memories') return memoryTree.value.length === 0
  if (props.section === 'skills') return shelves.value.length === 0
  return flat.value.length === 0
})

function isOpen(id: string): boolean {
  return searching.value || expanded.value.has(id)
}

function toggle(id: string): void {
  const next = new Set(expanded.value)
  if (!next.delete(id)) next.add(id)
  expanded.value = next
}

/** The groups holding the open memory, and those above them, are shown open: the selection is never hidden in a fold. */
watch(
  [() => props.selectedPath, memoryTree],
  ([path]) => {
    const trail = trailTo(path, memoryTree.value)
    if (trail.some((id) => !expanded.value.has(id))) expanded.value = new Set([...expanded.value, ...trail])
  },
  { immediate: true }
)

function moveTab(event: KeyboardEvent): void {
  const step = event.key === 'ArrowRight' ? 1 : event.key === 'ArrowLeft' ? -1 : 0
  if (step === 0) return
  event.preventDefault()
  const at = RESOURCE_SECTIONS.indexOf(props.section)
  const next = RESOURCE_SECTIONS[(at + step + RESOURCE_SECTIONS.length) % RESOURCE_SECTIONS.length]
  if (next) emit('update:section', next)
}
</script>

<template>
  <aside class="flex w-[336px] shrink-0 flex-col border-r border-border bg-surface" aria-label="Claude library">
    <div class="shrink-0 px-3 pt-3">
      <ResourceRow
        v-if="instructions"
        :summary="{ ...instructions, name: 'Global instructions', description: 'CLAUDE.md, loaded in every session' }"
        :selected="selectedPath === instructions.path"
        @select="emit('select', instructions.path)"
      />
      <div v-else-if="catalog" class="rounded-control border border-dashed border-border px-2.5 py-2 text-caption text-text-subtle">
        No CLAUDE.md yet.
        <button class="focus-ring text-accent hover:underline" @click="emit('select', 'CLAUDE.md')">Write one</button>
      </div>

      <ReviewEntry class="mt-1.5" :analysis="analysis" :active="reviewing" @open="emit('review')" />

      <div class="mt-3 flex gap-px rounded-control bg-canvas p-0.5 shadow-(--shadow-well)" role="tablist" aria-label="Kind of resource" @keydown="moveTab">
        <button
          v-for="tab in RESOURCE_SECTIONS"
          :id="`resource-tab-${tab}`"
          :key="tab"
          role="tab"
          class="focus-ring flex min-w-0 flex-1 items-center justify-center gap-1 rounded-key px-1.5 py-1 text-caption font-medium transition-colors"
          :class="section === tab ? 'bg-surface-raised text-text shadow-(--shadow-key)' : 'text-text-subtle hover:text-text'"
          :aria-selected="section === tab"
          :tabindex="section === tab ? 0 : -1"
          :data-testid="`resource-tab-${tab}`"
          @click="emit('update:section', tab)"
        >
          <span class="truncate">{{ SECTION_LABEL[tab] }}</span>
          <span v-if="catalog" class="font-mono text-micro tabular-nums text-text-subtle">{{ sectionCount(catalog, tab) }}</span>
        </button>
      </div>

      <div class="mt-2.5 flex items-center gap-2">
        <label class="sr-only" for="resource-search">Filter {{ SECTION_LABEL[section].toLowerCase() }}</label>
        <input
          id="resource-search"
          :value="query"
          type="search"
          autocomplete="off"
          spellcheck="false"
          class="field h-8 min-w-0 flex-1 rounded-control px-2.5 text-small text-text"
          :placeholder="`Filter ${SECTION_LABEL[section].toLowerCase()}`"
          data-testid="resource-search"
          @input="emit('update:query', ($event.target as HTMLInputElement).value)"
        />
        <AppButton size="sm" variant="secondary" data-testid="resource-new" @click="emit('create')">New</AppButton>
      </div>
    </div>

    <div
      class="mt-2 min-h-0 flex-1 overflow-y-auto px-3 pb-4"
      role="tabpanel"
      :aria-labelledby="`resource-tab-${section}`"
    >
      <AppSkeleton v-if="!catalog" variant="list" :rows="6" />

      <MemoryTree
        v-else-if="section === 'memories'"
        :nodes="memoryTree"
        :expanded="isOpen"
        :selected-path="selectedPath"
        @toggle="toggle"
        @select="emit('select', $event)"
      />

      <SkillShelves
        v-else-if="section === 'skills'"
        :shelves="shelves"
        :placements="placements"
        :selected-path="selectedPath"
        :searching="searching"
        @select="emit('select', $event)"
        @move="(path, group) => emit('move-skill', path, group)"
        @rename="(from, to) => emit('rename-group', from, to)"
      />

      <template v-else>
        <ResourceRow v-for="entry in flat" :key="entry.path" :summary="entry" :selected="selectedPath === entry.path" @select="emit('select', entry.path)" />
      </template>

      <AppEmptyState
        v-if="catalog && nothingShown"
        class="mt-2 py-8"
        :title="searching ? 'Nothing matches' : `No ${SECTION_LABEL[section].toLowerCase()} yet`"
        :description="searching ? 'Try a shorter filter.' : 'Create one with New.'"
      />
    </div>
  </aside>
</template>
