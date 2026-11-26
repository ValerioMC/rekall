<script setup lang="ts">
import { computed, onUnmounted, ref, watch } from 'vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import AppModeToggle from '@/components/ui/AppModeToggle.vue'
import ExportAction from '@/features/export/components/ExportAction.vue'
import ProjectPaneFrame from '@/features/project/components/ProjectPaneFrame.vue'
import ProjectReadingColumn from '@/features/project/components/ProjectReadingColumn.vue'
import TaskWidthMenu from '@/features/task/components/TaskWidthMenu.vue'
import { usePreferencesStore } from '@/features/settings/store/preferences.store'
import { identityHue } from '@/common/identity'
import { useConsoleStore } from '@/stores/console.store'
import { useEditorMode } from '@/composables/useEditorMode'
import type { ExportSelection } from '@/features/export/model/project-export'
import type { Project } from '@/features/catalog/model/catalog'

const SAVE_DELAY_MS = 700

const props = defineProps<{ project: Project }>()

const store = useConsoleStore()
const preferences = usePreferencesStore()
const draft = ref('')
const { mode, openFor } = useEditorMode('read')
let timer: ReturnType<typeof setTimeout> | null = null

function schedule(): void {
  if (timer) clearTimeout(timer)
  timer = setTimeout(() => {
    timer = null
    void store.saveProjectBlueprint(props.project.id, draft.value)
  }, SAVE_DELAY_MS)
}

async function settle(): Promise<void> {
  if (!timer) return
  clearTimeout(timer)
  timer = null
  await store.saveProjectBlueprint(props.project.id, draft.value)
}

function flush(): void {
  void settle()
}

const blueprintOnly: ExportSelection = { blueprint: true, sectionIds: [], componentIds: [] }
const emptyReason = computed(() => (draft.value.trim() === '' ? 'The blueprint is empty.' : null))

watch(
  () => props.project.id,
  (_, previous) => {
    if (previous !== undefined) flush()
    draft.value = props.project.blueprintMarkdown ?? ''
    openFor(draft.value)
  },
  { immediate: true }
)

onUnmounted(flush)
</script>

<template>
  <ProjectPaneFrame label="Blueprint" testid="blueprint-pane">
    <template #header>
      <div
        class="texture-grid pointer-events-none absolute inset-0"
        :style="{ '--texture-tint': identityHue(project.id).base }"
        aria-hidden="true"
      />
      <div class="relative px-5 py-3">
        <div class="flex items-center gap-3">
          <h2 class="min-w-0 flex-1 truncate text-title font-semibold tracking-[-0.01em] text-text">Blueprint</h2>
          <div class="flex shrink-0 items-center gap-2">
            <ExportAction :project-id="project.id" :selection="blueprintOnly" :empty-reason="emptyReason" :settle="settle" />
            <TaskWidthMenu v-model="preferences.taskWidth" />
            <AppModeToggle v-model="mode" testid-prefix="blueprint-mode" />
          </div>
        </div>
        <p class="mt-0.5 truncate text-small text-text-subtle">
          What <code class="text-anchor/80">/rk {{ project.anchor }}</code> hands to Claude
        </p>
      </div>
    </template>
    <template #default="{ editorHeight }">
      <Transition name="fade-quick" mode="out-in">
        <ProjectReadingColumn v-if="mode === 'write'" key="write">
          <AppMarkdownEditor
            v-model="draft"
            :height="editorHeight"
            :path-root="project.repoFolder"
            :picture-owner="{ kind: 'BLUEPRINT', id: project.id }"
            placeholder="# What this is

How it's built, how it's organised, the conventions to follow while working in it."
            @update:model-value="schedule"
          />
        </ProjectReadingColumn>
        <AppMarkdownEditor v-else key="read" :model-value="draft" readonly :read-settings="false" />
      </Transition>
    </template>
  </ProjectPaneFrame>
</template>
