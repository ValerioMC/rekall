<script setup lang="ts">
import { ref } from 'vue'
import { storeToRefs } from 'pinia'
import FormPanel from '@/components/ui/FormPanel.vue'
import FormRow from '@/components/ui/FormRow.vue'
import SegmentedControl, { type SegmentOption } from '@/components/ui/SegmentedControl.vue'
import { preferredSessionKind, setPreferredSessionKind } from '@/common/config/claude-launch'
import { preferredProvider, setPreferredProvider } from '@/common/config/session-launch'
import { SESSION_VIEWS, sessionViewChoiceLabel, type SessionView } from '@/features/claude/model/claude'
import { SESSION_PROVIDERS, SESSION_PROVIDER_LABEL, type SessionProvider } from '@/features/claude/model/session-provider'
import { useClaudeConnectionStore } from '@/features/claude/store/claudeConnection.store'

const PROVIDER_OPTIONS: readonly SegmentOption<SessionProvider>[] = SESSION_PROVIDERS.map((value) => ({
  value,
  label: SESSION_PROVIDER_LABEL[value]
}))
const VIEW_OPTIONS: readonly SegmentOption<SessionView>[] = SESSION_VIEWS.map((value) => ({
  value,
  label: sessionViewChoiceLabel(value)
}))

const { installation } = storeToRefs(useClaudeConnectionStore())

const provider = ref(preferredProvider())

function chooseProvider(choice: SessionProvider): void {
  provider.value = choice
  setPreferredProvider(choice)
}

const sessionView = ref(preferredSessionKind())

function chooseSessionView(choice: SessionView): void {
  sessionView.value = choice
  setPreferredSessionKind(choice)
}
</script>

<template>
  <FormPanel
    title="Both providers"
    description="Claude Code and Codex share these."
    aria-label="Both providers"
    data-testid="shared-session-settings"
  >
    <FormRow
      label="Default provider"
      hint="Starts new sessions. The switch beside Run here picks another one for a single task."
    >
      <SegmentedControl
        :model-value="provider"
        :options="PROVIDER_OPTIONS"
        label="Default provider"
        data-testid="default-provider"
        @update:model-value="chooseProvider"
      />
    </FormRow>
    <FormRow
      label="Session view"
      hint="Chat shows the conversation and approval requests full-size, under the open task tabs. The run queue has its own view."
    >
      <SegmentedControl
        :model-value="sessionView"
        :options="VIEW_OPTIONS"
        label="Session view"
        data-testid="session-view"
        @update:model-value="chooseSessionView"
      />
    </FormRow>
    <FormRow label="MCP server" hint="Where Rekall serves its tools. Connecting either provider registers this address.">
      <code class="min-w-0 break-all font-mono text-small text-text" data-testid="mcp-endpoint">
        {{ installation?.endpoint ?? 'Not read yet' }}
      </code>
    </FormRow>
  </FormPanel>
</template>
