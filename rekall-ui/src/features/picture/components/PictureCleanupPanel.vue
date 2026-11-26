<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import AppConfirm from '@/components/ui/AppConfirm.vue'
import FormPanel from '@/components/ui/FormPanel.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { formatFileSize } from '@/common/format/file-size'
import { plural } from '@/features/connection/lib/connection-impact'
import PictureCleanupProgress from '@/features/picture/components/PictureCleanupProgress.vue'
import UnusedPictureList from '@/features/picture/components/UnusedPictureList.vue'
import { restOf } from '@/features/picture/lib/picture-cleanup-wording'
import { usePictureCleanupStore } from '@/features/picture/store/pictureCleanup.store'

/** Finds the images no text shows and, once they have been looked at, deletes them. */
const store = usePictureCleanupStore()
const { status } = storeToRefs(store)
const { run, isRunning } = useAsyncAction()

const unreadable = ref(false)
const confirming = ref(false)

const rest = computed(() => (status.value ? restOf(status.value) : null))
const listed = computed(() => (status.value?.state === 'READY' && status.value.pictures.length > 0 ? status.value : null))
const listedCount = computed(() => listed.value?.pictures.length ?? 0)
const listedSize = computed(() => formatFileSize(listed.value?.totalBytes ?? 0))
const summary = computed(() => (listed.value ? `${plural(listedCount.value, 'image')} no text shows, ${listedSize.value}` : undefined))

async function load(): Promise<void> {
  unreadable.value = false
  unreadable.value = (await run(() => store.load())) === null
}

function search(): void {
  void run(() => store.search())
}

function discard(): void {
  void run(() => store.discard())
}

function deleteListed(): void {
  confirming.value = false
  void run(() => store.deleteListed())
}

onMounted(load)
</script>

<template>
  <FormPanel title="Unused images" :description="summary" aria-label="Unused images" data-testid="picture-cleanup">
    <div v-if="!status && unreadable" class="flex flex-wrap items-center justify-between gap-3 px-5 py-4">
      <p class="text-small text-text-subtle">Couldn't read where the image cleanup stands.</p>
      <AppButton size="sm" @click="load">Retry</AppButton>
    </div>

    <div v-else-if="!status" class="px-5 py-4" aria-hidden="true">
      <div class="skeleton h-9 rounded-control" />
    </div>

    <PictureCleanupProgress
      v-else-if="status.state === 'SEARCHING'"
      label="Searching your texts for images"
      :percentage="status.percentage"
    />

    <PictureCleanupProgress v-else-if="status.state === 'DELETING'" label="Deleting unused images" :percentage="status.percentage" />

    <template v-else-if="listed">
      <UnusedPictureList :pictures="listed.pictures" />
      <div class="flex flex-wrap items-center justify-end gap-2 border-t border-border bg-canvas px-5 py-3">
        <p class="mr-auto text-small text-text-subtle">Deleting removes these files for good.</p>
        <AppButton size="sm" variant="ghost" :disabled="isRunning" data-testid="picture-cleanup-cancel" @click="discard">
          Cancel
        </AppButton>
        <AppButton size="sm" variant="danger" :loading="isRunning" data-testid="picture-cleanup-delete" @click="confirming = true">
          Delete {{ plural(listedCount, 'image') }}
        </AppButton>
      </div>
    </template>

    <div
      v-else-if="rest"
      class="flex flex-wrap items-center justify-between gap-x-8 gap-y-3 px-5 py-4"
      :role="rest.failed ? 'alert' : undefined"
    >
      <div class="min-w-0 max-w-[72ch] flex-1 basis-80">
        <p class="text-body font-medium" :class="rest.failed ? 'text-danger' : 'text-text'">{{ rest.headline }}</p>
        <p v-if="rest.detail" class="mt-0.5 break-words text-small leading-relaxed text-text-subtle">{{ rest.detail }}</p>
      </div>
      <AppButton size="sm" :loading="isRunning" data-testid="picture-cleanup-search" @click="search">{{ rest.action }}</AppButton>
    </div>

    <Teleport to="body">
      <Transition name="dialog">
        <AppConfirm
          v-if="confirming && listed"
          :title="`Delete ${plural(listedCount, 'image')}?`"
          body="No text shows them. The files are deleted for good: only a backup taken before still holds them."
          :blast="`${plural(listedCount, 'image file')} · ${listedSize} · deleted for good`"
          :confirm-label="`Delete ${plural(listedCount, 'image')}`"
          @cancel="confirming = false"
          @confirm="deleteListed"
        />
      </Transition>
    </Teleport>
  </FormPanel>
</template>
