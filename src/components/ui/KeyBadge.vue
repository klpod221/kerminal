<template>
  <div class="inline-flex items-center gap-1.5">
    <template v-for="(key, index) in keys" :key="index">
      <kbd
        class="key-badge inline-flex items-center justify-center font-medium text-white transition-all duration-200"
        :class="[
          size === 'sm'
            ? 'min-w-[20px] h-5 px-1.5 text-[10px] leading-tight'
            : 'min-w-[24px] h-6 px-2 text-xs leading-tight',
          variant === 'light' ? 'key-badge-light' : 'key-badge-default',
        ]"
      >
        {{ formatKey(key) }}
      </kbd>
      <span
        v-if="index < keys.length - 1 && showSeparator"
        class="text-gray-400 text-xs font-medium leading-none"
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
.key-badge {
  background: #374151;
  border: 1px solid #4b5563;
  border-radius: 0.375rem;
  user-select: none;
  font-family:
    -apple-system, BlinkMacSystemFont, "Segoe UI", "Roboto", "Oxygen", "Ubuntu",
    "Cantarell", "Fira Sans", "Droid Sans", "Helvetica Neue", sans-serif;
  letter-spacing: 0.025em;
  box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.2);
  transition:
    border-color 0.2s ease,
    background-color 0.2s ease,
    box-shadow 0.2s ease;
}

/* Light variant */
.key-badge-light {
  background: #4b5563;
  border-color: #6b7280;
}
</style>
