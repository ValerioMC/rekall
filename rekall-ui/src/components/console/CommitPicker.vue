<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'
import { fetchRecentCommits } from '@/api/commitReference.api'
import { relativeTime } from '@/common/format/relative-time'
import type { RecentCommit } from '@/model/commitReference'
import type { TaskId } from '@/model/branded'

/**
 * The recent log of the task's project folder, anchored under the button that opened it, so a
 * commit can be logged by hand: pick a row, or paste a hash the log does not reach. It only
 * chooses; the button that hosts it does the logging and closes it.
 */
const props = defineProps<{
  taskId: TaskId
  /** The element the panel hangs from: its right edge is the panel's right edge. */
  anchor: HTMLElement
  /** Hashes already logged against this task/step, marked so they are not picked twice. */
  loggedHashes: readonly string[]
  /** Whichever the host is doing right now, so the rows read as busy rather than dead. */
  busy: boolean
}>()

const emit = defineEmits<{ pick: [hash: string]; close: [] }>()

const HASH = /^[0-9a-fA-F]{4,40}$/

const commits = ref<RecentCommit[]>([])
const loading = ref(true)
const failure = ref<string | null>(null)
const pasted = ref('')
const panel = ref<HTMLElement | null>(null)
const pasteField = ref<HTMLInputElement | null>(null)
const position = ref<{ top: number; left: number; width: number }>({ top: 0, left: 0, width: 0 })

const WIDTH = 400
const GUTTER = 8

const pastedHash = computed(() => pasted.value.trim())
const pastedIsHash = computed(() => HASH.test(pastedHash.value))

function isLogged(hash: string): boolean {
  return props.loggedHashes.includes(hash)
}

function place(): void {
  const rect = props.anchor.getBoundingClientRect()
  const width = Math.min(WIDTH, window.innerWidth - 2 * GUTTER)
  const left = Math.max(GUTTER, Math.min(rect.right - width, window.innerWidth - width - GUTTER))
  position.value = { top: rect.bottom + 6, left, width }
}

async function load(): Promise<void> {
  loading.value = true
  failure.value = null
  try {
    commits.value = await fetchRecentCommits(props.taskId)
  } catch (caught) {
    failure.value = caught instanceof Error ? caught.message : 'Could not read the recent commits.'
  } finally {
    loading.value = false
  }
}

function pick(hash: string): void {
  if (props.busy) return
  emit('pick', hash)
}

function submitPasted(): void {
  if (!pastedIsHash.value) return
  pick(pastedHash.value)
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    event.stopPropagation()
    emit('close')
  }
}

function onPointerDown(event: PointerEvent): void {
  const target = event.target as Node | null
  if (!target) return
  if (panel.value?.contains(target) || props.anchor.contains(target)) return
  emit('close')
}

onMounted(async () => {
  place()
  window.addEventListener('keydown', onKeydown, true)
  window.addEventListener('pointerdown', onPointerDown, true)
  window.addEventListener('resize', place)
  window.addEventListener('scroll', place, true)
  void load()
  await nextTick()
  pasteField.value?.focus()
})

onUnmounted(() => {
  window.removeEventListener('keydown', onKeydown, true)
  window.removeEventListener('pointerdown', onPointerDown, true)
  window.removeEventListener('resize', place)
  window.removeEventListener('scroll', place, true)
})
</script>

