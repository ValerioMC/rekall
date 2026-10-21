<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppDialog from '@/components/ui/AppDialog.vue'
import AppField from '@/components/ui/AppField.vue'
import AppInput from '@/components/ui/AppInput.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import ConnectionCredentialInput from '@/features/connection/components/ConnectionCredentialInput.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { slugify } from '@/features/catalog/model/catalog'
import { emptyDraft, isComplete, isStarted } from '@/features/connection/lib/credential-draft'
import { CONNECTION_KINDS, CONNECTION_KIND_LABEL } from '@/features/connection/model/connection'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import type {
  Connection,
  ConnectionInput,
  ConnectionKind,
  CredentialDraft
} from '@/features/connection/model/connection'
import type { CompanyId } from '@/model/branded'

/** Creates a company's connection, with its credential if one is given, or edits one's name and URL. */
const props = defineProps<{ companyId: CompanyId; connection: Connection | null }>()
const emit = defineEmits<{ close: []; saved: [connection: Connection] }>()

const store = useConnectionsStore()
const { run, isRunning } = useAsyncAction()
const isNew = props.connection === null

const kind = ref<ConnectionKind>(props.connection?.kind ?? 'KUBERNETES')
const title = ref(props.connection?.title ?? '')
const label = ref(props.connection?.label ?? '')
const baseUrl = ref(props.connection?.baseUrl ?? '')
const credential = ref<CredentialDraft>(emptyDraft(kind.value))
const submitted = ref(false)
const titleInput = ref<HTMLElement | null>(null)

watch(kind, (next) => {
  credential.value = emptyDraft(next)
})

const heading = isNew ? 'New connection' : `Edit ${props.connection?.title}`
const slug = computed(() => slugify(label.value || title.value))
const titleError = computed(() => (submitted.value && !title.value.trim() ? 'A title is required.' : null))
const urlError = computed(() =>
  submitted.value && kind.value === 'GITLAB' && !baseUrl.value.trim()
    ? 'The GitLab instance URL is required.'
    : null
)
const credentialError = computed(() =>
  submitted.value && isStarted(credential.value) && !isComplete(credential.value)
    ? 'Read the kubeconfig and choose a context, or leave it empty to set it later.'
    : null
)
const canSave = computed(
  () => !!title.value.trim() && !!slug.value && !(kind.value === 'GITLAB' && !baseUrl.value.trim())
)

function input(): ConnectionInput {
  const typed = { title: title.value.trim(), label: slug.value }
  const withUrl = kind.value === 'GITLAB' ? { ...typed, baseUrl: baseUrl.value.trim() } : typed
  return isNew ? { ...withUrl, kind: kind.value } : withUrl
}

async function save(): Promise<void> {
  if (isRunning.value) return
  submitted.value = true
  if (!canSave.value || credentialError.value) return
  const sending = isStarted(credential.value) ? credential.value : null
  credential.value = emptyDraft(kind.value)
  const saved = await run(
    () =>
      props.connection
        ? store.update(props.connection.id, input())
        : store.create(props.companyId, input(), sending),
    `${isNew ? 'Created' : 'Saved'} ${title.value.trim()}`
  )
  if (saved) {
    emit('saved', saved)
    emit('close')
  }
}

onMounted(async () => {
  await nextTick()
  titleInput.value?.querySelector('input')?.focus()
})
</script>

<template>
  <AppDialog
    :label="heading"
    panel-class="max-w-[600px]"
    panel-testid="connection-dialog"
    @dismiss="emit('close')"
  >
    <header class="flex items-center gap-3 border-b border-border px-6 py-4">
      <span class="min-w-0 flex-1 text-title font-semibold tracking-[-0.01em] text-text">{{ heading }}</span>
      <button
        class="focus-ring grid size-7 place-items-center rounded-control text-text-subtle transition-colors hover:bg-surface-raised hover:text-text"
        aria-label="Close"
        @click="emit('close')"
      >
        <CloseGlyph />
      </button>
    </header>

    <form class="max-h-[66vh] overflow-y-auto px-6 py-5" @submit.prevent="save">
      <div v-if="isNew" class="mb-4">
        <p class="mb-1.5 eyebrow text-caption">Kind</p>
        <div class="flex gap-1.5">
          <button
            v-for="option in CONNECTION_KINDS"
            :key="option"
            type="button"
            class="focus-ring h-8 rounded-control border px-3 text-small transition-colors"
            :class="
              kind === option
                ? 'border-accent bg-accent-soft text-accent'
                : 'border-border-strong bg-canvas text-text-muted hover:text-text'
            "
            :aria-pressed="kind === option"
            :data-testid="`kind-${option}`"
            @click="kind = option"
          >
            {{ CONNECTION_KIND_LABEL[option] }}
          </button>
        </div>
      </div>

      <div ref="titleInput">
        <AppField v-slot="{ fieldId, describedBy }" label="Title" required :error="titleError">
          <AppInput
            :id="fieldId"
            v-model="title"
            :described-by="describedBy"
            data-testid="connection-title"
            placeholder="Production cluster"
          />
        </AppField>
      </div>

      <AppField
        v-slot="{ fieldId, describedBy }"
        label="Label"
        hint="What a session names it by: kubectl --context and rekall-gitlab --connection."
      >
        <AppInput
          :id="fieldId"
          v-model="label"
          :described-by="describedBy"
          mono
          data-testid="connection-label"
          :placeholder="slug || 'prod'"
        />
      </AppField>

      <AppField
        v-if="kind === 'GITLAB'"
        v-slot="{ fieldId, describedBy }"
        label="Instance URL"
        required
        :error="urlError"
      >
        <AppInput
          :id="fieldId"
          v-model="baseUrl"
          :described-by="describedBy"
          mono
          data-testid="connection-url"
          placeholder="https://gitlab.example.com"
        />
      </AppField>

      <AppField
        v-if="isNew"
        :label="kind === 'GITLAB' ? 'Personal access token' : 'Kubeconfig'"
        :hint="
          kind === 'GITLAB'
            ? 'Optional now; stored in this machine\'s credential store.'
            : 'Optional now. A user that runs a command or a password is refused.'
        "
        :error="credentialError"
      >
        <template #default="{ fieldId }">
          <ConnectionCredentialInput :id="fieldId" v-model="credential" />
        </template>
      </AppField>
    </form>

    <footer class="flex items-center justify-end gap-2 border-t border-border bg-canvas px-6 py-3.5">
      <AppButton size="sm" variant="ghost" @click="emit('close')">Cancel</AppButton>
      <AppButton size="sm" variant="primary" :loading="isRunning" data-testid="save-connection" @click="save">
        {{ isNew ? 'Create connection' : 'Save' }}
      </AppButton>
    </footer>
  </AppDialog>
</template>
