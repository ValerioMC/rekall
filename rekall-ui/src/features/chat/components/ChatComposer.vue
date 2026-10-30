<script setup lang="ts">
import { computed, ref, useId } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import { matchingCommands, type SlashCommand } from '@/features/chat/lib/slash-commands'
import type { ComposerState } from '@/features/chat/lib/chat-status'

/**
 * The message field. Enter sends, Shift+Enter breaks the line, and a line starting with `/` offers
 * the commands that work headless; choosing one only fills the field. A failed send leaves the
 * text where it was, so the parent clears it with `clear()` once the send succeeded.
 */
const props = defineProps<{
  state: ComposerState
  sending: boolean
  stopping: boolean
  error: string | null
  followingNext: boolean
}>()

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
const commands = computed(() => (listDismissed.value || locked.value ? [] : matchingCommands(text.value)))
const hint = computed(() =>
  props.state === 'ended' && props.followingNext ? 'Session ended. Opening the next session…' : HINTS[props.state]
)

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
  <div class="relative shrink-0 border-t border-border bg-surface px-4 py-3 max-[420px]:px-3">
    <ul
      v-if="commands.length > 0"
      :id="listId"
      class="absolute inset-x-4 bottom-[calc(100%-4px)] z-(--z-overlay) overflow-hidden rounded-card border border-border-strong bg-surface-raised p-1 shadow-modal"
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
        class="field min-w-0 flex-1 resize-none rounded-control px-3 py-2 font-sans text-body leading-relaxed"
        :rows="rows"
        :disabled="locked"
        aria-label="Message to Claude"
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
      <AppButton v-if="state === 'working'" variant="danger" :disabled="stopping" data-testid="chat-stop" @click="emit('stop')">
        {{ stopping ? 'Stopping…' : 'Stop' }}
      </AppButton>
      <AppButton variant="primary" :disabled="!canSend" :loading="sending" data-testid="chat-send" @click="submit">Send</AppButton>
    </div>

    <p :id="`${listId}-hint`" class="mt-1.5 text-micro text-text-subtle" aria-live="polite" data-testid="chat-hint">{{ hint }}</p>
    <p v-if="error" class="mt-1 text-caption text-danger" role="alert" data-testid="chat-send-error">{{ error }}</p>
  </div>
</template>
