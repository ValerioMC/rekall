<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { relativeTime } from '@/common/format/relative-time'
import { WRAPUP_AUTHOR_LABEL } from '@/features/wrapup/model/wrapup'
import { useWrapupVersionsStore } from '@/features/wrapup/store/wrapup-versions.store'
import type { WrapupVersion, WrapupVersionDetail } from '@/features/wrapup/model/wrapup-version'
import type { TaskId } from '@/model/branded'

/**
 * The earlier texts of one task's wrapup. Restoring one writes it again as a new wrapup by hand,
 * so what it replaces is kept as a version too.
 */
const props = defineProps<{ taskId: TaskId; wrapupUpdatedAt: string | null }>()
const emit = defineEmits<{ restored: [] }>()

const store = useWrapupVersionsStore()
const { run } = useAsyncAction()

const versions = computed(() => store.versionsOf(props.taskId))
const chosen = ref<WrapupVersion | null>(null)
const chosenText = ref<WrapupVersionDetail | null>(null)

watch(
  () => [props.taskId, props.wrapupUpdatedAt],
  () => void reload(),
  { immediate: true }
)

async function reload(): Promise<void> {
  chosen.value = null
  chosenText.value = null
  await run(() => store.load(props.taskId))
}

async function choose(version: WrapupVersion): Promise<void> {
  chosen.value = version
  chosenText.value = await run(() => store.read(props.taskId, version.id))
}

async function restore(): Promise<void> {
  const version = chosen.value
  if (!version) return
  const done = await run(() => store.restore(props.taskId, version.id), 'Version restored')
  if (done !== null) emit('restored')
}

function shortCommit(hash: string): string {
  return hash.slice(0, 7)
}
</script>

<template>
  <div class="flex min-h-0 flex-1" data-testid="wrapup-versions">
    <p v-if="!versions.length" class="px-9 py-12 text-body text-text-muted" data-testid="wrapup-versions-empty">
      No earlier text yet. Each time the wrapup changes, its text is kept here, the last 20.
    </p>

    <template v-else>
      <ul class="w-[300px] shrink-0 overflow-y-auto border-r border-border" aria-label="Versions">
        <li v-for="version in versions" :key="version.id">
          <button
            class="focus-ring flex w-full flex-col gap-0.5 border-b border-border px-4 py-2.5 text-left hover:bg-surface"
            :class="chosen?.id === version.id && 'bg-surface'"
            :aria-pressed="chosen?.id === version.id"
            data-testid="wrapup-version"
            @click="choose(version)"
          >
            <span class="flex items-center gap-2 text-caption text-text-muted">
              <span class="font-mono text-text">v{{ version.number }}</span>
              <span>{{ WRAPUP_AUTHOR_LABEL[version.writtenBy] }}</span>
              <span>&middot; {{ relativeTime(version.writtenAt) }}</span>
              <span class="ml-auto font-mono">{{ version.length }} chars</span>
            </span>
            <span class="line-clamp-2 text-small text-text-muted">{{ version.preview }}</span>
            <span v-if="version.commitHash" class="font-mono text-micro text-text-subtle">
              at {{ shortCommit(version.commitHash) }}
            </span>
          </button>
        </li>
      </ul>

      <div class="flex min-h-0 min-w-0 flex-1 flex-col">
        <p v-if="!chosenText" class="px-9 py-12 text-body text-text-muted">Pick a version to read it.</p>
        <template v-else>
          <div class="flex shrink-0 items-center gap-3 border-b border-border bg-surface px-5 py-2.5">
            <span class="text-caption text-text-muted">
              Version {{ chosenText.number }} &middot; {{ chosenText.writtenAt }}
            </span>
            <AppButton class="ml-auto" size="sm" data-testid="restore-wrapup-version" @click="restore">
              Restore this version
            </AppButton>
          </div>
          <div class="min-h-0 flex-1 overflow-y-auto p-4">
            <AppMarkdownEditor :model-value="chosenText.bodyMarkdown" readonly />
          </div>
        </template>
      </div>
    </template>
  </div>
</template>
