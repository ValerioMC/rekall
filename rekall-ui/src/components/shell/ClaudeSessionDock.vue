<script setup lang="ts">
import { computed, ref } from 'vue'
import { storeToRefs } from 'pinia'
import { useRoute, useRouter } from 'vue-router'
import { useConsoleStore } from '@/stores/console.store'
import { useClaudeStore } from '@/stores/claude.store'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { identityHue } from '@/common/identity'
import { claudeStatusLabel, type ClaudeSession } from '@/model/claude'
import type { ClaudeSessionId } from '@/model/branded'

const store = useConsoleStore()
const claude = useClaudeStore()
const { liveSessions } = storeToRefs(claude)
const { paneFocus } = storeToRefs(store)
const { run } = useAsyncAction()
const route = useRoute()
const router = useRouter()

const expanded = ref(false)

const onSessionPane = computed(() => route.name === 'console' && paneFocus.value === 'claude')

const visible = computed(() => liveSessions.value.length > 0 && !onSessionPane.value)

const sorted = computed(() =>
  [...liveSessions.value].sort((a, b) => b.startedAt.localeCompare(a.startedAt))
)

const working = computed(
  () => sorted.value.filter((s) => s.status === 'WORKING' || s.status === 'STARTING').length
)

function projectIdOf(session: ClaudeSession): string {
  return store.tasks.find((t) => t.id === session.taskId)?.projectId ?? session.taskId
}

function jumpTo(session: ClaudeSession): void {
  store.selectTask(session.taskId)
  store.openClaude()
  void claude.selectSession(session.id)
  expanded.value = false
  if (router.currentRoute.value.name !== 'console') void router.push({ name: 'console' })
}

async function stop(id: ClaudeSessionId): Promise<void> {
  await run(() => claude.stop(id))
}
</script>

<template>
  <div
    v-if="visible"
    class="dock-lane-above fixed right-4 z-(--z-sticky) flex flex-col items-end gap-2"
    data-testid="claude-dock"
  >
    <div
      v-if="expanded"
      class="rise w-[300px] overflow-hidden rounded-[var(--radius-card)] border border-border-strong bg-surface shadow-modal"
    >
      <header class="flex items-center justify-between border-b border-border px-3.5 py-2.5">
        <span class="eyebrow flex items-center gap-1.5">
          <span class="h-2.5 w-[3px] shrink-0 rounded-full bg-anchor" aria-hidden="true" />
          Sessions
        </span>
        <button
          class="focus-ring grid size-6 place-items-center rounded-full text-text-subtle transition-colors hover:bg-surface-raised hover:text-text"
          aria-label="Collapse"
          @click="expanded = false"
        >
          &times;
        </button>
      </header>

      <ul class="max-h-[280px] overflow-y-auto p-1.5">
        <li v-for="session in sorted" :key="session.id" data-testid="claude-dock-row">
          <div class="flex items-center gap-2.5 rounded-[var(--radius-control)] px-2 py-2 hover:bg-surface-raised">
            <button
              class="focus-ring flex min-w-0 flex-1 items-center gap-2.5 rounded-[var(--radius-control)] text-left"
              :title="`Jump to ${session.taskTitle}`"
              data-testid="claude-dock-jump"
              @click="jumpTo(session)"
            >
              <span
                class="session-caret shrink-0"
                :class="{ 'session-caret-busy': session.status === 'WORKING' || session.status === 'STARTING' }"
                aria-hidden="true"
              />
              <span class="min-w-0 flex-1">
                <span class="block truncate text-[12.5px] font-medium text-text">
                  {{ session.taskTitle }}
                </span>
                <span class="mt-0.5 flex items-center gap-1.5 truncate font-mono text-[10px] text-anchor/80">
                  <span
                    class="size-1 shrink-0 rounded-full"
                    :style="{ backgroundColor: identityHue(projectIdOf(session)).base }"
                    aria-hidden="true"
                  />
                  {{ session.anchor }}
                </span>
              </span>
            </button>
            <span class="shrink-0 font-mono text-[10.5px] text-text-subtle">
              {{ claudeStatusLabel(session.status) }}
            </span>
            <button
              class="focus-ring grid size-6 shrink-0 place-items-center rounded-full text-text-subtle transition-colors hover:bg-danger-soft hover:text-danger"
              :aria-label="`Stop ${session.taskTitle}`"
              data-testid="claude-dock-stop"
              @click="stop(session.id)"
            >
              <svg class="size-3" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
                <rect x="4" y="4" width="8" height="8" rx="1.5" />
              </svg>
            </button>
          </div>
        </li>
      </ul>
    </div>

    <button
      class="focus-ring glass flex h-10 items-center gap-2.5 rounded-full border border-border-strong pl-3.5 pr-3.5 shadow-lift transition-colors hover:border-anchor/50"
      data-testid="claude-dock-toggle"
      :aria-expanded="expanded"
      @click="expanded = !expanded"
    >
      <span
        class="session-caret shrink-0"
        :class="{ 'session-caret-busy': working }"
        aria-hidden="true"
      />
      <span class="text-[12.5px] font-medium text-text">
        {{ liveSessions.length }} <span class="font-normal text-text-subtle">{{ liveSessions.length === 1 ? 'session' : 'sessions' }}</span>
      </span>
      <span v-if="working" class="font-mono text-[10.5px] text-anchor">
        {{ working }} working
      </span>
    </button>
  </div>
</template>
