<script setup lang="ts">
import { ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import EpicGlyph from '@/features/catalog/components/epic/EpicGlyph.vue'
import ConvertToEpicDialog from '@/features/task/components/ConvertToEpicDialog.vue'
import type { Task } from '@/features/catalog/model/catalog'

/** Opens the conversion of a task into an epic; the dialog says what is deleted before anything is. */
defineProps<{ task: Task }>()

const isConverting = ref(false)
</script>

<template>
  <AppButton size="sm" data-testid="task-convert-to-epic" title="Turn this task into an epic" @click="isConverting = true">
    <EpicGlyph :size="13" />
    <span data-label>Convert to epic</span>
  </AppButton>
  <Transition name="dialog">
    <ConvertToEpicDialog v-if="isConverting" :task="task" @close="isConverting = false" />
  </Transition>
</template>
