<script setup lang="ts">
import { computed } from 'vue'
import { changeBlocks } from '@/features/commits/lib/change-blocks'
import type { ReviewFile } from '@/features/commits/lib/change-sources'

/**
 * Every file of the task by folder, with the lines each changed in the scope. Files the scope does
 * not touch stay listed, quietly, so the list does not move when the scope does.
 */
const props = defineProps<{
  files: readonly ReviewFile[]
  inScope: ReadonlyMap<string, ReviewFile>
  viewed: ReadonlySet<string>
  current: string | null
}>()

const emit = defineEmits<{ select: [path: string] }>()

interface Folder {
  readonly name: string
  readonly files: readonly ReviewFile[]
}

const folders = computed<readonly Folder[]>(() => {
  const byFolder = new Map<string, ReviewFile[]>()
  for (const file of props.files) {
    const name = file.path.includes('/') ? file.path.slice(0, file.path.lastIndexOf('/')) : 'repository root'
    byFolder.set(name, [...(byFolder.get(name) ?? []), file])
  }
  return [...byFolder].map(([name, files]) => ({ name, files }))
})
const viewedCount = computed(() => props.files.filter((file) => props.viewed.has(file.path)).length)
const viewedShare = computed(() => (props.files.length === 0 ? 0 : (viewedCount.value / props.files.length) * 100))
const BLOCK_CLASS = { added: 'bg-safe', removed: 'bg-danger', none: 'bg-border-strong' } as const

function nameOf(path: string): string {
  return path.slice(path.lastIndexOf('/') + 1)
}
</script>

<template>
  <nav class="min-h-0 overflow-y-auto border-r border-border bg-surface" aria-label="Files changed" data-testid="change-file-tree">
    <div class="sticky top-0 z-10 border-b border-border bg-surface px-4 py-3">
      <p class="flex items-baseline justify-between text-small font-semibold text-text">
        Files
        <span class="text-caption font-normal text-text-subtle" data-testid="viewed-count">{{ viewedCount }} of {{ files.length }} viewed</span>
      </p>
      <div class="mt-2 h-1.5 overflow-hidden rounded-full bg-canvas shadow-well" role="img" :aria-label="`${viewedCount} of ${files.length} files viewed`">
        <div class="h-full rounded-full bg-text-muted transition-[width] duration-(--motion-move)" :style="{ width: `${viewedShare}%` }" />
      </div>
    </div>
    <template v-for="folder in folders" :key="folder.name">
      <p class="flex h-7.5 items-center px-4 font-mono text-caption text-text-subtle">
        <span class="truncate">{{ folder.name }}</span>
      </p>
      <button
        v-for="file in folder.files"
        :key="file.path"
        type="button"
        class="focus-ring relative flex min-h-8.5 w-full cursor-pointer items-center gap-2 pl-5 pr-4 text-left transition-colors duration-(--motion-quick) hover:bg-surface-raised"
        :class="current === file.path && inScope.has(file.path) ? 'bg-surface-raised before:absolute before:inset-y-1.5 before:left-0 before:w-0.5 before:rounded-full before:bg-accent' : ''"
        :title="inScope.has(file.path) ? file.path : 'Not part of this change: opens it in all changes'"
        data-testid="change-file-row"
        :data-in-scope="inScope.has(file.path)"
        @click="emit('select', file.path)"
      >
        <span
          class="grid size-3.5 shrink-0 place-items-center rounded-key border"
          :class="viewed.has(file.path) ? 'border-text-muted bg-surface-raised text-text' : 'border-border-field text-transparent'"
          aria-hidden="true"
        >
          <svg class="size-2.5" viewBox="0 0 10 10" fill="none"><path d="m2 5.2 2 2L8 3" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" /></svg>
        </span>
        <span class="min-w-0 flex-1 truncate font-mono text-small" :class="inScope.has(file.path) ? (viewed.has(file.path) ? 'text-text-muted' : 'text-text') : 'text-text-subtle'">{{ nameOf(file.path) }}</span>
        <template v-if="inScope.has(file.path)">
          <span class="flex shrink-0 gap-px" aria-hidden="true">
            <i v-for="(block, index) in changeBlocks(inScope.get(file.path)!.additions, inScope.get(file.path)!.deletions)" :key="index" class="h-2.25 w-1.25 rounded-key" :class="BLOCK_CLASS[block]" />
          </span>
          <span class="w-15 shrink-0 text-right font-mono text-caption tabular-nums">
            <span class="text-safe">+{{ inScope.get(file.path)!.additions }}</span> <span class="text-danger">−{{ inScope.get(file.path)!.deletions }}</span>
          </span>
        </template>
        <span v-else class="shrink-0 text-caption text-text-subtle">not in this</span>
      </button>
    </template>
  </nav>
</template>
