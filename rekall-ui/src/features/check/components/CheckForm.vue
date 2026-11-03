<script setup lang="ts">
import { computed, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppField from '@/components/ui/AppField.vue'
import AppInput from '@/components/ui/AppInput.vue'
import type { CheckInput, DoneCheck } from '@/features/check/model/check'

/**
 * The four things a check needs: a name to read in a refusal, the command, where it runs and
 * how long it may take. Used to add a check and to edit one.
 */
const props = defineProps<{ initial: DoneCheck | null; saving: boolean }>()

const emit = defineEmits<{ save: [input: CheckInput]; cancel: [] }>()

const DEFAULT_TIMEOUT_SECONDS = '600'
const MAX_TIMEOUT_SECONDS = 3600

const label = ref(props.initial?.label ?? '')
const command = ref(props.initial?.command ?? '')
const folder = ref(props.initial?.folder ?? '')
const timeout = ref(String(props.initial?.timeoutSeconds ?? DEFAULT_TIMEOUT_SECONDS))

const timeoutSeconds = computed(() => Number(timeout.value))
const timeoutError = computed(() => {
  const seconds = timeoutSeconds.value
  const valid = Number.isInteger(seconds) && seconds >= 1 && seconds <= MAX_TIMEOUT_SECONDS
  return valid ? null : `Between 1 and ${MAX_TIMEOUT_SECONDS} seconds.`
})
const folderError = computed(() => {
  const text = folder.value.trim()
  if (text.startsWith('/') || text.split('/').includes('..')) return 'A folder inside the project: not absolute, and no "..".'
  return null
})
const canSave = computed(
  () => label.value.trim() !== '' && command.value.trim() !== '' && !timeoutError.value && !folderError.value
)

function submit(): void {
  if (!canSave.value || props.saving) return
  emit('save', {
    label: label.value.trim(),
    command: command.value.trim(),
    folder: folder.value.trim(),
    timeoutSeconds: timeoutSeconds.value
  })
}
</script>

<template>
  <form class="rounded-card border border-border bg-canvas px-3 pb-1 pt-3" data-testid="check-form" @submit.prevent="submit">
    <AppField label="Name" required>
      <template #default="{ fieldId }">
        <AppInput :id="fieldId" v-model="label" placeholder="backend tests" data-testid="check-label-field" />
      </template>
    </AppField>
    <AppField label="Command" required hint="Runs in a shell. A non-zero exit refuses the claim.">
      <template #default="{ fieldId, describedBy }">
        <AppInput
          :id="fieldId"
          v-model="command"
          mono
          :described-by="describedBy"
          placeholder="cargo test --workspace"
          data-testid="check-command-field"
        />
      </template>
    </AppField>
    <div class="grid gap-x-3 sm:grid-cols-[1fr_9rem]">
      <AppField label="Folder" hint="Inside the project folder. Empty runs in the folder itself." :error="folderError">
        <template #default="{ fieldId, describedBy }">
          <AppInput
            :id="fieldId"
            v-model="folder"
            mono
            :described-by="describedBy"
            placeholder="rekall-ui"
            data-testid="check-folder-field"
          />
        </template>
      </AppField>
      <AppField label="Timeout (seconds)" required :error="timeoutError">
        <template #default="{ fieldId, describedBy }">
          <AppInput
            :id="fieldId"
            v-model="timeout"
            type="number"
            mono
            :described-by="describedBy"
            data-testid="check-timeout-field"
          />
        </template>
      </AppField>
    </div>
    <div class="mb-2 flex justify-end gap-2">
      <AppButton size="sm" variant="quiet" data-testid="check-cancel" @click="emit('cancel')">Cancel</AppButton>
      <AppButton size="sm" variant="primary" type="submit" :disabled="!canSave" :loading="saving" data-testid="check-save">
        {{ initial ? 'Save check' : 'Add check' }}
      </AppButton>
    </div>
  </form>
</template>
