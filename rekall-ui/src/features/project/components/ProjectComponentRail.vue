<script setup lang="ts">
import { computed, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import { identityHue } from '@/common/identity'
import { folderName } from '@/features/project/lib/component-folder'
import { componentBranches, componentMatches } from '@/features/project/lib/component-tree'
import type { ProjectComponent } from '@/features/project/model/project-component'
import type { ProjectComponentId } from '@/model/branded'

/**
 * The project's components as an index beside the one open, grouped under the folder that holds
 * them, with the key that has a session scan the folder for more.
 */
const props = defineProps<{
  components: readonly ProjectComponent[]
  selectedId: ProjectComponentId | null
  repoFolder: string | null
  loading: boolean
  scanning: boolean
}>()

const emit = defineEmits<{ select: [id: ProjectComponentId]; scan: []; add: [] }>()

const FILTER_FROM = 7

const query = ref('')

const branches = computed(() =>
  componentBranches(
    props.components.filter((component) => componentMatches(component, query.value)),
    props.repoFolder
  )
)

function shownPath(component: ProjectComponent, path: string): string | null {
  return folderName(path) === component.name ? null : path
}
</script>

<template>
  <nav class="flex min-h-0 flex-col gap-2 overflow-y-auto" aria-label="Components" data-testid="project-component-rail">
    <div class="flex items-center gap-1 pl-4 pr-1">
      <p class="eyebrow">Components<span v-if="components.length" class="ml-1.5 text-text-subtle">{{ components.length }}</span></p>
      <span class="flex-1" />
      <button
        type="button"
        class="key-quiet focus-ring grid size-6 place-items-center rounded-key"
        aria-label="Add a component by hand"
        title="Add a component by hand"
        data-testid="rail-add-component"
        @click="emit('add')"
      >
        <svg class="size-3.5" viewBox="0 0 14 14" fill="none" aria-hidden="true">
          <path d="M7 2.5v9M2.5 7h9" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
      </button>
    </div>

    <div class="px-1">
      <AppButton
        class="w-full"
        size="sm"
        :loading="scanning"
        :disabled="!repoFolder"
        :title="repoFolder ? 'A session reads the project’s folder and writes each part of it as a component' : 'Set the project folder first'"
        data-testid="scan-components"
        @click="emit('scan')"
      >
        <svg class="size-3.5" viewBox="0 0 14 14" fill="none" aria-hidden="true">
          <path d="M7 1.5 8.2 5.8 12.5 7 8.2 8.2 7 12.5 5.8 8.2 1.5 7 5.8 5.8Z" stroke="currentColor" stroke-width="1.3" stroke-linejoin="round" />
        </svg>
        Scan folder
      </AppButton>
    </div>

    <div v-if="components.length >= FILTER_FROM" class="px-1">
      <label class="sr-only" for="components-filter">Filter components</label>
      <input
        id="components-filter"
        v-model="query"
        type="search"
        autocomplete="off"
        spellcheck="false"
        placeholder="Filter"
        class="field text-text h-8 w-full rounded-control px-2.5 text-small"
        data-testid="components-filter"
      />
    </div>

    <div v-if="loading && !components.length" class="flex flex-col gap-1.5 px-1 pt-1" role="status" aria-live="polite" aria-busy="true">
      <span class="sr-only">Loading components</span>
      <div v-for="row in 4" :key="row" class="skeleton h-11 rounded-control" />
    </div>

    <p v-else-if="query.trim() && !branches.length" class="px-4 pt-1 text-small text-text-subtle" data-testid="components-no-match">
      No component matches “{{ query.trim() }}”.
    </p>

    <ul v-else class="flex flex-col gap-3" data-testid="components-tree">
      <li v-for="branch in branches" :key="branch.folder ?? 'outside'">
        <p
          v-if="branch.folder !== ''"
          class="mb-1 flex h-6 items-center gap-2 pl-4 font-mono text-caption text-text-subtle"
          data-testid="components-folder"
        >
          <span class="truncate">{{ branch.folder ?? 'Outside the project folder' }}</span>
          <span class="shrink-0">{{ branch.leaves.length }}</span>
        </p>
        <ul class="flex flex-col gap-0.5">
          <li v-for="leaf in branch.leaves" :key="leaf.component.id">
            <button
              type="button"
              class="focus-ring flex w-full cursor-pointer items-start gap-2.5 rounded-control py-2 pl-3 pr-3 text-left transition-colors"
              :class="selectedId === leaf.component.id ? 'selected-row' : 'hover:bg-surface-raised'"
              :style="{ '--select-tint': identityHue(leaf.component.id).base }"
              :aria-current="selectedId === leaf.component.id ? 'page' : undefined"
              :data-testid="`component-row-${leaf.component.id}`"
              @click="emit('select', leaf.component.id)"
            >
              <span class="mt-1.5 size-1.5 shrink-0 rounded-full" :style="{ background: identityHue(leaf.component.id).base }" aria-hidden="true" />
              <span class="min-w-0 flex-1">
                <span class="block truncate text-body text-text" data-testid="component-row-name">{{ leaf.component.name }}</span>
                <span v-if="shownPath(leaf.component, leaf.path)" class="block truncate font-mono text-micro text-text-subtle" dir="rtl">
                  <bdo dir="ltr">{{ shownPath(leaf.component, leaf.path) }}</bdo>
                </span>
                <span class="line-clamp-2 text-caption leading-snug text-text-muted">
                  {{ leaf.component.summary || 'No summary yet' }}
                </span>
              </span>
            </button>
          </li>
        </ul>
      </li>
    </ul>

    <p class="mt-auto border-t border-border px-4 pt-3 text-caption leading-relaxed text-text-subtle">
      A context lists each summary. A session reads a whole description with
      <code class="text-text-muted">rekall_read_components</code>.
    </p>
  </nav>
</template>
