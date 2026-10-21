<script setup lang="ts">
import { computed, ref } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import { useConnectionsStore } from '@/features/connection/store/connections.store'
import { KUBE_USER_KIND_LABEL } from '@/features/connection/model/connection'
import type { CredentialDraft, KubeContextOption } from '@/features/connection/model/connection'

type KubeDraft = Extract<CredentialDraft, { kind: 'KUBERNETES' }>

const props = defineProps<{ modelValue: KubeDraft }>()
const emit = defineEmits<{ 'update:modelValue': [value: KubeDraft] }>()

const store = useConnectionsStore()
const contexts = ref<readonly KubeContextOption[]>([])
const reading = ref(false)
const failure = ref<string | null>(null)
const fileInput = ref<HTMLInputElement | null>(null)

const hasText = computed(() => props.modelValue.kubeconfig.trim().length > 0)

function setText(kubeconfig: string): void {
  contexts.value = []
  failure.value = null
  emit('update:modelValue', { kind: 'KUBERNETES', kubeconfig, context: null })
}

async function pickFile(event: Event): Promise<void> {
  const file = (event.target as HTMLInputElement).files?.[0]
  if (!file) return
  setText(await file.text())
  if (fileInput.value) fileInput.value.value = ''
  await readContexts()
}

/** Asks the server for the file's contexts; nothing is stored until the connection is saved. */
async function readContexts(): Promise<void> {
  if (!hasText.value || reading.value) return
  reading.value = true
  failure.value = null
  try {
    contexts.value = await store.previewKubeconfig(props.modelValue.kubeconfig)
    const [only, ...others] = contexts.value.filter((context) => context.supported)
    if (only && !others.length) choose(only.name)
    if (!contexts.value.length) failure.value = 'This kubeconfig has no context.'
  } catch (error) {
    contexts.value = []
    failure.value = error instanceof Error ? error.message : 'The kubeconfig could not be read.'
  } finally {
    reading.value = false
  }
}

function choose(name: string): void {
  emit('update:modelValue', { ...props.modelValue, context: name })
}
</script>

<template>
  <div class="flex flex-col gap-2" data-testid="kubeconfig-import">
    <textarea
      :value="modelValue.kubeconfig"
      data-testid="kubeconfig-text"
      rows="5"
      spellcheck="false"
      autocomplete="off"
      class="field text-text w-full resize-y rounded-control p-3 font-mono text-caption leading-relaxed"
      placeholder="Paste a kubeconfig, or choose its file"
      @input="setText(($event.target as HTMLTextAreaElement).value)"
    />
    <div class="flex items-center gap-2">
      <input ref="fileInput" type="file" class="sr-only" data-testid="kubeconfig-file" @change="pickFile" />
      <AppButton size="sm" variant="ghost" @click="fileInput?.click()">Choose file</AppButton>
      <AppButton
        size="sm"
        variant="secondary"
        :disabled="!hasText"
        :loading="reading"
        data-testid="read-contexts"
        @click="readContexts"
      >
        Read contexts
      </AppButton>
      <span class="text-caption text-text-subtle"
        >Only the context you choose is kept, with its files inlined.</span
      >
    </div>
    <p v-if="failure" class="text-caption text-danger" data-testid="kubeconfig-failure">{{ failure }}</p>
    <ul v-if="contexts.length" class="flex flex-col gap-1" role="radiogroup" aria-label="Context to keep">
      <li v-for="context in contexts" :key="context.name">
        <button
          type="button"
          role="radio"
          :aria-checked="modelValue.context === context.name"
          :disabled="!context.supported"
          :data-testid="`context-${context.name}`"
          class="focus-ring flex w-full flex-col items-start rounded-control border px-3 py-2 text-left transition-colors disabled:cursor-not-allowed"
          :class="
            modelValue.context === context.name
              ? 'border-accent bg-accent-soft'
              : context.supported
                ? 'border-border-strong bg-canvas hover:border-text-subtle'
                : 'border-border bg-canvas opacity-70'
          "
          @click="choose(context.name)"
        >
          <span class="flex w-full items-baseline gap-2">
            <span class="font-mono text-small text-text">{{ context.name }}</span>
            <span class="text-caption text-text-subtle">{{ KUBE_USER_KIND_LABEL[context.userKind] }}</span>
            <span v-if="context.namespace" class="ml-auto font-mono text-caption text-text-subtle"
              >ns {{ context.namespace }}</span
            >
          </span>
          <span v-if="context.server" class="font-mono text-caption text-text-muted">{{
            context.server
          }}</span>
          <span v-if="context.refusal" class="mt-0.5 text-caption text-danger">{{ context.refusal }}</span>
        </button>
      </li>
    </ul>
  </div>
</template>
