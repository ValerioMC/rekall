<script setup lang="ts">
import { onUnmounted, ref, watch } from 'vue'
import AppMarkdownEditor from '@/components/ui/AppMarkdownEditor.vue'
import AppModeToggle from '@/components/ui/AppModeToggle.vue'
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
  <section class="flex min-h-0 flex-col" aria-label="Blueprint" data-testid="blueprint-pane">
    <header class="relative overflow-hidden rounded-t-[var(--radius-card)] border border-b-0 border-border bg-surface">
      <div
        class="texture-grid pointer-events-none absolute inset-0"
        :style="{ '--texture-tint': identityHue(project.id).base }"
        aria-hidden="true"
      />
      <div class="relative flex items-center gap-3 px-5 py-3.5">
        <div class="min-w-0 flex-1">
          <h2 class="text-[15px] font-semibold tracking-[-0.01em] text-text">Blueprint</h2>
          <p class="mt-0.5 truncate text-[12px] text-text-subtle">
            What <code class="text-anchor/80">/rk {{ project.anchor }}</code> hands to Claude
          </p>
        </div>
        <AppModeToggle v-model="mode" testid-prefix="blueprint-mode" />
      </div>
    </header>
    <div class="rounded-b-[var(--radius-card)] border border-border bg-surface p-4">
      <Transition name="fade-quick" mode="out-in">
        <AppMarkdownEditor
          v-if="mode === 'write'"
          key="write"
          v-model="draft"
          height="520px"
          :path-root="project.repoFolder"
          placeholder="# What this is

How it's built, how it's organised, the conventions to follow while working in it."
          @update:model-value="schedule"
        />
        <AppMarkdownEditor v-else key="read" :model-value="draft" readonly />
      </Transition>
    </div>
  </section>
</template>
