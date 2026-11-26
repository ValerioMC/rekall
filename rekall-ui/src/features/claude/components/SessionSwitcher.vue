<script setup lang="ts">
import { ref } from 'vue'
import SessionProviderMark from '@/features/claude/components/SessionProviderMark.vue'
import { SESSION_PROVIDER_LABEL } from '@/features/claude/model/session-provider'
import { identityHue } from '@/common/identity'
import { sessionSubject } from '@/common/format/session-title'
import type { Terminal } from '@/features/claude/model/terminal'

/** The other live sessions of a full-size view, and the way to put one of them in it. */
defineProps<{ sessions: readonly Terminal[] }>()

const emit = defineEmits<{ switch: [session: Terminal] }>()

const open = ref(false)

function switchTo(session: Terminal): void {
  open.value = false
  emit('switch', session)
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key !== 'Escape' || !open.value) return
  event.stopPropagation()
  open.value = false
}
</script>

<template>
  <div class="relative" @keydown="onKeydown">
    <button
      type="button"
      class="rounded-key focus-ring inline-flex h-7 cursor-pointer items-center gap-1.5 px-2.5 text-caption"
      :class="open ? 'key-lit' : 'key-quiet'"
      data-testid="other-sessions"
      aria-haspopup="true"
      :aria-expanded="open"
      @click="open = !open"
    >
      <span class="session-caret shrink-0" aria-hidden="true" />
      Other {{ sessions.length }} {{ sessions.length === 1 ? 'session' : 'sessions' }}
    </button>

    <div
      v-if="open"
      class="rise absolute right-0 top-[calc(100%+6px)] z-(--z-overlay) w-[300px] overflow-hidden rounded-card border border-border-strong bg-surface-raised shadow-modal"
      data-testid="other-sessions-list"
    >
      <ul class="max-h-[280px] overflow-y-auto p-1.5">
        <li v-for="session in sessions" :key="session.id">
          <button
            type="button"
            class="focus-ring flex w-full min-w-0 cursor-pointer items-center gap-2.5 rounded-control px-2 py-2 text-left transition-colors hover:bg-surface"
            :title="`Switch to ${sessionSubject(session)}`"
            data-testid="other-sessions-row"
            @click="switchTo(session)"
          >
            <SessionProviderMark :provider="session.provider" :size="14" />
            <span class="min-w-0 flex-1">
              <span class="block truncate text-small font-medium text-text">{{ sessionSubject(session) }}</span>
              <span class="mt-0.5 flex items-center gap-1.5 truncate font-mono text-micro text-anchor/80">
                <span
                  class="size-1 shrink-0 rounded-full"
                  :style="{ backgroundColor: identityHue(session.projectId).base }"
                  aria-hidden="true"
                />
                <span class="min-w-0 truncate">{{ session.anchors }}</span>
                <span class="shrink-0 font-sans text-caption text-text-muted">{{ SESSION_PROVIDER_LABEL[session.provider] }}</span>
              </span>
            </span>
          </button>
        </li>
      </ul>
    </div>

    <button
      v-if="open"
      type="button"
      class="fixed inset-0 z-(--z-sticky) cursor-default"
      tabindex="-1"
      aria-hidden="true"
      @click="open = false"
    />
  </div>
</template>
