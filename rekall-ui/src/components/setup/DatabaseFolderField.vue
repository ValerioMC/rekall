<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import RestartingOverlay from '@/components/setup/RestartingOverlay.vue'
import { useDatabaseSetup } from '@/composables/useDatabaseSetup'

/**
 * Type a folder, see what will happen to it, commit.
 *
 * A native folder-browser dialog cannot exist here: a web page is never handed a real
 * filesystem path by the browser, only a sandboxed handle a JDBC URL cannot use. So this is a
 * plain path field instead, made trustworthy with the one thing a picker would have given for
 * free — live feedback on what is actually at that path before anything commits to it.
 *
 * Self-contained: it owns its own submit, its own restart wait and its own reload. Every host —
 * the first-run wizard, the unreachable-database screen, Settings — just places it and reacts to
 * nothing, because a successful submit ends in a full page reload regardless of where this was.
 */
const props = withDefaults(
  defineProps<{ submitLabel?: string; autofocus?: boolean; cancellable?: boolean }>(),
  { submitLabel: 'Use this folder', autofocus: false, cancellable: false }
)

const emit = defineEmits<{ cancel: []; busy: [boolean] }>()

const { checking, check, phase, error, timedOut, checkPath, submitNewFolder } = useDatabaseSetup()

/**
 * A host that embeds this field inline (Settings, the unreachable-database screen) needs to
 * know when it must not let itself be dismissed: once a submit is in flight, the backend has
 * already committed to restarting, and closing the host only hides that — it does not cancel
 * it, and the reload still happens underneath whatever the user does next.
 */
watch(phase, (value) => emit('busy', value === 'submitting' || value === 'restarting'), { immediate: true })

const path = ref('')
const touched = ref(false)
const input = ref<HTMLInputElement | null>(null)

function onInput(value: string): void {
  path.value = value
  touched.value = true
  checkPath(value)
}

type FieldState = 'empty' | 'checking' | 'missing' | 'file' | 'unusable' | 'existing' | 'new'

const state = computed<FieldState>(() => {
  if (!touched.value || !path.value.trim()) return 'empty'
  if (checking.value) return 'checking'
  if (!check.value) return 'empty'
  if (!check.value.exists) return 'missing'
  if (!check.value.isDirectory) return 'file'
  if (!check.value.writable) return 'unusable'
  return check.value.hasDatabase ? 'existing' : 'new'
})

const canSubmit = computed(() => state.value === 'existing' || state.value === 'new')

const hint = computed<{ tone: 'danger' | 'safe' | 'accent'; text: string } | null>(() => {
  switch (state.value) {
    case 'missing':
      return {
        tone: 'danger',
        text: "This folder doesn't exist. Rekall never creates one on its own — create it, then try again."
      }
    case 'file':
      return { tone: 'danger', text: 'That path points at a file, not a folder.' }
    case 'unusable':
      return { tone: 'danger', text: "Rekall can't write to this folder. Check its permissions and try again." }
    case 'existing':
      return { tone: 'safe', text: 'A database already lives here. It will be opened as is.' }
    case 'new':
      return { tone: 'accent', text: 'No Rekall database here yet. One will be created in this folder.' }
    default:
      return null
  }
})

const submitButtonLabel = computed(() => {
  if (state.value === 'existing') return 'Open this database'
  if (state.value === 'new') return 'Create database here'
  return props.submitLabel
})

async function submit(): Promise<void> {
  if (!canSubmit.value) return
  await submitNewFolder(path.value)
}

function reload(): void {
  window.location.reload()
}

onMounted(() => {
  if (props.autofocus) input.value?.focus()
})

defineExpose({ focus: () => input.value?.focus() })
</script>

<template>
  <div>
    <RestartingOverlay v-if="phase === 'restarting'" />

    <div v-else class="space-y-3">
      <div>
        <label for="database-folder" class="sr-only">Database folder</label>
        <input
          id="database-folder"
          ref="input"
          :value="path"
          type="text"
          spellcheck="false"
          autocomplete="off"
          data-testid="database-folder-input"
          placeholder="/Users/you/Documents/rekall"
          aria-describedby="database-folder-help"
          class="focus-ring h-10 w-full rounded-[var(--radius-control)] border bg-canvas px-3.5 font-mono text-[13px] text-text outline-none transition-colors placeholder:text-text-subtle hover:border-border-strong focus-visible:border-accent"
          :class="hint?.tone === 'danger' ? 'border-danger' : 'border-border'"
          @input="onInput(($event.target as HTMLInputElement).value)"
          @keydown.enter="submit"
        />
        <p id="database-folder-help" class="mt-1.5 text-[11.5px] text-text-subtle">
          A full path to an existing folder. Browsers can't hand a page a real folder picker, so
          this is typed rather than browsed.
        </p>
      </div>

      <!-- What was actually typed can differ from what gets checked, once `~` or a relative
           segment is in play — shown so a decision this hard to undo is never made on a guess. -->
      <p
        v-if="check"
        class="truncate font-mono text-[11px] text-text-subtle"
        :title="check.resolvedPath"
        data-testid="resolved-path"
      >
        &rarr; {{ check.resolvedPath }}
      </p>

      <p
        v-if="hint"
        class="rounded-[var(--radius-control)] border px-3.5 py-2.5 text-[12.5px] leading-relaxed"
        :class="{
          'border-danger bg-danger-soft text-danger': hint.tone === 'danger',
          'border-safe bg-safe-soft text-safe': hint.tone === 'safe',
          'border-accent-deep bg-accent-soft text-accent': hint.tone === 'accent'
        }"
        role="status"
        aria-live="polite"
        data-testid="folder-hint"
      >
        {{ hint.text }}
      </p>
      <p v-else-if="state === 'checking'" class="text-[11.5px] text-text-subtle" aria-live="polite">
        Checking&hellip;
      </p>

      <p v-if="error" class="flex items-center gap-2 text-[12.5px] text-danger" role="alert">
        {{ error }}
        <button
          v-if="timedOut"
          type="button"
          class="focus-ring shrink-0 rounded underline decoration-dotted"
          @click="reload"
        >
          Reload
        </button>
      </p>

      <div class="flex items-center gap-2" :class="props.cancellable ? 'justify-end' : ''">
        <AppButton
          v-if="props.cancellable"
          type="button"
          variant="ghost"
          size="sm"
          :disabled="phase === 'submitting'"
          @click="emit('cancel')"
        >
          Cancel
        </AppButton>
        <AppButton
          type="button"
          variant="primary"
          :disabled="!canSubmit"
          :loading="phase === 'submitting'"
          data-testid="folder-submit"
          @click="submit"
        >
          {{ submitButtonLabel }}
        </AppButton>
      </div>
    </div>
  </div>
</template>
