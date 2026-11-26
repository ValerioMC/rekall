<script setup lang="ts">
import { computed, ref, useId } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import { matchingCommands, type SlashCommand } from '@/features/chat/lib/slash-commands'
import type { ComposerState } from '@/features/chat/lib/chat-status'
import { SESSION_PROVIDER_LABEL, type SessionProvider } from '@/features/claude/model/session-provider'

/**
 * The message field. Enter sends, Shift+Enter breaks the line, and a line starting with `/` offers
 * the commands that work headless; choosing one only fills the field. A failed send leaves the
 * text where it was, so the parent clears it with `clear()` once the send succeeded.
 */
const props = withDefaults(defineProps<{
  state: ComposerState
  provider?: SessionProvider
  sending: boolean
  stopping: boolean
  error: string | null
  followingNext: boolean
}>(), { provider: 'CLAUDE' })

const emit = defineEmits<{ send: [text: string]; stop: [] }>()

const MAX_ROWS = 8

const HINTS: Readonly<Record<ComposerState, string>> = {
  idle: 'Enter sends · Shift+Enter new line · / for commands',
  working: 'Claude is working. Your message is read at its next step.',
  paused: 'Paused: waiting for your approval',
  connecting: 'Connecting…',
  reconnecting: 'Reconnecting, nothing is lost',
  ended: 'Session ended'
}

const text = ref('')
const field = ref<HTMLTextAreaElement | null>(null)
const highlighted = ref(0)
const listDismissed = ref(false)
const listId = `commands-${useId()}`

const locked = computed(() => props.state === 'paused' || props.state === 'ended')
const canSend = computed(
  () =>
    !locked.value &&
    props.state !== 'connecting' &&
    props.state !== 'reconnecting' &&
    !props.sending &&
    text.value.trim() !== ''
)
const rows = computed(() => Math.min(MAX_ROWS, text.value.split('\n').length))
const commands = computed(() =>
  props.provider === 'CODEX' || listDismissed.value || locked.value ? [] : matchingCommands(text.value)
)
const hint = computed(() => {
  if (props.state === 'ended' && props.followingNext) return 'Session ended. Opening the next session…'
  if (props.provider === 'CODEX' && props.state === 'idle') return 'Enter sends · Shift+Enter new line'
  if (props.provider === 'CODEX' && props.state === 'working') return 'Your message will run after this turn.'
  return HINTS[props.state]
})

function submit(): void {
  if (!canSend.value) return
  emit('send', text.value.trim())
}

function choose(command: SlashCommand): void {
  text.value = command.insert
  listDismissed.value = true
  field.value?.focus()
}

function onInput(): void {
  listDismissed.value = false
  highlighted.value = 0
}

function moveHighlight(step: number): void {
  highlighted.value = (highlighted.value + step + commands.value.length) % commands.value.length
}

function onListKey(event: KeyboardEvent): boolean {
  if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
    event.preventDefault()
    moveHighlight(event.key === 'ArrowDown' ? 1 : -1)
    return true
  }
  if (event.key === 'Enter' || event.key === 'Tab') {
    const command = commands.value[highlighted.value]!
    if (event.key === 'Enter' && command.insert === text.value) return false
    event.preventDefault()
    choose(command)
    return true
  }
  if (event.key === 'Escape') {
    event.stopPropagation()
    listDismissed.value = true
    return true
  }
  return false
}

function onKeydown(event: KeyboardEvent): void {
  if (event.isComposing) return
  if (commands.value.length > 0 && onListKey(event)) return
  if (event.key !== 'Enter' || event.shiftKey) return
  event.preventDefault()
  submit()
}

function clear(): void {
  text.value = ''
}

function focus(): void {
  field.value?.focus()
}

defineExpose({ focus, clear })
</script>

<template>
  <div class="relative">
    <ul
      v-if="commands.length > 0"
      :id="listId"
      class="absolute inset-x-0 bottom-[calc(100%+4px)] z-(--z-overlay) overflow-hidden rounded-card border border-border-strong bg-surface-raised p-1 shadow-modal"
      role="listbox"
      aria-label="Commands"
      data-testid="chat-commands"
    >
      <li
        v-for="(command, index) in commands"
        :id="`${listId}-${index}`"
        :key="command.name"
        class="flex cursor-pointer items-baseline gap-3 rounded-control px-2 py-1.5"
        :class="index === highlighted ? 'bg-surface text-text' : 'text-text-muted'"
        role="option"
        :aria-selected="index === highlighted"
        @mousedown.prevent="choose(command)"
      >
        <span class="font-mono text-small">{{ command.name }}</span>
        <span class="text-caption text-text-subtle">{{ command.hint }}</span>
      </li>
    </ul>

    <div class="flex items-end gap-2">
      <textarea
        ref="field"
        v-model="text"
        class="field min-w-0 flex-1 resize-none rounded-control px-3 py-2 font-sans text-body leading-normal @min-[560px]/talk:min-h-14"
        :rows="rows"
        :disabled="locked"
        :aria-label="`Message to ${SESSION_PROVIDER_LABEL[provider]}`"
        :aria-describedby="`${listId}-hint`"
        role="combobox"
        aria-autocomplete="list"
        :aria-expanded="commands.length > 0"
        :aria-controls="commands.length > 0 ? listId : undefined"
        :aria-activedescendant="commands.length > 0 ? `${listId}-${highlighted}` : undefined"
        data-testid="chat-input"
        @input="onInput"
        @keydown="onKeydown"
      />
      <AppButton
        v-if="state === 'working'"
        variant="danger"
        class="@min-[560px]/talk:h-9 @min-[560px]/talk:px-4"
        :disabled="stopping"
        data-testid="chat-stop"
        @click="emit('stop')"
      >
        {{ stopping ? 'Stopping…' : 'Stop' }}
      </AppButton>
      <AppButton
        variant="primary"
        class="@min-[560px]/talk:h-9 @min-[560px]/talk:px-4"
        :disabled="!canSend"
        :loading="sending"
        data-testid="chat-send"
        @click="submit"
      >
        Send
      </AppButton>
    </div>

    <p v-if="error" class="mt-1.5 text-caption text-danger" role="alert" data-testid="chat-send-error">{{ error }}</p>
    <!-- The dock keeps the bottom left corner: where the column reaches it, the hint moves right. -->
    <p
      :id="`${listId}-hint`"
      class="mt-1.5 text-micro text-text-subtle @max-[1100px]/talk:mt-1 @max-[1100px]/talk:flex @max-[1100px]/talk:min-h-10 @max-[1100px]/talk:items-center @max-[1100px]/talk:justify-end @max-[1100px]/talk:pl-[140px] @max-[1100px]/talk:text-right"
      aria-live="polite"
      data-testid="chat-hint"
    >
      {{ hint }}
    </p>
  </div>
</template>
