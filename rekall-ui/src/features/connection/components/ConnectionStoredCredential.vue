<script setup lang="ts">
import { computed, toRef } from 'vue'
import AppButton from '@/components/ui/AppButton.vue'
import CopyGlyph from '@/components/ui/CopyGlyph.vue'
import EyeGlyph from '@/components/ui/EyeGlyph.vue'
import { useRevealedCredential } from '@/features/connection/composables/useRevealedCredential'
import { credentialNoun } from '@/features/connection/lib/credential-draft'
import type { Connection } from '@/features/connection/model/connection'

/** The stored credential, hidden like a password until the eye is pressed, with a button to copy it. */
const props = defineProps<{ connection: Connection }>()

const HIDDEN_MASK = '••••••••••••'

const { revealed, visible, loading, copied, toggle, copy } = useRevealedCredential(toRef(props, 'connection'))

const noun = computed(() => credentialNoun(props.connection.kind))
const secretText = computed(() => revealed.value?.token ?? revealed.value?.kubeconfig ?? '')
const multiline = computed(() => visible.value && secretText.value.includes('\n'))
</script>

<template>
  <div class="flex flex-col gap-2" data-testid="stored-credential">
    <div v-if="revealed?.email" class="flex items-center gap-2">
      <label class="w-14 shrink-0 text-caption text-text-subtle" for="stored-credential-email">Email</label>
      <input
        id="stored-credential-email"
        :value="revealed.email"
        type="text"
        readonly
        class="field text-text h-(--spacing-control) min-w-0 flex-1 rounded-control px-3 text-body"
        data-testid="stored-credential-email"
      />
      <AppButton variant="quiet" size="icon" aria-label="Copy email" title="Copy email" data-testid="copy-stored-email" @click="copy('email')">
        <CopyGlyph :copied="copied === 'email'" />
      </AppButton>
    </div>
    <div class="flex items-start gap-2">
      <label class="w-14 shrink-0 pt-2 text-caption text-text-subtle" for="stored-credential-secret">{{ noun }}</label>
      <textarea
        v-if="multiline"
        id="stored-credential-secret"
        :value="secretText"
        readonly
        rows="8"
        spellcheck="false"
        class="field text-text min-w-0 flex-1 rounded-control px-3 py-2 font-mono text-caption"
        data-testid="stored-credential-secret"
      />
      <input
        v-else
        id="stored-credential-secret"
        :value="visible ? secretText : HIDDEN_MASK"
        :type="visible ? 'text' : 'password'"
        readonly
        autocomplete="off"
        spellcheck="false"
        class="field text-text h-(--spacing-control) min-w-0 flex-1 rounded-control px-3 font-mono text-body"
        data-testid="stored-credential-secret"
      />
      <AppButton
        variant="quiet"
        size="icon"
        :loading="loading"
        :aria-label="visible ? `Hide ${noun}` : `Show ${noun}`"
        :title="visible ? `Hide ${noun}` : `Show ${noun}`"
        data-testid="toggle-stored-credential"
        @click="toggle"
      >
        <EyeGlyph :shown="visible" />
      </AppButton>
      <AppButton
        variant="quiet"
        size="icon"
        :aria-label="`Copy ${noun}`"
        :title="`Copy ${noun}`"
        data-testid="copy-stored-credential"
        @click="copy('secret')"
      >
        <CopyGlyph :copied="copied === 'secret'" />
      </AppButton>
    </div>
  </div>
</template>
