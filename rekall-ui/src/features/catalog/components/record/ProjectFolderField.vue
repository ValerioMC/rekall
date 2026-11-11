<script setup lang="ts">
import AppButton from '@/components/ui/AppButton.vue'
import { desktopHost, pickFolder } from '@/common/native/desktop'

/** The folder a new project's sessions run in: tasks cannot be opened in a terminal without it. */
const folder = defineModel<string>({ required: true })

const canBrowse = desktopHost() !== null

async function browse(): Promise<void> {
  const chosen = await pickFolder(folder.value)
  if (chosen) folder.value = chosen
}
</script>

<template>
  <label for="record-project-folder" class="mb-1.5 mt-5 block eyebrow text-caption">Folder</label>
  <div class="flex items-center gap-2">
    <input
      id="record-project-folder"
      v-model="folder"
      data-testid="record-project-folder"
      spellcheck="false"
      autocomplete="off"
      class="field text-text h-8 min-w-0 flex-1 rounded-control px-2.5 font-mono text-caption"
      placeholder="/Users/you/Projects/thing"
    />
    <AppButton v-if="canBrowse" size="sm" variant="secondary" data-testid="record-project-folder-browse" @click="browse">
      Browse
    </AppButton>
  </div>
  <p class="mt-1.5 text-caption text-text-subtle">
    Where <span class="text-text-muted">Open in terminal</span> starts the session, so the tasks can run.
  </p>
</template>