<template>
  <Teleport to="body">
    <div
      ref="panel"
      class="commit-picker fixed z-(--z-overlay) flex max-h-[min(440px,calc(100vh-24px))] flex-col overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface shadow-lift"
      :style="{ top: `${position.top}px`, left: `${position.left}px`, width: `${position.width}px` }"
      role="dialog"
      aria-label="Pick a commit to log"
      data-testid="commit-picker"
    >
      <div class="flex items-center gap-2 border-b border-border px-3.5 py-2.5">
        <span class="h-2.5 w-[3px] shrink-0 rounded-full bg-accent" aria-hidden="true" />
        <span class="eyebrow">Recent commits</span>
        <span
          v-if="!loading && !failure"
          class="ml-auto font-mono text-[10.5px] tabular-nums text-text-subtle"
          data-testid="commit-picker-count"
        >
          {{ commits.length }}
        </span>
      </div>

      <div class="min-h-0 flex-1 overflow-y-auto py-1" data-testid="commit-picker-log">
        <p
          v-if="loading"
          class="px-3.5 py-3 text-[11.5px] text-text-subtle"
          data-testid="commit-picker-loading"
        >
          Reading the log…
        </p>
        <p
          v-else-if="failure"
          class="px-3.5 py-3 text-[11.5px] leading-relaxed text-danger"
          data-testid="commit-picker-failure"
        >
          {{ failure }}
        </p>
        <ol v-else class="relative" :class="busy && 'pointer-events-none opacity-60'">
          <li v-for="(commit, index) in commits" :key="commit.hash" class="relative">
            <button
              type="button"
              class="commit-row focus-ring group/row flex w-full items-start gap-2.5 px-3.5 py-1.5 text-left transition-colors hover:bg-surface-raised disabled:cursor-default disabled:hover:bg-transparent"
              :disabled="isLogged(commit.hash)"
              :data-hash="commit.hash"
              data-testid="commit-picker-row"
              @click="pick(commit.hash)"
            >
              <span class="relative mt-[5px] flex w-3 shrink-0 flex-col items-center" aria-hidden="true">
                <span
                  class="size-2 rounded-full border-[1.5px]"
                  :class="
                    isLogged(commit.hash)
                      ? 'border-accent bg-accent'
                      : index === 0
                        ? 'border-anchor bg-anchor/30'
                        : 'border-border-strong bg-surface group-hover/row:border-accent'
                  "
                />
                <span
                  v-if="index < commits.length - 1"
                  class="absolute top-2.5 h-[calc(100%+6px)] w-px bg-border"
                />
              </span>

              <span class="flex min-w-0 flex-1 flex-col gap-0.5">
                <span class="flex min-w-0 items-center gap-2">
                  <span
                    class="shrink-0 rounded-[6px] border border-border-strong bg-surface-raised px-1.5 py-px font-mono text-[10.5px] text-accent"
                    data-testid="commit-picker-hash"
                  >
                    {{ commit.hash.slice(0, 7) }}
                  </span>
                  <span
                    v-if="index === 0"
                    class="shrink-0 rounded-full border border-anchor-line bg-anchor-soft px-1.5 py-px font-mono text-[9.5px] uppercase tracking-[0.06em] text-anchor"
                  >
                    tip
                  </span>
                  <span
                    v-if="isLogged(commit.hash)"
                    class="shrink-0 rounded-full bg-accent-soft px-1.5 py-px text-[9.5px] font-semibold uppercase tracking-[0.06em] text-accent"
                    data-testid="commit-picker-logged"
                  >
                    logged
                  </span>
                  <span class="ml-auto shrink-0 text-[10.5px] text-text-subtle">
                    {{ relativeTime(commit.committedAt) }}
                  </span>
                </span>
                <span
                  class="truncate text-[12px] leading-snug"
                  :class="isLogged(commit.hash) ? 'text-text-subtle' : 'text-text'"
                  :title="commit.subject"
                  data-testid="commit-picker-subject"
                >
                  {{ commit.subject }}
                </span>
              </span>
            </button>
          </li>
        </ol>
      </div>

      <form
        class="flex items-center gap-2 border-t border-border bg-canvas px-3 py-2.5"
        @submit.prevent="submitPasted"
      >
        <label class="sr-only" for="commit-picker-hash">A commit hash the log does not reach</label>
        <input
          id="commit-picker-hash"
          ref="pasteField"
          v-model="pasted"
          type="text"
          autocomplete="off"
          spellcheck="false"
          placeholder="or paste a hash…"
          class="focus-ring h-7 min-w-0 flex-1 rounded-[var(--radius-control)] border border-border bg-surface px-2.5 font-mono text-[11.5px] text-text placeholder:font-sans placeholder:text-text-subtle"
          :class="pasted && !pastedIsHash && 'border-danger'"
          :disabled="busy"
          data-testid="commit-picker-paste"
        />
        <button
          type="submit"
          class="focus-ring h-7 shrink-0 rounded-[var(--radius-control)] border border-accent bg-accent-soft px-3 text-[11.5px] font-medium text-accent transition-colors hover:bg-accent hover:text-accent-ink disabled:cursor-not-allowed disabled:opacity-50 disabled:hover:bg-accent-soft disabled:hover:text-accent"
          :disabled="busy || !pastedIsHash"
          data-testid="commit-picker-log-pasted"
        >
          Log
        </button>
      </form>
    </div>
  </Teleport>
</template>

<style scoped>
.commit-picker {
  transform-origin: top right;
}
</style>
