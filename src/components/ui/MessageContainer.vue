<template>
  <Teleport to="body">
    <div
      class="fixed top-8 left-1/2 transform -translate-x-1/2 z-100 flex flex-col items-center gap-3 pointer-events-none"
    >
      <TransitionGroup
        enter-active-class="transition-all duration-300 ease-out"
        enter-from-class="opacity-0 transform -translate-y-4 scale-95"
        enter-to-class="opacity-100 transform translate-y-0 scale-100"
        leave-active-class="transition-all duration-200 ease-in absolute"
        leave-from-class="opacity-100 transform translate-y-0 scale-100"
        leave-to-class="opacity-0 transform -translate-y-4 scale-95"
        move-class="transition-all duration-300 ease-in-out"
      >
        <Message
          v-for="msg in messages"
          :key="msg.id"
          v-bind="msg"
          class="pointer-events-auto"
          @close="remove(msg.id)"
        />
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref } from "vue";
import Message from "./Message.vue";
import type { MessageOptions } from "../../utils/message";

interface MessageItem extends MessageOptions {
  id: number;
  onClose?: () => void;
}

const messages = ref<MessageItem[]>([]);
let seed = 0;

const add = (options: MessageOptions) => {
  const id = seed++;
  const msg: MessageItem = {
    ...options,
    id,
  };
  messages.value.push(msg);
  // Note: Message component handles its own timer to emit close event
};

const remove = (id: number) => {
  const index = messages.value.findIndex((m) => m.id === id);
  if (index !== -1) {
    // Call the onClose callback if it exists
    const msg = messages.value[index];
    if (msg.onClose) {
      msg.onClose();
    }
    messages.value.splice(index, 1);
  }
};

defineExpose({
  add,
});
</script>
