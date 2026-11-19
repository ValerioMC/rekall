<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { changeBlocks } from '@/features/commits/lib/change-blocks'
import { buildChangeTree, folderIdsAbove, visibleRows, type TreeFolder, type TreeNode, type TreeRow } from '@/features/commits/lib/change-tree'
import type { RepositoryRef, ReviewFile } from '@/features/commits/lib/change-sources'

/**
 * The files of the task as a folder tree you can open and close, with the lines each changed in the
 * scope, under a heading per repository when the task changed more than one. A folder holding only
 * another folder reads as one element. Files the scope does not touch stay listed, quietly, so the
 * tree does not move when the scope does. Files are named by `ReviewFile.id`.
 */
const props = defineProps<{
  files: readonly ReviewFile[]
  inScope: ReadonlyMap<string, ReviewFile>
  viewed: ReadonlySet<string>
  current: string | null
}>()

const emit = defineEmits<{ select: [fileId: string] }>()

interface Section {
  readonly repository: RepositoryRef
  readonly tree: readonly TreeNode[]
  readonly files: readonly ReviewFile[]
}

const INDENT = 14
const BASE_INDENT = 12
const BLOCK_CLASS = { added: 'bg-safe', removed: 'bg-danger', none: 'bg-border-strong' } as const

const closed = ref<ReadonlySet<string>>(new Set())
const nav = ref<HTMLElement | null>(null)

const sections = computed<readonly Section[]>(() => {
  const byRepository = new Map<string, { repository: RepositoryRef; files: ReviewFile[] }>()
  for (const file of props.files) {
    const entry = byRepository.get(file.repository.id) ?? { repository: file.repository, files: [] }
    entry.files.push(file)
    byRepository.set(file.repository.id, entry)
  }
  return [...byRepository.values()].map(({ repository, files }) => ({
    repository,
    files,
    tree: buildChangeTree(repository.id, files)
  }))
})
const several = computed(() => sections.value.length > 1)
const viewedCount = computed(() => props.files.filter((file) => props.viewed.has(file.id)).length)
const viewedShare = computed(() => (props.files.length === 0 ? 0 : (viewedCount.value / props.files.length) * 100))

watch(
  () => props.current,
  (fileId) => {
    if (fileId === null) return
    const hidden = sections.value.flatMap((section) => folderIdsAbove(section.tree, fileId))
    if (hidden.some((id) => closed.value.has(id))) closed.value = new Set([...closed.value].filter((id) => !hidden.includes(id)))
  }
)

function rowsOf(section: Section): readonly TreeRow[] {
  return visibleRows(section.tree, closed.value)
}

function changedIn(files: readonly ReviewFile[]): readonly ReviewFile[] {
  return files.filter((file) => props.inScope.has(file.id))
}

function linesIn(files: readonly ReviewFile[]): { additions: number; deletions: number } {
  return changedIn(files).reduce(
    (sum, file) => ({ additions: sum.additions + file.additions, deletions: sum.deletions + file.deletions }),
    { additions: 0, deletions: 0 }
  )
}

function toggle(folder: TreeFolder): void {
  const next = new Set(closed.value)
  if (!next.delete(folder.id)) next.add(folder.id)
  closed.value = next
}

function indentOf(depth: number): string {
  return `${BASE_INDENT + depth * INDENT}px`
}

function onKeydown(event: KeyboardEvent): void {
  const rows = Array.from(nav.value?.querySelectorAll<HTMLElement>('[data-tree-row]') ?? [])
  const at = rows.indexOf(document.activeElement as HTMLElement)
  if (at < 0) return
  const row = rows[at]!
  const folderOpen = row.getAttribute('aria-expanded')
  if (event.key === 'ArrowDown') rows[Math.min(at + 1, rows.length - 1)]?.focus()
  else if (event.key === 'ArrowUp') rows[Math.max(at - 1, 0)]?.focus()
  else if (event.key === 'ArrowRight' && folderOpen === 'false') row.click()
  else if (event.key === 'ArrowLeft' && folderOpen === 'true') row.click()
  else return
  event.preventDefault()
}
</script>

