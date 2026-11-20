<script setup lang="ts">
import { computed } from 'vue'
import GitlabNamespaceGlyph from '@/features/connection/components/GitlabNamespaceGlyph.vue'
import { ancestorsOf, leafOf, splitOnMatch, type PathCursor } from '@/features/connection/lib/gitlab-path'
import type { GitlabNamespace } from '@/features/connection/model/connection'

/**
 * What sits under the group typed so far: its subgroups and projects as rows, the groups above it
 * as a breadcrumb to jump back to. A row of a group says how far it goes, a row of a project how
 * private it is.
 */
const props = defineProps<{
  id: string
  cursor: PathCursor
  rootPath: string | null
  rows: readonly GitlabNamespace[]
  activeIndex: number
  unavailable: string | null
}>()
const emit = defineEmits<{ choose: [row: GitlabNamespace]; jump: [groupPath: string] }>()

const scopeParent = computed(() => props.cursor.parent || props.rootPath || '')
const trail = computed(() => ancestorsOf(scopeParent.value))

function optionId(index: number): string {
  return `${props.id}-option-${index}`
}
</script>

<template>
  <div class="mt-1.5 overflow-hidden rounded-card border border-border-strong bg-surface-raised" data-testid="gitlab-path-list">
    <div class="flex items-center gap-0.5 border-b border-border px-2 py-1.5 text-caption text-text-subtle">
      <GitlabNamespaceGlyph kind="GROUP" class="mr-1" />
      <template v-for="(ancestor, index) in trail" :key="ancestor.path">
        <span v-if="index > 0" aria-hidden="true">/</span>
        <button
          type="button"
          class="focus-ring h-6 rounded-key px-1.5 font-mono text-caption text-text-muted hover:bg-surface-hover hover:text-text"
          @mousedown.prevent
          @click="emit('jump', ancestor.path)"
        >
          {{ ancestor.name }}
        </button>
      </template>
      <span class="ml-auto text-micro">Only what the token can read{{ rootPath ? ` under ${rootPath}` : '' }}</span>
    </div>

    <ul v-if="rows.length" :id="id" role="listbox" class="max-h-60 overflow-auto p-1">
      <li
        v-for="(row, index) in rows"
        :id="optionId(index)"
        :key="row.path"
        role="option"
        :aria-selected="index === activeIndex"
        class="flex min-h-(--spacing-row) cursor-pointer items-center gap-2.5 rounded-control px-2"
        :class="index === activeIndex ? 'bg-surface-hover' : 'hover:bg-surface-hover'"
        data-testid="gitlab-path-option"
        @mousedown.prevent
        @click="emit('choose', row)"
      >
        <GitlabNamespaceGlyph :kind="row.kind" :class="row.kind === 'GROUP' ? 'text-text-muted' : 'text-text-subtle'" />
        <span class="truncate font-mono text-small text-text-muted">
          <template v-for="(part, partIndex) in splitOnMatch(leafOf(row.path), cursor.fragment)" :key="partIndex">
            <b v-if="partIndex === 1" class="font-semibold text-text">{{ part }}</b>
            <template v-else>{{ part }}</template>
          </template>
        </span>
        <span class="ml-auto flex shrink-0 items-center gap-2 text-caption text-text-subtle">
          <template v-if="row.kind === 'GROUP'">Group</template>
          <template v-else>{{ row.visibility ?? 'Project' }}</template>
        </span>
      </li>
    </ul>
    <p v-else-if="unavailable" class="px-3 py-3.5 text-small text-text-muted" role="status" data-testid="gitlab-path-unavailable">
      The list is unavailable: {{ unavailable }}
    </p>
    <p v-else class="px-3 py-3.5 text-small text-text-muted" data-testid="gitlab-path-empty">
      Nothing under <span class="font-mono">{{ scopeParent || 'the instance' }}</span> matches
      <span class="font-mono">{{ cursor.fragment }}</span
      >.
    </p>

    <div class="flex gap-3.5 border-t border-border px-3 py-1.5 text-micro text-text-subtle">
      <span><kbd>↑↓</kbd> move</span>
      <span><kbd>Enter</kbd> choose</span>
      <span><kbd>Esc</kbd> close</span>
    </div>
  </div>
</template>
