<script setup lang="ts">
/**
 * Announces a newer release when the desktop app starts. One button installs it over this app and
 * restarts; after a failed install, or where the app cannot, it opens the download instead.
 */
import { nextTick, onMounted, onUnmounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import { useModalGate } from '@/composables/useModalGate'
import { trapTabKey } from '@/common/a11y/focus-trap'
import { useUpdateStore } from '@/stores/update.store'

const update = useUpdateStore()
const { open: openModal, close: closeModal } = useModalGate()

const panel = ref<HTMLElement | null>(null)
const installButton = ref<InstanceType<typeof AppButton> | null>(null)

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    event.stopPropagation()
    update.dismiss()
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
    v-if="update.latest"
    class="fixed inset-0 z-(--z-modal) grid place-items-center bg-black/60 p-5 backdrop-blur-sm"
    @click.self="update.dismiss"
  >
    <div
      ref="panel"
      class="dialog-panel w-full max-w-[440px] overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface shadow-lift"
      role="dialog"
      aria-modal="true"
      :aria-label="`Rekall ${update.latest.version} is available`"
      data-testid="update-dialog"
    >
      <div class="p-5">
        <h2 class="mb-2 text-[16px] font-semibold text-text">Rekall {{ update.latest.version }} is available</h2>
        <p class="text-[13px] leading-relaxed text-text-muted">
          <template v-if="update.installsItself">
            Rekall downloads the new version, installs it in place of this one and restarts. Your projects, tasks and
            notes stay where they are.
          </template>
          <template v-else>
            Download the new version and install it over this one. Your projects, tasks and notes stay where they are.
          </template>
        </p>
        <p
          v-if="update.installFailure"
          class="mt-3.5 text-[12px] leading-relaxed text-danger"
          role="alert"
          data-testid="update-failure"
        >
          The install did not finish: {{ update.installFailure }}. Download it and install it by hand instead.
        </p>
      </div>
      <div class="flex justify-end gap-2 border-t border-border bg-canvas px-5 py-3">
        <AppButton variant="ghost" size="sm" :disabled="update.installing" @click="update.dismiss">Later</AppButton>
        <AppButton
          v-if="update.installsItself"
          ref="installButton"
          variant="primary"
          size="sm"
          :loading="update.installing"
          data-testid="update-install"
          @click="update.install"
        >
          {{ update.installing ? 'Installing…' : 'Install' }}
        </AppButton>
        <a
          v-else-if="update.downloadHref"
          :href="update.downloadHref"
          class="focus-ring key-gold inline-flex h-7 items-center justify-center gap-1.5 rounded-[3px] px-2.5 text-xs font-semibold whitespace-nowrap select-none"
          data-testid="update-download"
          @click="update.dismiss"
        >
          Download
        </a>
      </div>
    </div>
  </div>
</template>
