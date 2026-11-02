<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import AppDialog from '@/components/ui/AppDialog.vue'
import AppField from '@/components/ui/AppField.vue'
import AppInput from '@/components/ui/AppInput.vue'
import CloseGlyph from '@/components/ui/CloseGlyph.vue'
import ConnectionCompanyPicker from '@/features/connection/components/ConnectionCompanyPicker.vue'
import ConnectionCredentialInput from '@/features/connection/components/ConnectionCredentialInput.vue'
import ConnectionKindPicker from '@/features/connection/components/ConnectionKindPicker.vue'
import ConnectionRootPathField from '@/features/connection/components/ConnectionRootPathField.vue'
import { useAsyncAction } from '@/composables/useAsyncAction'
import { slugify } from '@/features/catalog/model/catalog'
import { CREDENTIAL_FIELD, ROOT_PATH_FIELD, TAKES_PROXY, URL_FIELD } from '@/features/connection/lib/connection-fields'
import { emptyDraft, isComplete, isStarted } from '@/features/connection/lib/credential-draft'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import type { Connection, ConnectionInput, ConnectionKind, CredentialDraft } from '@/features/connection/model/connection'
import type { CompanyId } from '@/model/branded'

/**
 * Creates a connection, with its credential and the companies it is shared with, or edits one's
 * title, address and root path. On an edit the kind is fixed and companies are changed from the
 * Shared with panel, so the form never offers two ways to do the same thing.
 */
const props = defineProps<{
  connection: Connection | null
  initialKind: ConnectionKind | null
  initialCompanyIds: readonly CompanyId[]
  bindToProject?: string | null
  companies: readonly { readonly id: CompanyId; readonly name: string }[]
}>()
const emit = defineEmits<{ close: []; saved: [connection: Connection]; editCompanies: [] }>()

const store = useConnectionsStore()
const { run, isRunning } = useAsyncAction()
const isNew = props.connection === null

const kind = ref<ConnectionKind>(props.connection?.kind ?? props.initialKind ?? 'KUBERNETES')
const title = ref(props.connection?.title ?? '')
const label = ref(props.connection?.label ?? '')
const baseUrl = ref(props.connection?.baseUrl ?? '')
const proxyUrl = ref(props.connection?.proxyUrl ?? '')
const rootPath = ref(props.connection?.rootPath ?? '')
const companyIds = ref<readonly CompanyId[]>(props.initialCompanyIds)
const credential = ref<CredentialDraft>(emptyDraft(kind.value))
const submitted = ref(false)
const summary = ref<HTMLElement | null>(null)
const titleField = ref<HTMLElement | null>(null)

watch(kind, (next) => {
  credential.value = emptyDraft(next)
  rootPath.value = ''
})

const sharedCompanyNames = computed(() =>
  props.companies.filter((company) => props.initialCompanyIds.includes(company.id)).map((company) => company.name)
)
const heading = isNew ? 'New connection' : `Edit ${props.connection?.title}`
const slug = computed(() => slugify(label.value || title.value))
const urlField = computed(() => URL_FIELD[kind.value])
const credentialField = computed(() => CREDENTIAL_FIELD[kind.value])
const takesProxy = computed(() => TAKES_PROXY[kind.value])
const takesRootPath = computed(() => ROOT_PATH_FIELD[kind.value] !== null)

const titleError = computed(() => (!title.value.trim() ? 'A title is required.' : null))
const urlError = computed(() =>
  urlField.value && !baseUrl.value.trim() ? `The ${urlField.value.label} is required.` : null
)
const labelClash = computed(
  () => store.connections.find((known) => known.label === slug.value && known.id !== props.connection?.id) ?? null
)
const labelError = computed(() => {
  if (!slug.value) return null
  return labelClash.value ? `The label ${slug.value} is already used by ${labelClash.value.title}.` : null
})
const credentialError = computed(() =>
  isStarted(credential.value) && !isComplete(credential.value) ? credentialField.value.incomplete : null
)
const errors = computed(() =>
  [titleError.value, urlError.value, labelError.value, credentialError.value].filter((error): error is string => error !== null)
)

function input(): ConnectionInput {
  const typed = { title: title.value.trim(), label: slug.value }
  const withUrl = urlField.value ? { ...typed, baseUrl: baseUrl.value.trim() } : typed
  const withProxy = takesProxy.value ? { ...withUrl, proxyUrl: proxyUrl.value.trim() } : withUrl
  const withRoot = takesRootPath.value ? { ...withProxy, rootPath: rootPath.value.trim() } : withProxy
  return isNew ? { ...withRoot, kind: kind.value, companyIds: companyIds.value } : withRoot
}

async function save(): Promise<void> {
  if (isRunning.value) return
  submitted.value = true
  if (errors.value.length) {
    await nextTick()
    summary.value?.focus()
    return
  }
  const sending = isStarted(credential.value) ? credential.value : null
  credential.value = emptyDraft(kind.value)
  const saved = await run(
    () => (props.connection ? store.update(props.connection.id, input()) : store.create(input(), sending)),
    `${isNew ? 'Created' : 'Saved'} ${title.value.trim()}`
  )
  if (saved) {
    emit('saved', saved)
    emit('close')
  }
}

