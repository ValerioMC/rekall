<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import { useModalGate } from '@/composables/useModalGate'
import { trapTabKey } from '@/common/a11y/focus-trap'
import { rkWrapupCommand } from '@/common/format/rk-command'

const props = withDefaults(
  defineProps<{ taskTitle: string; anchor: string; sending?: boolean }>(),
  { sending: false }
)
const emit = defineEmits<{ cancel: []; send: [message: string] }>()

const { open: openModal, close: closeModal } = useModalGate()

const panel = ref<HTMLElement | null>(null)
const field = ref<HTMLInputElement | null>(null)
const message = ref('')

const preview = computed(() => rkWrapupCommand(props.anchor, message.value))

function send(): void {
  if (props.sending) return
  emit('send', message.value)
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    event.stopPropagation()
    if (!props.sending) emit('cancel')
    return
  }
  if (panel.value) trapTabKey(panel.value, event)
}

onMounted(async () => {
  openModal()
  window.addEventListener('keydown', onKeydown, true)
  await nextTick()
  field.value?.focus()
})

onUnmounted(() => {
  closeModal()
  window.removeEventListener('keydown', onKeydown, true)
})
</script>

<template>
  <div
    class="fixed inset-0 z-(--z-modal) grid place-items-center bg-black/70 p-5 backdrop-blur-sm"
    @click.self="emit('cancel')"
  >
    <div
      ref="panel"
      class="dialog-panel w-full max-w-[480px] overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface shadow-modal"
      role="dialog"
      aria-modal="true"
      aria-label="Run the wrapup here"
      data-testid="wrapup-here-dialog"
    >
      <header class="flex items-center gap-3 border-b border-border px-5 py-4">
        <span class="session-caret session-caret-busy shrink-0" aria-hidden="true" />
        <span class="min-w-0 flex-1">
          <span class="block text-[15px] font-semibold tracking-[-0.01em] text-text">
            Run the wrapup here
          </span>
          <span class="mt-0.5 block truncate text-[12.5px] leading-relaxed text-text-muted">
            Types the command into the session already running on {{ taskTitle }}.
          </span>
        </span>
        <button
          class="focus-ring grid size-7 shrink-0 place-items-center rounded-md text-text-subtle transition-colors hover:bg-surface-raised hover:text-text disabled:cursor-not-allowed disabled:opacity-40"
          aria-label="Close"
          :disabled="sending"
          @click="emit('cancel')"
        >
          <CloseGlyph />
        </button>
      </header>

      <div class="px-5 py-4">
        <label for="wrapup-here-message" class="mb-1.5 block eyebrow text-[11px]">
          Directive (optional)
        </label>
        <input
          id="wrapup-here-message"
          ref="field"
          v-model="message"
          type="text"
          data-testid="wrapup-here-message"
          :disabled="sending"
          class="field text-text h-10 w-full rounded-[var(--radius-control)] px-3 text-[13.5px] disabled:opacity-60"
          placeholder="What to focus the wrapup on, in your own words"
          @keydown.enter.prevent="send"
        />
        <p class="mt-1.5 text-[11.5px] text-text-subtle">
          Left blank, Claude writes the state as it stands. Said something, it writes only that.
        </p>

        <p
          class="anchor-chip mt-4 flex items-center gap-2 truncate px-2.5 py-1.5 font-mono text-[11px]"
          data-testid="wrapup-here-preview"
        >
          <span class="shrink-0 opacity-60">&gt;</span>
          <span class="min-w-0 flex-1 truncate">{{ preview }}</span>
        </p>
      </div>

      <footer class="flex justify-end gap-2 border-t border-border bg-canvas px-5 py-3">
        <AppButton variant="ghost" size="sm" :disabled="sending" @click="emit('cancel')">
          Cancel
        </AppButton>
        <AppButton
          variant="primary"
          size="sm"
          data-testid="wrapup-here-send"
          :loading="sending"
          @click="send"
        >
          Send to terminal
        </AppButton>
      </footer>
    </div>
  </div>
</template>
