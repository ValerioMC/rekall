<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { useToastStore } from '@/stores/toast.store'

const store = useToastStore()
const { toasts } = storeToRefs(store)
</script>

<template>
  <div class="pointer-events-none fixed bottom-6 right-6 z-(--z-toast) flex w-full max-w-sm flex-col gap-2">
    <TransitionGroup
      enter-active-class="transition duration-200 ease-out"
      enter-from-class="translate-y-2 opacity-0"
      leave-active-class="transition duration-150 ease-in"
      leave-to-class="translate-y-1 opacity-0"
    >
      <div
        v-for="toast in toasts"
        :key="toast.id"
        data-testid="toast"
        class="pointer-events-auto flex items-start gap-3 rounded-[var(--radius-card)] border p-3.5 text-[13px] shadow-lift backdrop-blur"
        :class="
          toast.kind === 'error'
            ? 'border-danger/40 bg-danger-soft text-text'
            : 'border-safe/40 bg-safe-soft text-text'
        "
        role="status"
      >
        <span
          class="mt-1.5 size-1.5 shrink-0 rounded-full"
          :class="toast.kind === 'error' ? 'bg-danger' : 'bg-safe'"
          aria-hidden="true"
        />
        <p class="flex-1 leading-snug">{{ toast.message }}</p>
        <button
          class="focus-ring shrink-0 rounded text-text-subtle transition-colors hover:text-text"
          aria-label="Dismiss"
          @click="store.dismiss(toast.id)"
        >
          <svg viewBox="0 0 14 14" fill="none" class="size-3.5">
            <path d="M3 3l8 8M11 3l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
          </svg>
        </button>
      </div>
    </TransitionGroup>
  </div>
</template>
