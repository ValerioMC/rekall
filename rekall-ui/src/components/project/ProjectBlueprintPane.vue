<script setup lang="ts">
import { onUnmounted, ref, watch } from 'vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import AppModeToggle from '@/components/ui/AppModeToggle.vue'
import ProjectPaneFrame from '@/components/project/ProjectPaneFrame.vue'
import { identityHue } from '@/common/identity'
import { useConsoleStore } from '@/stores/console.store'
import type { Project } from '@/model/catalog'

const SAVE_DELAY_MS = 700

const props = defineProps<{ project: Project }>()

const store = useConsoleStore()
const draft = ref('')
const mode = ref<'write' | 'read'>('read')
let timer: ReturnType<typeof setTimeout> | null = null

function schedule(): void {
  if (timer) clearTimeout(timer)
  timer = setTimeout(() => {
    timer = null
    void store.saveProjectBlueprint(props.project.id, draft.value)
  }, SAVE_DELAY_MS)
}

function flush(): void {
  if (!timer) return
  clearTimeout(timer)
  timer = null
  void store.saveProjectBlueprint(props.project.id, draft.value)
}

watch(
  () => props.project.id,
  (_, previous) => {
    if (previous !== undefined) flush()
    draft.value = props.project.blueprintMarkdown ?? ''
    mode.value = draft.value.trim() ? 'read' : 'write'
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
      <div class="relative flex items-center gap-3 px-5 py-3">
        <div class="min-w-0 flex-1">
          <h2 class="text-title font-semibold tracking-[-0.01em] text-text">Blueprint</h2>
          <p class="mt-0.5 truncate text-small text-text-subtle">
            What <code class="text-anchor/80">/rk {{ project.anchor }}</code> hands to Claude
          </p>
        </div>
        <AppModeToggle v-model="mode" testid-prefix="blueprint-mode" />
      </div>
    </template>
    <template #default="{ editorHeight }">
      <Transition name="fade-quick" mode="out-in">
        <AppMarkdownEditor
          v-if="mode === 'write'"
          key="write"
          v-model="draft"
          :height="editorHeight"
          :path-root="project.repoFolder"
          placeholder="# What this is

How it's built, how it's organised, the conventions to follow while working in it."
          @update:model-value="schedule"
        />
        <AppMarkdownEditor v-else key="read" :model-value="draft" readonly />
      </Transition>
    </template>
  </ProjectPaneFrame>
</template>
