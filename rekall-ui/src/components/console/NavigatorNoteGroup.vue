<script setup lang="ts">
/**
 * One scope in the notes navigator, built like a project card on the Tasks side: a header that
 * folds the scope away, its notes on a task under it on the scope's rail, and the loose ones on
 * their own shelf.
 */
import { computed } from 'vue'
import NavigatorNoteRow from '@/components/console/NavigatorNoteRow.vue'
import NavigatorFilingDrawer from '@/components/console/NavigatorFilingDrawer.vue'
import ProjectPageLink from '@/components/console/ProjectPageLink.vue'
import ProjectMark from '@/components/ui/ProjectMark.vue'
import { identityHue } from '@/common/identity'
import type { NoteGroup } from '@/common/catalog/note-groups'
import type { NoteShelves } from '@/composables/useNoteShelves'
import type { DocumentId } from '@/model/branded'

const props = defineProps<{
  group: NoteGroup
  shelves: NoteShelves
  selectedNoteId: DocumentId | null
}>()

defineEmits<{ select: [noteId: DocumentId] }>()

const projectScope = computed(() => (props.group.scope.kind === 'PROJECT' ? props.group.scope : null))
const tint = computed(() => (projectScope.value ? identityHue(projectScope.value.id) : null))
const expanded = computed(() => !props.shelves.collapsedGroups.has(props.group.key))
</script>

<template>
  <div class="px-2 pt-2.5 first:pt-1" data-testid="note-group">
    <div
      class="project-header-card flex items-stretch rounded-card"
      :style="tint ? { '--project-tint-soft': tint.soft } : undefined"
    >
      <button
        data-testid="note-group-toggle"
        class="focus-ring flex min-w-0 flex-1 items-start gap-2.5 py-2.5 pl-2.5 pr-2 text-left"
        :class="projectScope ? 'rounded-l-card' : 'rounded-card'"
        :aria-expanded="expanded"
        @click="shelves.toggleCollapsed(group.key)"
      >
        <svg
          class="mt-1.5 size-3 shrink-0 text-text-subtle transition-transform"
          :class="expanded && 'rotate-90'"
          viewBox="0 0 24 24"
          fill="none"
          aria-hidden="true"
        >
          <path d="M9 6l6 6-6 6" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" />
        </svg>

        <ProjectMark
          v-if="projectScope"
          class="mt-px"
          :project-id="projectScope.id"
          :icon="group.icon"
          :image-version="group.iconImageVersion"
          :size="24"
        />
        <span
          v-else
          class="mt-px grid size-6 shrink-0 place-items-center rounded-control border border-border-strong bg-surface text-text-muted"
          aria-hidden="true"
        >
          <svg v-if="group.scope.kind === 'COMPANY'" class="size-3.5" viewBox="0 0 12 12" fill="none">
            <path d="M2 10.5V2.5h5v8M7 5.5h3v5M1 10.5h10M3.6 4.5h1.8M3.6 6.8h1.8" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" />
          </svg>
          <svg v-else class="size-3.5" viewBox="0 0 12 12" fill="none">
            <circle cx="6" cy="6" r="4.6" stroke="currentColor" stroke-width="1.1" />
            <path d="M1.4 6h9.2M6 1.4c1.4 1.3 2 2.8 2 4.6s-.6 3.3-2 4.6c-1.4-1.3-2-2.8-2-4.6s.6-3.3 2-4.6Z" stroke="currentColor" stroke-width="1.1" />
          </svg>
        </span>

        <span class="min-w-0 flex-1">
          <span class="flex items-center gap-2">
            <span
              class="min-w-0 flex-1 truncate text-body font-semibold tracking-[-0.006em] text-text"
              data-testid="note-group-name"
            >
              {{ group.name }}
            </span>
            <span class="shrink-0 rounded-full bg-surface px-1.5 py-px font-mono text-micro tabular-nums text-text-subtle">
              {{ group.notes.length }}
            </span>
          </span>
          <span v-if="group.caption" class="mt-1 block truncate text-micro text-text-subtle">{{ group.caption }}</span>
        </span>
      </button>
      <ProjectPageLink v-if="projectScope" :project-id="projectScope.id" :project-title="group.name" />
    </div>

    <div
      v-show="expanded"
      class="ml-[15px] mt-1 flex flex-col gap-0.5 border-l-2 pl-2"
      :style="{ borderLeftColor: tint ? tint.line : 'var(--color-border-strong)' }"
    >
      <NavigatorNoteRow
        v-for="note in group.placed"
        :key="note.id"
        :note="note"
        :selected="note.id === selectedNoteId"
        :tint="tint"
        @select="$emit('select', note.id)"
      />

      <NavigatorFilingDrawer
        v-if="group.loose.length"
        kind="loose"
        :count="group.loose.length"
        :open="shelves.openLoose.has(group.key)"
        @toggle="shelves.toggleLoose(group.key)"
      >
        <NavigatorNoteRow
          v-for="note in group.loose"
          :key="note.id"
          :note="note"
          :selected="note.id === selectedNoteId"
          :tint="tint"
          @select="$emit('select', note.id)"
        />
      </NavigatorFilingDrawer>
    </div>
  </div>
</template>
