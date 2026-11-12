<script setup lang="ts">
import AppConfirm from '@/components/ui/AppConfirm.vue'
import type { RekallDocument } from '@/features/notes/model/document'

defineProps<{ note: RekallDocument }>()

const emit = defineEmits<{ cancel: []; confirm: [] }>()
</script>

<template>
  <AppConfirm
    :title="`Delete ${note.title}?`"
    body="This removes the note and its content."
    blast="removes it from its task · not recoverable"
    confirm-label="Delete note"
    @cancel="emit('cancel')"
    @confirm="emit('confirm')"
  />
</template>
