<script setup lang="ts">
import { computed, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import GitlabPathField from '@/features/connection/components/GitlabPathField.vue'
import type { ProjectConnection } from '@/features/connection/model/connection'

/**
 * Changes the GitLab path one binding is narrowed to, with the same field and verdict a new binding
 * gets. The path can be saved only once GitLab confirms it; clearing it gives the binding the whole
 * connection.
 */
const props = defineProps<{ binding: ProjectConnection; rootPath: string | null }>()
const emit = defineEmits<{ save: [target: string | null]; cancel: [] }>()

const resolved = ref<{ readonly ready: boolean; readonly path: string | null }>({ ready: false, path: props.binding.target })
const canSave = computed(() => resolved.value.ready && resolved.value.path !== props.binding.target)
</script>

<template>
  <form
    class="mt-1 mb-2 flex flex-col gap-3 rounded-control border border-border-strong bg-surface px-3 py-3"
    data-testid="gitlab-target-editor"
    @submit.prevent="canSave && emit('save', resolved.path)"
    @keydown.esc.stop="emit('cancel')"
  >
    <GitlabPathField
      :connection-id="binding.connectionId"
      :connection-title="binding.title"
      :base-url="binding.baseUrl"
      :root-path="rootPath"
      :initial-path="binding.target"
      @resolve="resolved = $event"
    />
    <div class="flex items-center justify-end gap-2">
      <AppButton size="sm" variant="ghost" class="max-sm:min-h-11" @click="emit('cancel')">Cancel</AppButton>
      <AppButton size="sm" variant="primary" type="submit" class="max-sm:min-h-11" :disabled="!canSave" data-testid="gitlab-target-save"
        >Save target</AppButton
      >
    </div>
  </form>
</template>
