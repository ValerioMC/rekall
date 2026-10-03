<script setup lang="ts">
/**
 * Offers a newer release in the desktop app. One button installs it over this app and restarts;
 * after a failed install it opens the download in the browser instead.
 */
import { nextTick, onMounted, onUnmounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import { useModalGate } from '@/composables/useModalGate'
import { trapTabKey } from '@/common/a11y/focus-trap'
import { installUpdate } from '@/common/native/desktop'

const props = defineProps<{
  version: string
  downloadHref: string
}>()

const emit = defineEmits<{ close: [] }>()

const { open: openModal, close: closeModal } = useModalGate()

const panel = ref<HTMLElement | null>(null)
const installButton = ref<InstanceType<typeof AppButton> | null>(null)
const installing = ref(false)
const failure = ref<string | null>(null)

function dismiss(): void {
  if (!installing.value) emit('close')
}

// On success the app restarts underneath this dialog, so `installing` is never cleared.
async function install(): Promise<void> {
  installing.value = true
  failure.value = null
  try {
    const restarting = await installUpdate()
    if (!restarting) {
      installing.value = false
      emit('close')
    }
  } catch (error) {
    failure.value = error instanceof Error ? error.message : String(error)
    installing.value = false
  }
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    event.stopPropagation()
    dismiss()
    return
  }
  if (panel.value) trapTabKey(panel.value, event)
}

onMounted(async () => {
  openModal()
  window.addEventListener('keydown', onKeydown, true)
  await nextTick()
  ;(installButton.value?.$el as HTMLElement | undefined)?.focus()
})

onUnmounted(() => {
  closeModal()
  window.removeEventListener('keydown', onKeydown, true)
})
</script>

<template>
  <div
    class="fixed inset-0 z-(--z-modal) grid place-items-center bg-black/60 p-5 backdrop-blur-sm"
    @click.self="dismiss"
  >
    <div
      ref="panel"
      class="dialog-panel w-full max-w-[440px] overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface shadow-lift"
      role="dialog"
      aria-modal="true"
      :aria-label="`Rekall ${props.version} is available`"
      data-testid="update-dialog"
    >
      <div class="p-5">
        <h2 class="mb-2 text-[16px] font-semibold text-text">Rekall {{ props.version }} is available</h2>
        <p class="text-[13px] leading-relaxed text-text-muted">
          Rekall downloads the new version, installs it in place of this one and restarts. Your projects, tasks and
          notes stay where they are.
        </p>
        <p
          v-if="failure"
          class="mt-3.5 text-[12px] leading-relaxed text-danger"
          role="alert"
          data-testid="update-failure"
        >
          The install did not finish: {{ failure }}. Download it and install it by hand instead.
        </p>
      </div>
      <div class="flex justify-end gap-2 border-t border-border bg-canvas px-5 py-3">
        <AppButton variant="ghost" size="sm" :disabled="installing" @click="dismiss">Later</AppButton>
        <a
          v-if="failure"
          :href="props.downloadHref"
          class="focus-ring key-gold inline-flex h-7 items-center justify-center gap-1.5 rounded-[3px] px-2.5 text-xs font-semibold whitespace-nowrap select-none"
          data-testid="update-download"
          @click="emit('close')"
        >
          Download
        </a>
        <AppButton
          v-else
          ref="installButton"
          variant="primary"
          size="sm"
          :loading="installing"
          data-testid="update-install"
          @click="install"
        >
          {{ installing ? 'Installing…' : 'Install' }}
        </AppButton>
      </div>
    </div>
  </div>
</template>
