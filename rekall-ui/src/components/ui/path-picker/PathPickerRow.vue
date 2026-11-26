<script setup lang="ts">
import { computed } from 'vue'
import { splitByRanges, type PathRow } from '@/common/path/path-query'
import FileGlyph from '@/components/ui/FileGlyph.vue'
import { PICKER_MODES, type PickerMode } from '@/components/ui/path-picker/picker-mode'
import PictureThumbnail from '@/features/picture/components/PictureThumbnail.vue'
import { sourceThumbnailUrl } from '@/features/picture/api/pictures.api'

/** One folder or file of the folder in view, with the letters the filter matched lit. A picture to choose shows itself. */
const props = defineProps<{
  id: string
  row: PathRow
  highlighted: boolean
  mode: PickerMode
}>()

defineEmits<{ hover: []; activate: []; insert: [] }>()

const showsPictures = computed(() => props.mode === 'image')
</script>

<template>
  <li
    :id="id"
    role="option"
    :aria-selected="highlighted"
    class="path-row group/row flex cursor-pointer items-center gap-2.5 rounded-control pr-1.5"
    :class="[
      showsPictures ? 'h-11 pl-1.5' : 'h-[30px] pl-3.5',
      highlighted ? 'selected-row' : 'hover:bg-surface-hover',
      row.entry.hidden && 'opacity-60'
    ]"
    :data-directory="row.entry.directory"
    data-testid="path-picker-row"
    @mousemove="$emit('hover')"
    @click="$emit('activate')"
  >
    <PictureThumbnail v-if="showsPictures && !row.entry.directory" :src="sourceThumbnailUrl(row.entry.path)" class="size-9 shrink-0" />
    <!-- Beside thumbnails a folder's glyph takes the same box, so the names stay in one column. -->
    <span v-else :class="showsPictures ? 'grid size-9 shrink-0 place-items-center' : 'contents'">
      <svg
        v-if="row.entry.directory"
        class="size-3.5 shrink-0"
        :class="highlighted ? 'text-accent' : 'text-text-muted'"
        viewBox="0 0 16 16"
        aria-hidden="true"
      >
        <path
          d="M1.75 4.25c0-.55.45-1 1-1h3.1l1.4 1.5h6c.55 0 1 .45 1 1v6.5c0 .55-.45 1-1 1H2.75c-.55 0-1-.45-1-1v-8Z"
          fill="currentColor"
          fill-opacity="0.16"
          stroke="currentColor"
          stroke-width="1.25"
          stroke-linejoin="round"
        />
      </svg>
      <FileGlyph v-else />
    </span>

    <span
      class="min-w-0 flex-1 truncate font-mono text-small"
      :class="highlighted ? 'text-text' : 'text-text-muted'"
      :title="row.entry.name"
    >
      <template v-for="(part, partIndex) in splitByRanges(row.entry.name, row.ranges)" :key="partIndex">
        <mark v-if="part.matched" class="path-match">{{ part.text }}</mark>
        <template v-else>{{ part.text }}</template>
      </template>
    </span>

    <!-- On the highlighted folder, both of its moves, since a click only does one. -->
    <span
      v-if="row.entry.directory && highlighted && PICKER_MODES[mode].choosesFolders"
      class="flex shrink-0 items-center gap-1 text-micro text-text-subtle"
    >
      <button
        type="button"
        tabindex="-1"
        class="rounded-control px-1.5 py-0.5 transition-colors hover:bg-accent-soft hover:text-accent"
        data-testid="path-picker-insert-folder"
        @click.stop="$emit('insert')"
      >
        <kbd>↵</kbd> insert
      </button>
    </span>
    <svg
      v-if="row.entry.directory"
      class="size-3 shrink-0 transition-colors"
      :class="highlighted ? 'text-text-muted' : 'text-text-subtle'"
      viewBox="0 0 12 12"
      fill="none"
      aria-hidden="true"
    >
      <path d="m4.5 2.75 3.25 3.25-3.25 3.25" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
  </li>
</template>

<style scoped>
/* The matched letters: lit, not boxed, so a scattered match still reads as one word. */
.path-match {
  background: none;
  color: var(--color-accent-strong);
  font-weight: 600;
}
</style>