<template>
  <nav ref="nav" class="min-h-0 overflow-y-auto border-r border-border bg-surface" aria-label="Files changed" data-testid="change-file-tree" @keydown="onKeydown">
    <div class="sticky top-0 z-10 border-b border-border bg-surface px-4 py-3">
      <p class="flex items-baseline justify-between text-small font-semibold text-text">
        Files
        <span class="text-caption font-normal text-text-subtle" data-testid="viewed-count">{{ viewedCount }} of {{ files.length }} viewed</span>
      </p>
      <div class="mt-2 h-1.5 overflow-hidden rounded-full bg-canvas shadow-well" role="img" :aria-label="`${viewedCount} of ${files.length} files viewed`">
        <div class="h-full rounded-full bg-text-muted transition-[width] duration-(--motion-move)" :style="{ width: `${viewedShare}%` }" />
      </div>
    </div>
    <section v-for="section in sections" :key="section.repository.id" :data-repository="section.repository.id" data-testid="change-tree-repository">
      <p v-if="several" class="flex h-9 items-center gap-2 border-b border-border bg-canvas px-4" data-testid="change-tree-repository-name">
        <svg class="size-3.5 shrink-0 text-text-subtle" viewBox="0 0 14 14" fill="none" aria-hidden="true">
          <path d="M3 2.5h6.2A1.8 1.8 0 0 1 11 4.3v7.2H4.8A1.8 1.8 0 0 1 3 9.7V2.5Z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
          <path d="M3 9.7a1.8 1.8 0 0 1 1.8-1.8H11" stroke="currentColor" stroke-width="1.2" />
        </svg>
        <span class="min-w-0 flex-1 truncate text-small font-semibold text-text">{{ section.repository.name }}</span>
        <span class="shrink-0 text-caption text-text-subtle">{{ changedIn(section.files).length }} of {{ section.files.length }}</span>
      </p>
      <div role="tree" class="py-1">
        <template v-for="row in rowsOf(section)" :key="row.node.id">
          <button
            v-if="row.node.kind === 'folder'"
            type="button"
            role="treeitem"
            class="focus-ring flex h-7 w-full cursor-pointer items-center gap-1.5 pr-4 text-left transition-colors duration-(--motion-quick) hover:bg-surface-raised"
            :style="{ paddingLeft: indentOf(row.depth) }"
            :aria-expanded="row.open"
            :title="row.node.name"
            data-tree-row
            data-testid="change-folder-row"
            @click="toggle(row.node)"
          >
            <svg class="size-3 shrink-0 text-text-subtle transition-transform duration-(--motion-quick)" :class="row.open ? 'rotate-90' : ''" viewBox="0 0 12 12" fill="none" aria-hidden="true">
              <path d="m4.5 2.5 3.5 3.5-3.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
            <svg class="size-3.5 shrink-0 text-text-subtle" viewBox="0 0 14 14" fill="none" aria-hidden="true">
              <path d="M1.8 3.6c0-.5.4-.9.9-.9h2.9l1.3 1.4h4.4c.5 0 .9.4.9.9v5.5c0 .5-.4.9-.9.9H2.7a.9.9 0 0 1-.9-.9V3.6Z" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
            </svg>
            <span class="min-w-0 flex-1 truncate font-mono text-small" :class="changedIn(row.node.files).length > 0 ? 'text-text' : 'text-text-subtle'">{{ row.node.name }}</span>
            <span v-if="changedIn(row.node.files).length > 0" class="shrink-0 font-mono text-caption tabular-nums" data-testid="change-folder-lines">
              <span class="text-safe">+{{ linesIn(row.node.files).additions }}</span> <span class="text-danger">−{{ linesIn(row.node.files).deletions }}</span>
            </span>
          </button>
          <button
            v-else
            type="button"
            role="treeitem"
            class="focus-ring relative flex h-7 w-full cursor-pointer items-center gap-2 pr-4 text-left transition-colors duration-(--motion-quick) hover:bg-surface-raised"
            :class="current === row.node.id && inScope.has(row.node.id) ? 'bg-surface-raised before:absolute before:inset-y-1 before:left-0 before:w-0.5 before:rounded-full before:bg-accent' : ''"
            :style="{ paddingLeft: indentOf(row.depth + 1) }"
            :title="inScope.has(row.node.id) ? row.node.file.path : 'Not part of this change: opens it in all changes'"
            :aria-selected="current === row.node.id"
            data-tree-row
            data-testid="change-file-row"
            :data-in-scope="inScope.has(row.node.id)"
            @click="emit('select', row.node.id)"
          >
            <span
              class="grid size-3.5 shrink-0 place-items-center rounded-key border"
              :class="viewed.has(row.node.id) ? 'border-text-muted bg-surface-raised text-text' : 'border-border-field text-transparent'"
              aria-hidden="true"
            >
              <svg class="size-2.5" viewBox="0 0 10 10" fill="none"><path d="m2 5.2 2 2L8 3" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" /></svg>
            </span>
            <span class="min-w-0 flex-1 truncate font-mono text-small" :class="inScope.has(row.node.id) ? (viewed.has(row.node.id) ? 'text-text-muted' : 'text-text') : 'text-text-subtle'">{{ row.node.name }}</span>
            <template v-if="inScope.has(row.node.id)">
              <span class="flex shrink-0 gap-px" aria-hidden="true">
                <i v-for="(block, index) in changeBlocks(inScope.get(row.node.id)!.additions, inScope.get(row.node.id)!.deletions)" :key="index" class="h-2.25 w-1.25 rounded-key" :class="BLOCK_CLASS[block]" />
              </span>
              <span class="w-15 shrink-0 text-right font-mono text-caption tabular-nums">
                <span class="text-safe">+{{ inScope.get(row.node.id)!.additions }}</span> <span class="text-danger">−{{ inScope.get(row.node.id)!.deletions }}</span>
              </span>
            </template>
            <span v-else class="shrink-0 text-caption text-text-subtle">not in this</span>
          </button>
        </template>
      </div>
    </section>
  </nav>
</template>
