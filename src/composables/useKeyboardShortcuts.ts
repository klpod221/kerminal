import { onMounted, onBeforeUnmount, type Ref } from "vue";

/**
 * Keyboard shortcuts configuration
 */
interface KeyboardShortcutConfig {
  key: string;
  ctrlKey?: boolean;
  altKey?: boolean;
  shiftKey?: boolean;
  metaKey?: boolean;
  preventDefault?: boolean;
  action: () => void;
}

/**
 * Composable for managing keyboard shortcuts
 * @param shortcuts - Array of keyboard shortcut configurations
 * @param enabled - Ref to control if shortcuts are enabled
 */
export function useKeyboardShortcuts(
  shortcuts: KeyboardShortcutConfig[],
  enabled?: Ref<boolean>,
) {
  const handleKeydown = (event: KeyboardEvent): void => {
    if (enabled && !enabled.value) return;

    const target = event.target as HTMLElement;
    const isInputElement =
      target.tagName === "INPUT" ||
      target.tagName === "TEXTAREA" ||
      target.contentEditable === "true";

    if (isInputElement && !target.classList.contains("xterm-helper-textarea")) {
      return;
    }

    for (const shortcut of shortcuts) {
      const keyMatches = event.key.toLowerCase() === shortcut.key.toLowerCase();
      const ctrlMatches = !!shortcut.ctrlKey === event.ctrlKey;
      const altMatches = !!shortcut.altKey === event.altKey;
      const shiftMatches = !!shortcut.shiftKey === event.shiftKey;
      const metaMatches = !!shortcut.metaKey === event.metaKey;

      if (
        keyMatches &&
        ctrlMatches &&
        altMatches &&
        shiftMatches &&
        metaMatches
      ) {
        if (shortcut.preventDefault !== false) {
          event.preventDefault();
          event.stopPropagation();
        }
        shortcut.action();
        break;
      }
    }
  };

  onMounted(() => {
    document.addEventListener("keydown", handleKeydown);
  });

  onBeforeUnmount(() => {
    document.removeEventListener("keydown", handleKeydown);
  });

  return {
    handleKeydown,
  };
}
