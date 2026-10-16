<script setup lang="ts">
/**
 * Announces a newer release when the desktop app starts. One button installs it over this app and
 * restarts; after a failed install, or where the app cannot, it opens the download instead.
 */
import { nextTick, onMounted, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppDialog from '@/components/ui/AppDialog.vue'
import { useUpdateStore } from '@/features/settings/store/update.store'

const update = useUpdateStore()
const installButton = ref<InstanceType<typeof AppButton> | null>(null)

onMounted(async () => {
  await nextTick()
  ;(installButton.value?.$el as HTMLElement | undefined)?.focus()
})
</script>

<template>
  <AppDialog
    v-if="update.latest"
    :label="`Rekall ${update.latest.version} is available`"
    panel-class="max-w-[440px]"
    panel-testid="update-dialog"
    :dismissible="!update.installing"
    @dismiss="update.dismiss()"
  >
    <div class="p-5">
      <h2 class="mb-2 text-title font-semibold text-text">Rekall {{ update.latest.version }} is available</h2>
      <p class="text-body leading-relaxed text-text-muted">
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
        class="mt-3.5 text-small leading-relaxed text-danger"
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
        class="focus-ring key-gold inline-flex h-7 items-center justify-center gap-1.5 rounded-key px-3 text-caption font-semibold whitespace-nowrap select-none"
        data-testid="update-download"
        @click="update.dismiss"
      >
        Download
      </a>
    </div>
  </AppDialog>
</template>
