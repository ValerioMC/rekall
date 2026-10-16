<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import FolderPickerButton from '@/features/project/components/FolderPickerButton.vue'
import AppButton from '@/components/ui/AppButton.vue'
import { folderName } from '@/features/project/lib/component-folder'
import type { ProjectComponentInput } from '@/features/project/model/project-component'

/**
 * The form that adds a component. The folder comes first because it names the component: until
 * a name is typed, the folder's own name stands in, and the field shows it as its placeholder.
 */
defineProps<{ root: string | null; creating: boolean }>()

const emit = defineEmits<{ submit: [input: ProjectComponentInput]; cancel: [] }>()

const folder = ref('')
const name = ref('')
const description = ref('')
const folderInput = ref<HTMLInputElement | null>(null)

const suggestedName = computed(() => folderName(folder.value))
const canSubmit = computed(() => folder.value.trim() !== '')

function submit(): void {
  if (!canSubmit.value) return
  emit('submit', {
    folder: folder.value.trim(),
    name: name.value.trim() || suggestedName.value,
    description: description.value.trim()
  })
}

onMounted(() => folderInput.value?.focus())
</script>

<template>
  <form
    class="rounded-control border border-dashed border-accent/40 bg-accent-soft p-3"
    data-testid="component-composer"
    @submit.prevent="submit"
    @keydown.esc.stop.prevent="emit('cancel')"
  >
    <label class="mb-1 block text-caption text-text-muted" for="new-component-folder">Folder</label>
    <div class="flex items-center gap-1.5">
      <input
        id="new-component-folder"
        ref="folderInput"
        v-model="folder"
        class="field text-text h-8 min-w-0 flex-1 rounded-control px-2 font-mono text-caption"
        maxlength="1000"
        autocomplete="off"
        spellcheck="false"
        placeholder="/Users/you/Projects/thing/module"
        data-testid="new-component-folder"
      />
      <FolderPickerButton :root="root" :current="folder" data-testid="new-component-browse" @pick="folder = $event" />
    </div>

    <label class="mb-1 mt-2 block text-caption text-text-muted" for="new-component-name">Name</label>
    <input
      id="new-component-name"
      v-model="name"
      class="field text-text h-8 w-full rounded-control px-2 text-small"
      maxlength="200"
      autocomplete="off"
      :placeholder="suggestedName || 'The folder’s name'"
      data-testid="new-component-name"
    />

    <label class="mb-1 mt-2 block text-caption text-text-muted" for="new-component-description">Description</label>
    <textarea
      id="new-component-description"
      v-model="description"
      rows="2"
      class="field text-text block w-full resize-y rounded-control p-2 text-small leading-relaxed"
      placeholder="What this component is and what it owns."
      data-testid="new-component-description"
      @keydown.meta.enter.prevent="submit"
      @keydown.ctrl.enter.prevent="submit"
    />

    <div class="mt-2.5 flex items-center justify-end gap-2">
      <AppButton variant="ghost" size="sm" data-testid="new-component-cancel" @click="emit('cancel')">Cancel</AppButton>
      <AppButton type="submit" variant="primary" size="sm" :disabled="!canSubmit" :loading="creating" data-testid="new-component-submit">
        Add
      </AppButton>
    </div>
  </form>
</template>
