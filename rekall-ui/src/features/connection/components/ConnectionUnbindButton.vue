<script setup lang="ts">
import { ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'

/** Unbinding loses the binding's target, so the first press asks and the second one does it. */
const props = defineProps<{ subject: string; testid: string }>()
const emit = defineEmits<{ confirm: [] }>()

const asking = ref(false)

function confirm(): void {
  asking.value = false
  emit('confirm')
}
</script>

<template>
  <span v-if="asking" class="inline-flex items-center gap-1.5" role="group" :aria-label="`Unbind ${subject}?`">
    <span class="text-caption text-text-muted">Unbind {{ subject }}?</span>
    <AppButton size="sm" variant="danger" class="max-sm:min-h-11" :data-testid="`${props.testid}-confirm`" @click="confirm">Yes, unbind</AppButton>
    <AppButton size="sm" variant="ghost" class="max-sm:min-h-11" @click="asking = false">Cancel</AppButton>
  </span>
  <AppButton
    v-else
    size="sm"
    variant="danger-quiet"
    class="max-sm:min-h-11"
    :aria-label="`Unbind ${subject}`"
    :data-testid="props.testid"
    @click="asking = true"
    >Unbind</AppButton
  >
</template>
