<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="fixed top-8 left-1/2 transform -translate-x-1/2 z-50"
    >
      <Transition
        enter-active-class="transition-all duration-300 ease-out"
        enter-from-class="opacity-0 transform -translate-y-4 scale-95"
        enter-to-class="opacity-100 transform translate-y-0 scale-100"
        leave-active-class="transition-all duration-200 ease-in"
        leave-from-class="opacity-100 transform translate-y-0 scale-100"
        leave-to-class="opacity-0 transform -translate-y-4 scale-95"
      >
        <div
          v-if="visible"
          :class="[
            'flex items-center gap-3 px-4 py-3 rounded-lg shadow-2xl border backdrop-blur-sm',
            'min-w-[300px] max-w-[500px]',
            messageClasses,
          ]"
        >
          <!-- Icon -->
          <div class="flex-shrink-0">
            <div class="rounded-lg p-2" :class="iconBackgroundClass">
              <component :is="iconComponent" :size="16" :class="iconClass" />
            </div>
          </div>

          <!-- Content -->
          <div class="flex-1 min-w-0">
            <div v-if="title" class="font-semibold text-sm text-white mb-1">
              {{ title }}
            </div>
            <div class="text-sm text-gray-300 leading-5">{{ content }}</div>
          </div>

          <!-- Close button -->
          <Button
            v-if="closable"
            variant="ghost"
            size="sm"
            :icon="X"
            class="flex-shrink-0 !p-1.5"
            @click="close"
          />
        </div>
      </Transition>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import {
  CheckCircle,
  XCircle,
  AlertTriangle,
  Info,
  Loader2,
  X,
} from "lucide-vue-next";
import Button from "./Button.vue";

interface MessageProps {
  type?: "success" | "error" | "warning" | "info" | "loading";
  title?: string;
  content: string;
  duration?: number;
  closable?: boolean;
  onClose?: () => void;
}

const props = withDefaults(defineProps<MessageProps>(), {
  type: "info",
  duration: 3000,
  closable: true,
});

const visible = ref(false);
let timer: number | null = null;

const messageClasses = computed(() => {
  switch (props.type) {
    case "success":
      return "bg-[#1a1a1a] border-green-700/50";
    case "error":
      return "bg-[#1a1a1a] border-red-700/50";
    case "warning":
      return "bg-[#1a1a1a] border-yellow-700/50";
    case "loading":
      return "bg-[#1a1a1a] border-blue-700/50";
    default:
      return "bg-[#1a1a1a] border-blue-700/50";
  }
});

const iconBackgroundClass = computed(() => {
  switch (props.type) {
    case "success":
      return "bg-green-500/20";
    case "error":
      return "bg-red-500/20";
    case "warning":
      return "bg-yellow-500/20";
    case "loading":
      return "bg-blue-500/20";
    default:
      return "bg-blue-500/20";
  }
});

const iconClass = computed(() => {
  switch (props.type) {
    case "success":
      return "text-green-400";
    case "error":
      return "text-red-400";
    case "warning":
      return "text-yellow-400";
    case "loading":
      return "text-blue-400 animate-spin";
    default:
      return "text-blue-400";
  }
});

const iconComponent = computed(() => {
  switch (props.type) {
    case "success":
      return CheckCircle;
    case "error":
      return XCircle;
    case "warning":
      return AlertTriangle;
    case "loading":
      return Loader2;
    default:
      return Info;
  }
});

/**
 * Close the message
 */
const close = (): void => {
  visible.value = false;
  if (timer) {
    window.clearTimeout(timer);
    timer = null;
  }
  setTimeout(() => {
    props.onClose?.();
  }, 200);
};

onMounted(() => {
  visible.value = true;

  if (props.duration > 0 && props.type !== "loading") {
    timer = window.setTimeout(() => {
      close();
    }, props.duration);
  }
});

defineExpose({
  close,
});
</script>
