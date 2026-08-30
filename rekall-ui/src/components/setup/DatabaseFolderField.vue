<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import RestartingOverlay from '@/components/setup/RestartingOverlay.vue'
import { useDatabaseSetup } from '@/composables/useDatabaseSetup'
import { desktopHost, pickFolder } from '@/common/native/desktop'

/**
 * Type a folder, see what will happen to it, commit.
 *
 * A browser never hands a page a real filesystem path, only a sandboxed handle a JDBC URL cannot
 * use. So the path is typed, and made trustworthy with the one thing a picker would have given
 * for free: live feedback on what is actually at that path before anything commits to it.
 *
 * The macOS application has a real NSOpenPanel to open and installs a bridge to it, so there the
 * folder icon is a button and typing becomes the fallback rather than the only way in. Both
 * paths end in the same string in the same field, checked by the same request.
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

/** Fixed for the lifetime of the page: the bridge is installed before the application boots. */
const canBrowse = desktopHost() !== null

/**
 * A dismissed panel leaves what was typed alone. Focus goes back to the field either way, so the
 * chosen path can be corrected by hand without reaching for the mouse again.
 */
async function browse(): Promise<void> {
  const chosen = await pickFolder(path.value)
  if (chosen) onInput(chosen)
  input.value?.focus()
}

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
      return {
        tone: 'danger',
        text: "Rekall can't write to this folder. Check its permissions and try again."
      }
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
        <!-- The icon is centred on the input alone. While the help text below shared this box,
             it was centred on the pair and sat visibly below the line of the path it labels. -->
        <div class="relative">
          <label for="database-folder" class="sr-only">Database folder</label>
          <component
            :is="canBrowse ? 'button' : 'span'"
            :type="canBrowse ? 'button' : undefined"
            :aria-label="canBrowse ? 'Choose a folder' : undefined"
            :aria-hidden="canBrowse ? undefined : 'true'"
            :title="canBrowse ? 'Choose a folder' : undefined"
            data-testid="folder-browse"
            class="absolute left-1 top-1/2 grid size-8 -translate-y-1/2 place-items-center rounded-[var(--radius-control)] text-text-subtle transition-colors"
            :class="canBrowse ? 'focus-ring hover:bg-surface-hover hover:text-accent' : 'pointer-events-none'"
            @click="browse"
          >
            <svg class="size-4" viewBox="0 0 16 16" fill="none" aria-hidden="true">
              <path
                d="M2 4.5A1.5 1.5 0 0 1 3.5 3h2.6l1.2 1.5H12.5A1.5 1.5 0 0 1 14 6v6a1.5 1.5 0 0 1-1.5 1.5h-9A1.5 1.5 0 0 1 2 12Z"
                stroke="currentColor"
                stroke-width="1.3"
                stroke-linejoin="round"
              />
            </svg>
          </component>
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
            class="focus-ring h-10 w-full rounded-[var(--radius-control)] border bg-canvas pl-9 pr-3.5 font-mono text-[13px] text-text outline-none transition-colors placeholder:text-text-subtle hover:border-border-strong focus-visible:border-accent"
            :class="hint?.tone === 'danger' ? 'border-danger' : 'border-border'"
            @input="onInput(($event.target as HTMLInputElement).value)"
            @keydown.enter="submit"
          />
        </div>
        <p id="database-folder-help" class="mt-1.5 text-[11.5px] text-text-subtle">
          <template v-if="canBrowse"> Click the folder to pick one, or type a full path. </template>
          <template v-else>
            A full path to an existing folder. Browsers can't hand a page a real folder picker, so this is
            typed rather than browsed.
          </template>
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