onMounted(async () => {
  await nextTick()
  titleField.value?.querySelector('input')?.focus()
})
</script>

<template>
  <AppDialog
    :label="heading"
    placement="right"
    panel-class="sm:max-w-[520px]"
    panel-testid="connection-sheet"
    @dismiss="emit('close')"
  >
    <header class="flex items-center gap-3 border-b border-border px-6 py-4">
      <h2 class="min-w-0 flex-1 truncate text-title font-semibold tracking-[-0.01em] text-text">{{ heading }}</h2>
      <button
        class="focus-ring grid size-11 place-items-center rounded-control text-text-subtle transition-colors hover:bg-surface-raised hover:text-text sm:size-8"
        aria-label="Close"
        @click="emit('close')"
      >
        <CloseGlyph />
      </button>
    </header>

    <form class="min-h-0 flex-1 overflow-y-auto px-6 py-5" novalidate @submit.prevent="save">
      <div
        v-if="submitted && errors.length"
        ref="summary"
        tabindex="-1"
        role="alert"
        class="mb-4 rounded-control border border-danger bg-danger-soft px-3 py-2 text-small text-danger"
        data-testid="error-summary"
      >
        <p class="font-medium">Fix {{ errors.length }} {{ errors.length === 1 ? 'thing' : 'things' }} to save:</p>
        <ul class="list-disc pl-5">
          <li v-for="error in errors" :key="error">{{ error }}</li>
        </ul>
      </div>

      <ConnectionKindPicker v-if="isNew" v-model="kind" />

      <div ref="titleField">
        <AppField v-slot="{ fieldId, describedBy }" label="Title" required :error="submitted ? titleError : null">
          <AppInput
            :id="fieldId"
            v-model="title"
            :described-by="describedBy"
            data-testid="connection-title"
            placeholder="Production cluster"
          />
        </AppField>
      </div>

      <AppField v-if="urlField" v-slot="{ fieldId, describedBy }" :label="urlField.title" required :error="submitted ? urlError : null">
        <AppInput
          :id="fieldId"
          v-model="baseUrl"
          :described-by="describedBy"
          mono
          data-testid="connection-url"
          :placeholder="urlField.placeholder"
        />
      </AppField>

      <ConnectionRootPathField v-if="takesRootPath" :key="kind" v-model="rootPath" :kind="kind" />

      <AppField
        v-if="takesProxy"
        v-slot="{ fieldId, describedBy }"
        label="Proxy URL"
        hint="Optional. Check, the issue import and rekall-jira all reach the site through it."
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

      <AppField v-if="isNew" :label="credentialField.title" :hint="`${credentialField.hint} Set it later from the connection.`" :error="submitted ? credentialError : null">
        <template #default="{ fieldId }">
          <ConnectionCredentialInput :id="fieldId" v-model="credential" />
        </template>
      </AppField>

      <section v-if="isNew" class="mb-4" aria-labelledby="sheet-sharing-heading">
        <h3 id="sheet-sharing-heading" class="mb-1.5 text-small font-medium text-text-muted">Shared with</h3>
        <p v-if="bindToProject" class="text-small text-text" data-testid="sheet-bind-note">
          Will be shared with {{ sharedCompanyNames.join(', ') }} and bound to {{ bindToProject }}.
        </p>
        <template v-else>
          <ConnectionCompanyPicker v-model="companyIds" :companies="companies" />
          <p v-if="!companyIds.length" class="mt-1.5 text-caption text-warn" data-testid="sheet-unshared-note">
            Not shared yet. No project can use it.
          </p>
          <p v-else class="mt-1.5 text-caption text-text-subtle">A project can bind the connection only if its company is in this list.</p>
        </template>
      </section>
      <p v-else class="mb-4 text-small text-text-muted">
        Companies are changed in
        <button type="button" class="focus-ring rounded-key text-accent underline-offset-2 hover:underline" data-testid="sheet-edit-companies" @click="emit('editCompanies')">
          Shared with</button
        >.
      </p>

      <details class="mb-2" :open="submitted && !!labelError">
        <summary class="focus-ring cursor-pointer rounded-key text-small font-medium text-text-muted">Label and advanced settings</summary>
        <AppField
          v-slot="{ fieldId, describedBy }"
          class="mt-3"
          label="Label"
          hint="The short name sessions use for this connection: kubectl --context, rekall-gitlab, rekall-github and rekall-jira --connection. Made from the title when left empty."
          :error="submitted ? labelError : null"
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
      </details>
    </form>

    <footer class="flex items-center justify-end gap-2 border-t border-border bg-canvas px-6 py-3.5">
      <AppButton size="sm" variant="ghost" class="max-sm:min-h-11" @click="emit('close')">Cancel</AppButton>
      <AppButton size="sm" variant="primary" class="max-sm:min-h-11" :loading="isRunning" data-testid="save-connection" @click="save">
        {{ isNew ? 'Create connection' : 'Save' }}
      </AppButton>
    </footer>
  </AppDialog>
</template>
