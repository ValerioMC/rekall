<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { useConsoleStore } from '@/stores/console.store'
import { fetchContextSize } from '@/api/context.api'
import { compactCount, tokensOf } from '@/model/context'
import type { ContextSize } from '@/model/context'
import type { TaskId } from '@/model/branded'

const props = defineProps<{ taskId: TaskId }>()

const store = useConsoleStore()
const { selectedTask, selectedWrapup, taskDocuments, selectedTaskSteps } = storeToRefs(store)

const MEASURE_DEBOUNCE_MS = 400

const size = ref<ContextSize | null>(null)
const failed = ref(false)
const open = ref(false)
const chip = ref<HTMLElement | null>(null)
const position = ref({ top: 0, left: 0 })

let timer: ReturnType<typeof setTimeout> | null = null
let sequence = 0

// Everything that changes what /rk hands over for this task. The server renders the real text;
// this only decides when to ask it again.
const signature = computed(() =>
  [
    props.taskId,
    selectedTask.value?.updatedAt,
    selectedWrapup.value?.updatedAt,
    ...taskDocuments.value.map((document) => `${document.id}:${document.updatedAt}:${document.contextMode}`),
    ...selectedTaskSteps.value.map((step) => `${step.id}:${step.updatedAt}:${step.state}`)
  ].join('|')
)

async function measure(taskId: TaskId, at: number): Promise<void> {
  try {
    const measured = await fetchContextSize(taskId)
    if (at !== sequence) return
    size.value = measured
    failed.value = false
  } catch {
    if (at !== sequence) return
    failed.value = true
  }
}

watch(
  signature,
  () => {
    if (timer) clearTimeout(timer)
    const at = ++sequence
    const taskId = props.taskId
    timer = setTimeout(() => void measure(taskId, at), MEASURE_DEBOUNCE_MS)
  },
  { immediate: true }
)

watch(
  () => props.taskId,
  () => {
    size.value = null
    open.value = false
  }
)

onUnmounted(() => {
  if (timer) clearTimeout(timer)
})

// The header the chip sits in clips its overflow, so the breakdown is drawn on the body, under
// the chip, at the chip's position when it opened.
function show(): void {
  const rect = chip.value?.getBoundingClientRect()
  if (rect) position.value = { top: rect.bottom + 6, left: rect.left }
  open.value = true
}

const heaviestNote = computed(() =>
  size.value?.parts.find((part) => part.label.startsWith('Note: ') && !part.reference) ?? null
)
</script>

<template>
  <span class="relative inline-flex" @mouseleave="open = false">
    <button
      ref="chip"
      type="button"
      class="focus-ring inline-flex h-[26px] items-center gap-1.5 rounded-full border border-border px-2.5 font-mono text-[11px] text-text-muted transition-colors hover:border-border-strong hover:text-text"
      :title="failed ? 'Could not measure this task\'s context' : 'What /rk hands a session for this task'"
      :aria-expanded="open"
      data-testid="context-size"
      @click="open ? (open = false) : show()"
      @mouseenter="show"
      @keydown.esc="open = false"
      @blur="open = false"
    >
      <template v-if="size">≈ {{ compactCount(size.estimatedTokens) }} tokens</template>
      <template v-else-if="failed">size unknown</template>
      <template v-else>measuring…</template>
    </button>

    <Teleport to="body">
      <Transition name="popover">
        <div
          v-if="open && size"
          class="pointer-events-none fixed z-(--z-overlay) w-[340px] overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface shadow-modal"
          :style="{ top: `${position.top}px`, left: `${position.left}px` }"
          role="tooltip"
          aria-label="Context size"
          data-testid="context-size-parts"
        >
          <p class="border-b border-border px-3.5 py-2 text-[11px] leading-relaxed text-text-subtle">
            {{ size.characters.toLocaleString() }} characters on <span class="font-mono">/rk</span>, about
            {{ size.estimatedTokens.toLocaleString() }} tokens. An estimate for comparing, not a bill.
          </p>
          <ul class="max-h-[300px] overflow-y-auto py-1">
            <li
              v-for="part in size.parts"
              :key="part.label"
              class="flex items-baseline gap-2 px-3.5 py-1 text-[11.5px]"
            >
              <span class="min-w-0 flex-1 truncate text-text">{{ part.label }}</span>
              <span v-if="part.reference" class="shrink-0 text-[10px] uppercase tracking-[0.04em] text-accent">reference</span>
              <span class="shrink-0 font-mono text-text-muted">≈ {{ compactCount(tokensOf(part.characters)) }}</span>
            </li>
          </ul>
          <p v-if="heaviestNote" class="border-t border-border px-3.5 py-2 text-[11px] leading-relaxed text-text-subtle">
            A note most sessions do not need can go <span class="text-text">By reference</span> from its
            own pane: sessions then get a line and its anchor, and load it only when the work asks.
          </p>
        </div>
      </Transition>
    </Teleport>
  </span>
</template>
