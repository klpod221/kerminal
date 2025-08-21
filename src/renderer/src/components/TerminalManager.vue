<template>
  <div class="w-full h-full relative">
    <Terminal
      v-for="terminal in terminals"
      :key="terminal.id"
      :ref="(el) => setTerminalRef(terminal.id, el)"
      :terminal-id="terminal.id"
      :class="{ hidden: terminal.id !== activeTerminalId }"
      class="w-full h-full absolute inset-0"
      @terminal-ready="onTerminalReady"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import Terminal from './Terminal.vue'

interface TerminalInstance {
  id: string
  ready: boolean
}

interface TerminalComponent extends ComponentPublicInstance {
  focus: () => void
}

interface Props {
  terminals: TerminalInstance[]
  activeTerminalId?: string
}

const props = defineProps<Props>()

import type { ComponentPublicInstance } from 'vue'

const terminalRefs = ref<Record<string, ComponentPublicInstance | null>>({})

const emit = defineEmits(['terminal-ready'])

/**
 * Set the ref for a terminal instance.
 * Only store Vue component instances, not DOM elements.
 * @param {string} terminalId - The terminal id.
 * @param {any} el - The ref value (component instance or DOM element).
 */
const setTerminalRef = (terminalId: string, el: ComponentPublicInstance | Element | null): void => {
  // Check if el is a Vue component instance (has $el property)
  if (el && typeof el === 'object' && '$el' in el) {
    terminalRefs.value[terminalId] = el as ComponentPublicInstance
  } else {
    delete terminalRefs.value[terminalId]
  }
}

const onTerminalReady = (terminalId: string): void => {
  emit('terminal-ready', terminalId)
}

// Watch for active terminal changes to ensure proper focus
watch(
  () => props.activeTerminalId,
  async (newActiveId) => {
    if (newActiveId && terminalRefs.value[newActiveId]) {
      await nextTick()
      // Focus the active terminal
      const terminalInstance = terminalRefs.value[newActiveId]
      if (terminalInstance && 'focus' in terminalInstance) {
        ;(terminalInstance as TerminalComponent).focus()
      }
    }
  },
  { immediate: true }
)
</script>
