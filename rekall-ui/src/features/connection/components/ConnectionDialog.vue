<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppDialog from '@/components/ui/AppDialog.vue'
import AppField from '@/components/ui/AppField.vue'
import AppInput from '@/components/ui/AppInput.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import ConnectionCredentialInput from '@/features/connection/components/ConnectionCredentialInput.vue'
import ConnectionKindIcon from '@/features/connection/components/ConnectionKindIcon.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { slugify } from '@/features/catalog/model/catalog'
import { emptyDraft, isComplete, isStarted } from '@/features/connection/lib/credential-draft'
import { CREDENTIAL_FIELD, TAKES_PROXY, URL_FIELD } from '@/features/connection/lib/connection-fields'
import { CONNECTION_KINDS, CONNECTION_KIND_LABEL } from '@/features/connection/model/connection'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import type {
  Connection,
  ConnectionInput,
  ConnectionKind,
  CredentialDraft
} from '@/features/connection/model/connection'
import type { CompanyId } from '@/model/branded'

/** Creates a company's connection, with its credential if one is given, or edits one's name, URL and proxy. */
const props = defineProps<{
  companyId: CompanyId
  connection: Connection | null
  initialKind?: ConnectionKind | null
}>()
const emit = defineEmits<{ close: []; saved: [connection: Connection] }>()

const store = useConnectionsStore()
const { run, isRunning } = useAsyncAction()
const isNew = props.connection === null

const kind = ref<ConnectionKind>(props.connection?.kind ?? props.initialKind ?? 'KUBERNETES')
const title = ref(props.connection?.title ?? '')
const label = ref(props.connection?.label ?? '')
const baseUrl = ref(props.connection?.baseUrl ?? '')
const proxyUrl = ref(props.connection?.proxyUrl ?? '')
const credential = ref<CredentialDraft>(emptyDraft(kind.value))
const submitted = ref(false)
const titleInput = ref<HTMLElement | null>(null)

watch(kind, (next) => {
  credential.value = emptyDraft(next)
})

const heading = isNew ? 'New connection' : `Edit ${props.connection?.title}`
const slug = computed(() => slugify(label.value || title.value))
const titleError = computed(() => (submitted.value && !title.value.trim() ? 'A title is required.' : null))
const urlField = computed(() => URL_FIELD[kind.value])
const credentialField = computed(() => CREDENTIAL_FIELD[kind.value])
const takesProxy = computed(() => TAKES_PROXY[kind.value])
const urlMissing = computed(() => urlField.value !== null && !baseUrl.value.trim())
const urlError = computed(() =>
  submitted.value && urlMissing.value ? `The ${urlField.value?.label} is required.` : null
)
const credentialError = computed(() =>
  submitted.value && isStarted(credential.value) && !isComplete(credential.value)
    ? credentialField.value.incomplete
    : null
)
const canSave = computed(() => !!title.value.trim() && !!slug.value && !urlMissing.value)

function input(): ConnectionInput {
  const typed = { title: title.value.trim(), label: slug.value }
  const withUrl = urlField.value ? { ...typed, baseUrl: baseUrl.value.trim() } : typed
  const withProxy = takesProxy.value ? { ...withUrl, proxyUrl: proxyUrl.value.trim() } : withUrl
  return isNew ? { ...withProxy, kind: kind.value } : withProxy
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
            class="focus-ring inline-flex h-9 cursor-pointer items-center gap-2 rounded-control border px-3 text-small transition-colors"
            :class="
              kind === option
                ? 'border-accent bg-accent-soft text-accent'
                : 'border-border-strong bg-canvas text-text-muted hover:text-text'
            "
            :aria-pressed="kind === option"
            :data-testid="`kind-${option}`"
            @click="kind = option"
          >
            <ConnectionKindIcon :kind="option" :size="16" />
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
        hint="What a session names it by: kubectl --context, rekall-gitlab --connection and rekall-jira --connection."
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
        v-if="urlField"
        v-slot="{ fieldId, describedBy }"
        :label="urlField.title"
        required
        :error="urlError"
      >
        <AppInput
          :id="fieldId"
          v-model="baseUrl"
          :described-by="describedBy"
          mono
          data-testid="connection-url"
          :placeholder="urlField.placeholder"
        />
      </AppField>

      <AppField
        v-if="takesProxy"
        v-slot="{ fieldId, describedBy }"
        label="Proxy URL"
        hint="Optional. Check now, the issue import and rekall-jira all reach the site through it."
      >
        <AppInput
          :id="fieldId"
          v-model="proxyUrl"
          :described-by="describedBy"
          mono
          data-testid="connection-proxy"
          placeholder="http://proxy.example.com:8080"
        />
      </AppField>

      <AppField
        v-if="isNew"
        :label="credentialField.title"
        :hint="credentialField.hint"
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
