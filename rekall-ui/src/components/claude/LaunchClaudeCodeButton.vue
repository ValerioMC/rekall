<script setup lang="ts">
import { computed, ref, useId } from 'vue'
import { canLaunchClaudeCode, launchClaudeCode } from '@/common/native/desktop'
import { skipsPermissions } from '@/common/config/claude-launch'
import { useToastStore } from '@/stores/toast.store'

/**
 * Three root nodes, not one: the button, its always-present screen-reader description, and the
 * teleported preview. Vue only auto-forwards a caller's `class` onto a single root, so it is
 * turned off here and pointed at the button by hand — the one root a caller's `class` (every
 * pane places this with `shrink-0`) is ever meant to land on.
 */
defineOptions({ inheritAttrs: false })

/**
 * The shortest path between a record and a session that has read it.
 *
 * A terminal opens in the project's folder with `/rk` already running, so the anchor is never
 * copied, pasted or typed. It exists only inside the macOS application: a browser tab cannot
 * open a terminal, and the endpoint that would let it is one any other page could call.
 *
 * Without a folder the button stays and says so when it is pressed. Not disabled: a disabled
 * button shows no tooltip in this window, so the one explanation there was would be invisible to
 * exactly the person who needs it, on a button that looks broken.
 *
 * The glyph tells the same story the toast does, a beat earlier and on the thing you actually
 * clicked: a prompt, then a spinner, then a checkmark that holds long enough to read before the
 * button settles back to being a prompt. A hover or a focus opens a preview of exactly what is
 * about to run — the folder and the `/rk` line — so pressing it is never the first look at what
 * it does. The preview is teleported to the document body rather than positioned in place,
 * because two of the four panes this button sits in clip their header for a texture layer behind
 * it, and a popover clipped by its own background would be worse than no popover.
 */
const props = defineProps<{
  /** The anchors as `/rk` takes them, which the server built. */
  anchors: string
  /** The project's folder, or null when nobody has set one. */
  folder: string | null
  /** Where to go and set it, in words, for the case where it is not on this screen. */
  missingHint?: string
}>()

const toast = useToastStore()

const available = canLaunchClaudeCode()
const descriptionId = useId()

/** idle → launching → launched → idle. Never anything but idle while there is no folder: a
 *  press that cannot run has nothing to animate toward. */
const phase = ref<'idle' | 'launching' | 'launched'>('idle')
const nudging = ref(false)
let settleTimer: ReturnType<typeof setTimeout> | null = null
let nudgeTimer: ReturnType<typeof setTimeout> | null = null

const missing = computed(() => props.missingHint ?? 'Set this project’s folder to open a session from it')

const detail = computed(() =>
  props.folder ? `Opens a terminal in ${props.folder} running /rk ${props.anchors}` : missing.value
)

const toneClasses = computed(() =>
  props.folder
    ? 'border-anchor-line bg-anchor-soft text-anchor hover:border-anchor hover:bg-anchor/10 hover:text-anchor-strong hover:shadow-[0_0_20px_-6px_var(--color-anchor-soft)]'
    : 'border-transparent bg-transparent text-text-subtle hover:bg-surface-raised hover:text-text-muted'
)

// ------------------------------------------------------------------ the preview

/**
 * Shown on hover or focus, positioned from the button's own rect rather than laid out beside it
 * in the DOM — teleported, so the header it sits in can clip its texture without clipping this.
 */
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

// ------------------------------------------------------------------ launching

async function launch(): Promise<void> {
  if (phase.value === 'launching') return
  if (!props.folder) {
    toast.notifyError(new Error(missing.value))
    if (nudgeTimer) clearTimeout(nudgeTimer)
    nudging.value = false
    // Restarts the animation even on a second press before the first finished, which a bare
    // class toggle would not: the browser needs the class off for at least one frame to see it
    // as new rather than as already applied.
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
        <svg v-else key="prompt" class="size-3.5" viewBox="0 0 12 12" fill="none" aria-hidden="true">
          <rect x="0.9" y="1.5" width="10.2" height="9" rx="1.6" stroke="currentColor" stroke-width="1.1" />
          <path
            d="M2.9 4.5 4.8 6 2.9 7.5"
            stroke="currentColor"
            stroke-width="1.2"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
          <path d="M5.9 7.5h2.3" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
        </svg>
      </Transition>
    </span>
    <span>Open in Claude Code</span>
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
        {{ folder ? 'Opens a terminal here' : 'Nothing to open yet' }}
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
/* The prompt glyph swapping for the spinner, and the spinner for the check: a cut rather than a
   cross-fade, because two overlapping strokes at this size read as a smudge. */
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

/* Said no without moving anywhere: the button that cannot run yet shakes off the click instead
   of pretending to take it. */
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
