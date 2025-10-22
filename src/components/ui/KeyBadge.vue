<template>
  <div class="inline-flex items-center">
    <template v-for="(key, index) in keys" :key="index">
      <kbd
        class="inline-flex items-center justify-center min-w-[24px] h-6 px-2 text-xs font-medium text-gray-300 bg-gray-800 border border-gray-600 rounded shadow-sm"
        :class="[
          size === 'sm'
            ? 'min-w-[20px] h-5 text-xs'
            : 'min-w-[24px] h-6 text-xs',
          variant === 'light'
            ? 'bg-gray-700 border-gray-500 text-gray-200'
            : 'bg-gray-800 border-gray-600 text-gray-300',
        ]"
      >
        {{ formatKey(key) }}
      </kbd>
      <span
        v-if="index < keys.length - 1 && showSeparator"
        class="text-gray-500 text-xs font-medium mx-1"
      >
        +
      </span>
    </template>
  </div>
</template>

<script setup lang="ts">
interface KeyBadgeProps {
  keys: string[];
  size?: "sm" | "md";
  variant?: "default" | "light";
  showSeparator?: boolean;
}

withDefaults(defineProps<KeyBadgeProps>(), {
  size: "md",
  variant: "default",
  showSeparator: true,
});

/**
 * Format key for display with VSCode-style symbols
 */
const formatKey = (key: string): string => {
  const keyMap: Record<string, string> = {
    Ctrl: "Ctrl",
    Control: "Ctrl",
    Alt: "Alt",
    Shift: "Shift",
    Meta: "Cmd",
    Command: "Cmd",
    ArrowUp: "↑",
    ArrowDown: "↓",
    ArrowLeft: "←",
    ArrowRight: "→",
    Space: "Space",
    Enter: "Enter",
    Tab: "Tab",
    Backspace: "Backspace",
    Delete: "Delete",
    Escape: "Esc",
    PageUp: "PgUp",
    PageDown: "PgDn",
    Home: "Home",
    End: "End",
    Insert: "Ins",
    F1: "F1",
    F2: "F2",
    F3: "F3",
    F4: "F4",
    F5: "F5",
    F6: "F6",
    F7: "F7",
    F8: "F8",
    F9: "F9",
    F10: "F10",
    F11: "F11",
    F12: "F12",
    "0": "0",
    "1": "1",
    "2": "2",
    "3": "3",
    "4": "4",
    "5": "5",
    "6": "6",
    "7": "7",
    "8": "8",
    "9": "9",
    "\\": "\\",
    "-": "-",
    "=": "=",
    "[": "[",
    "]": "]",
    ";": ";",
    "'": "'",
    ",": ",",
    ".": ".",
    "/": "/",
    "`": "`",
  };

  return keyMap[key] || key.toUpperCase();
};
</script>

<style scoped>
kbd {
  box-shadow:
    0 1px 0 0 rgb(255 255 255 / 0.1) inset,
    0 1px 2px 0 rgb(0 0 0 / 0.3);
  background: linear-gradient(180deg, #374151 0%, #1f2937 100%);
  border: 1px solid #4b5563;
  position: relative;
}

kbd:before {
  content: "";
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 1px;
  background: linear-gradient(
    90deg,
    transparent,
    rgba(255, 255, 255, 0.1),
    transparent
  );
}

/* VSCode-style key styling */
.variant-light kbd {
  background: linear-gradient(180deg, #6b7280 0%, #374151 100%);
  border-color: #9ca3af;
  color: #f3f4f6;
}
</style>
