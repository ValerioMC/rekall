<script setup lang="ts">
import { onMounted, ref } from 'vue'
import AppBadge from '@/components/ui/AppBadge.vue'
import AppButton from '@/components/ui/AppButton.vue'
import { useUpdateStore } from '@/stores/update.store'

const update = useUpdateStore()
const failure = ref<string | null>(null)

async function checkNow(): Promise<void> {
  failure.value = null
  try {
    await update.check(true)
  } catch (caught) {
    failure.value = caught instanceof Error ? caught.message : String(caught)
  }
}

onMounted(() => {
  if (update.status === null) void checkNow()
})
</script>

<template>
  <section aria-labelledby="version-heading">
    <h3 id="version-heading" class="eyebrow mb-2">Version</h3>
    <div class="flex items-center gap-3">
      <span class="font-mono text-[12.5px] text-text" data-testid="version-current">
        Rekall {{ update.status ? `v${update.status.current}` : '…' }}
      </span>
      <AppButton size="sm" variant="secondary" :loading="update.checking" data-testid="version-check" @click="checkNow">
        Check for updates
      </AppButton>
    </div>

    <p v-if="failure" class="mt-2.5 text-[12px] text-danger" role="alert">{{ failure }}</p>

    <div
      v-else-if="update.latest"
      class="mt-3 flex items-center gap-3 rounded-[var(--radius-control)] border border-accent-deep bg-accent-soft px-3.5 py-3"
      data-testid="version-available"
    >
      <span class="min-w-0 flex-1 text-[12.5px] text-text">
        Rekall <strong class="font-semibold">{{ update.latest.version }}</strong> is available.
      </span>
      <AppButton
        v-if="update.installsItself"
        variant="primary"
        size="sm"
        :loading="update.installing"
        data-testid="version-install"
        @click="update.install"
      >
        {{ update.installing ? 'Installing…' : 'Install' }}
      </AppButton>
      <a
        v-else-if="update.downloadHref"
        :href="update.downloadHref"
        target="_blank"
        rel="noopener noreferrer"
        class="focus-ring key-gold inline-flex h-7 items-center justify-center rounded-[3px] px-2.5 text-xs font-semibold whitespace-nowrap select-none"
        data-testid="version-download"
      >
        Download
      </a>
    </div>
    <p v-if="update.installFailure" class="mt-2.5 text-[12px] leading-relaxed text-danger" role="alert">
      The install did not finish: {{ update.installFailure }}. The button now opens the download instead.
    </p>

    <p v-if="!failure && update.status?.check === 'UP_TO_DATE'" class="mt-2.5 flex items-center gap-2 text-[12px]">
      <AppBadge tone="safe" dot>Up to date</AppBadge>
      <span class="text-text-subtle">This is the latest published version.</span>
    </p>
    <p v-else-if="!failure && update.status?.check === 'DISABLED'" class="mt-2.5 text-[11.5px] text-text-subtle">
      Update checks are turned off (<code>rekall.update-check.enabled</code>).
    </p>
    <p v-else-if="!failure && update.status?.check === 'UNAVAILABLE'" class="mt-2.5 text-[11.5px] text-text-subtle">
      The latest release could not be reached. Try again in a moment.
    </p>
    <p v-else-if="!update.installFailure" class="mt-2.5 text-[11.5px] leading-relaxed text-text-subtle">
      Rekall checks when it starts and offers a newer version in a dialog.
    </p>
  </section>
</template>
