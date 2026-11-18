<script setup lang="ts">
import { computed } from 'vue'
import { storeToRefs } from 'pinia'
import AppButton from '@/components/ui/AppButton.vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import AppModeToggle from '@/components/ui/AppModeToggle.vue'
import { useDescriptionDraft } from '@/features/catalog/composables/useDescriptionDraft'
import { useSelectedProjectTasks } from '@/features/catalog/composables/useSelectedProjectTasks'
import { useSelectedProjectComponents } from '@/features/project/composables/useSelectedProjectComponents'
import { useSelectedProjectConnections } from '@/features/connection/composables/useSelectedProjectConnections'
import { usePreferencesStore } from '@/features/settings/store/preferences.store'
import { columnMaxWidth } from '@/features/task/model/task-width'
import { useConsoleStore } from '@/stores/console.store'

/** The description every task of the epic inherits, written in place. It is the main part of the epic, so it takes the width the tasks leave. */
const { selectedTask } = storeToRefs(useConsoleStore())
const preferences = usePreferencesStore()
const { draft, showEditor, mode, scheduleSave, beginWriting } = useDescriptionDraft()

const projectComponents = useSelectedProjectComponents()
const projectConnections = useSelectedProjectConnections()
const projectTasks = useSelectedProjectTasks()

const columnStyle = computed(() => ({ maxWidth: columnMaxWidth(preferences.taskWidth) }))
</script>

<template>
  <section v-if="selectedTask" class="flex min-h-0 min-w-0 flex-1 flex-col" aria-label="Description" data-testid="epic-description">
    <header class="flex h-11 shrink-0 items-center gap-2.5 border-b border-border px-6">
      <h2 class="eyebrow text-caption">Description</h2>
      <span class="min-w-0 truncate text-micro text-text-subtle">Every task of this epic gets it in its context</span>
      <AppModeToggle v-if="showEditor" v-model="mode" class="ml-auto" testid-prefix="epic-description-mode" />
    </header>

    <div v-if="!showEditor" class="flex items-center gap-3 px-6 py-6">
      <p class="min-w-0 flex-1 text-small leading-relaxed text-text-subtle">
        No description yet. Write the concept the tasks share: it reaches the context of each of them.
      </p>
      <AppButton size="sm" data-testid="epic-write-description" @click="beginWriting">
        <span class="text-accent" aria-hidden="true">+</span> Write it
      </AppButton>
    </div>

    <div v-else class="min-h-0 flex-1 page-scroll p-4">
      <div class="h-full" :style="columnStyle" data-testid="epic-description-column">
        <Transition name="fade-quick" mode="out-in">
          <AppMarkdownEditor
            v-if="mode === 'write'"
            key="write"
            v-model="draft"
            height="100%"
            :page-fullscreen="false"
            :path-root="selectedTask.projectRepoFolder"
            :components="projectComponents"
            :connections="projectConnections"
            :tasks="projectTasks"
            @update:model-value="scheduleSave"
          />
          <AppMarkdownEditor
            v-else
            key="read"
            :model-value="draft"
            :components="projectComponents"
            :connections="projectConnections"
            :tasks="projectTasks"
            readonly
            :read-settings="false"
          />
        </Transition>
      </div>
    </div>
  </section>
</template>
