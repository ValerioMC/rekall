<script setup lang="ts">
import { computed } from 'vue'
import AppField from '@/components/ui/AppField.vue'
import AppInput from '@/components/ui/AppInput.vue'
import { ROOT_PATH_FIELD } from '@/features/connection/lib/connection-fields'
import type { ConnectionKind } from '@/features/connection/model/connection'

/** What a GitLab or GitHub token covers, so several connections on one instance can sit side by side. */
const props = defineProps<{ kind: ConnectionKind; modelValue: string }>()
const emit = defineEmits<{ 'update:modelValue': [value: string] }>()

const field = computed(() => ROOT_PATH_FIELD[props.kind])
</script>

<template>
  <AppField
    v-if="field"
    v-slot="{ fieldId, describedBy }"
    :label="field.title"
    :hint="`${field.example} Leave empty to cover the whole instance.`"
  >
    <AppInput
      :id="fieldId"
      :model-value="modelValue"
      :described-by="describedBy"
      mono
      data-testid="connection-root-path"
      :placeholder="field.placeholder"
      @update:model-value="emit('update:modelValue', $event)"
    />
  </AppField>
</template>
