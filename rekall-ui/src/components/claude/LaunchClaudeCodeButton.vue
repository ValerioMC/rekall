<script setup lang="ts">
import { computed, ref, useId } from 'vue'
import { canLaunchClaudeCode, launchClaudeCode } from '@/common/native/desktop'
import { skipsPermissions } from '@/common/config/claude-launch'
import { useToastStore } from '@/stores/toast.store'

defineOptions({ inheritAttrs: false })

const props = defineProps<{
  anchors: string
  folder: string | null
  missingHint?: string
}>()

const toast = useToastStore()

const available = canLaunchClaudeCode()
const descriptionId = useId()

const phase = ref<'idle' | 'launching' | 'launched'>('idle')
const nudging = ref(false)
let settleTimer: ReturnType<typeof setTimeout> | null = null
let nudgeTimer: ReturnType<typeof setTimeout> | null = null

const missing = computed(() => props.missingHint ?? 'Set this project’s folder to open a session from it')

const detail = computed(() =>
  props.folder
    ? `Opens ${props.folder} in your terminal app, running /rk ${props.anchors}`
    : missing.value
)

const toneClasses = computed(() =>
  props.folder
    ? 'border-border bg-transparent text-text-muted hover:border-border-strong hover:bg-surface-raised hover:text-text'
    : 'border-transparent bg-transparent text-text-subtle hover:bg-surface-raised hover:text-text-muted'
)

const buttonEl = ref<HTMLButtonElement | null>(null)
const previewing = ref(false)
const previewStyle = ref<Record<string, string>>({})
const PREVIEW_WIDTH = 272

function positionPreview(): void {
  const rect = buttonEl.value?.getBoundingClientRect()
  if (!rect) return
  const left = Math.min(Math.max(12, rect.left), window.innerWidth - PREVIEW_WIDTH - 12)
  previewStyle.value = { top: `${rect.bottom + 8}px`, left: `${left}px` }
}

function openPreview(): void {
  positionPreview()
  previewing.value = true
}

function closePreview(): void {
  previewing.value = false
}

async function launch(): Promise<void> {
  if (phase.value === 'launching') return
  if (!props.folder) {
    toast.notifyError(new Error(missing.value))
    if (nudgeTimer) clearTimeout(nudgeTimer)
    nudging.value = false
    requestAnimationFrame(() => {
      nudging.value = true
      nudgeTimer = setTimeout(() => (nudging.value = false), 400)
    })
    return
  }
  phase.value = 'launching'
  try {
    const terminal = await launchClaudeCode({
      directory: props.folder,
      anchors: props.anchors,
      skipPermissions: skipsPermissions()
    })
    toast.notify(`Opened in ${terminal}.`)
    phase.value = 'launched'
    if (settleTimer) clearTimeout(settleTimer)
    settleTimer = setTimeout(() => (phase.value = 'idle'), 1200)
  } catch (caught) {
    toast.notifyError(caught)
    phase.value = 'idle'
  }
}
</script>

<template>
  <button
    v-if="available"
    ref="buttonEl"
    v-bind="$attrs"
    type="button"
    class="launch-btn focus-ring relative inline-flex h-7 shrink-0 items-center gap-1.5 rounded-[var(--radius-control)] border px-2.5 text-xs font-medium transition-all duration-150 active:translate-y-px disabled:cursor-not-allowed disabled:opacity-60"
    :class="[toneClasses, nudging && 'nudge', phase === 'launched' && 'flash']"
    :disabled="phase === 'launching'"
    :aria-describedby="descriptionId"
    data-testid="launch-claude-code"
    @click="launch"
    @mouseenter="openPreview"
    @mouseleave="closePreview"
    @focus="openPreview"
    @blur="closePreview"
  >
    <span class="relative grid size-3.5 shrink-0 place-items-center">
      <Transition name="glyph" mode="out-in">
        <span
          v-if="phase === 'launching'"
          key="spin"
          class="size-3 animate-spin rounded-full border-[1.6px] border-current/25 border-t-current"
          aria-hidden="true"
        />
        <svg v-else-if="phase === 'launched'" key="check" class="size-3.5" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path
            d="M2.1 6.3 4.6 8.8 9.9 3.4"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        <svg v-else key="out" class="size-3.5" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <path
            d="M9.4 6.9v2a1.1 1.1 0 0 1-1.1 1.1H3a1.1 1.1 0 0 1-1.1-1.1V3.6A1.1 1.1 0 0 1 3 2.5h2"
            stroke="currentColor"
            stroke-width="1.1"
            stroke-linecap="round"
          />
          <path
            d="M7.2 2.1H10v2.8"
            stroke="currentColor"
            stroke-width="1.2"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
          <path d="M10 2.1 5.7 6.4" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
        </svg>
      </Transition>
    </span>
    <span>Open in terminal</span>
  </button>

  <span v-if="available" :id="descriptionId" class="sr-only">{{ detail }}</span>

  <Teleport v-if="available" to="body">
    <div
      v-if="previewing"
      class="rise pointer-events-none fixed z-(--z-toast) w-[272px] rounded-[var(--radius-card)] border border-border-strong bg-surface-raised p-3 shadow-modal"
      :style="previewStyle"
      role="tooltip"
    >
      <p class="flex items-center gap-1.5 eyebrow">
        <span
          class="size-1.5 shrink-0 rounded-full"
          :class="folder ? 'bg-safe' : 'bg-warn'"
          aria-hidden="true"
        />
        {{ folder ? 'Opens your terminal app' : 'Nothing to open yet' }}
      </p>

      <template v-if="folder">
        <p class="mt-2 truncate font-mono text-[11px] text-text-subtle">{{ folder }}</p>
        <p class="anchor-chip mt-1.5 flex max-w-full items-center gap-1.5 truncate px-2 py-1 font-mono text-[11px]">
          <span class="shrink-0 opacity-60">/rk</span>
          <span class="truncate">{{ anchors }}</span>
        </p>
      </template>
      <p v-else class="mt-1.5 text-[11.5px] leading-relaxed text-text-muted">{{ missing }}</p>
    </div>
  </Teleport>
</template>

<style scoped>
.glyph-enter-active,
.glyph-leave-active {
  transition:
    opacity 110ms ease,
    transform 110ms ease;
}
.glyph-enter-from,
.glyph-leave-to {
  opacity: 0;
  transform: scale(0.55);
}

@keyframes launch-nudge {
  0%,
  100% {
    transform: translateX(0);
  }
  25% {
    transform: translateX(-3px);
  }
  75% {
    transform: translateX(3px);
  }
}
.nudge {
  animation: launch-nudge 380ms ease;
}
</style>
