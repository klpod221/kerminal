<template>
  <pre
    class="syntax-highlight"
    :class="className"
    v-html="highlightedCode"
  ></pre>
</template>

<script setup lang="ts">
import { computed } from "vue";

interface Props {
  code: string;
  language?: "shell" | "bash" | "json" | "text";
  className?: string;
}

const props = withDefaults(defineProps<Props>(), {
  language: "shell",
  className: "",
});

const escapeHtml = (text: string): string => {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#039;");
};

const highlightShell = (code: string): string => {
  let highlighted = escapeHtml(code);

  // Comments
  highlighted = highlighted.replace(
    /(#.*)$/gm,
    '<span class="token-comment">$1</span>'
  );

  // Strings (double and single quotes)
  highlighted = highlighted.replace(
    /("(?:[^"\\]|\\.)*")/g,
    '<span class="token-string">$1</span>'
  );
  highlighted = highlighted.replace(
    /('(?:[^'\\]|\\.)*')/g,
    '<span class="token-string">$1</span>'
  );

  // Variables ($VAR, ${VAR})
  highlighted = highlighted.replace(
    /(\$\{[^}]+\}|\$[a-zA-Z_][a-zA-Z0-9_]*)/g,
    '<span class="token-variable">$1</span>'
  );

  // Keywords (common shell commands and keywords)
  const keywords = [
    "sudo",
    "apt",
    "apt-get",
    "yum",
    "dnf",
    "brew",
    "npm",
    "yarn",
    "pnpm",
    "docker",
    "git",
    "cd",
    "ls",
    "mkdir",
    "rm",
    "cp",
    "mv",
    "chmod",
    "chown",
    "cat",
    "grep",
    "find",
    "sed",
    "awk",
    "curl",
    "wget",
    "ssh",
    "scp",
    "tar",
    "zip",
    "unzip",
    "ps",
    "kill",
    "systemctl",
    "service",
    "if",
    "then",
    "else",
    "elif",
    "fi",
    "for",
    "while",
    "do",
    "done",
    "case",
    "esac",
    "function",
    "export",
    "source",
    "echo",
    "printf",
    "read",
    "exit",
    "return",
  ];

  keywords.forEach((keyword) => {
    const regex = new RegExp(`\\b(${keyword})\\b`, "g");
    highlighted = highlighted.replace(
      regex,
      '<span class="token-keyword">$1</span>'
    );
  });

  // Operators and pipes
  highlighted = highlighted.replace(
    /(&amp;&amp;|\|\||&gt;&gt;|&gt;|&lt;|\||;)/g,
    '<span class="token-operator">$1</span>'
  );

  // Numbers
  highlighted = highlighted.replace(
    /\b(\d+)\b/g,
    '<span class="token-number">$1</span>'
  );

  // Flags (-flag, --flag)
  highlighted = highlighted.replace(
    /(\s)(--?[a-zA-Z0-9-]+)/g,
    '$1<span class="token-flag">$2</span>'
  );

  return highlighted;
};

const highlightJson = (code: string): string => {
  let highlighted = escapeHtml(code);

  // Strings
  highlighted = highlighted.replace(
    /("(?:[^"\\]|\\.)*")(\s*:)/g,
    '<span class="token-property">$1</span>$2'
  );
  highlighted = highlighted.replace(
    /:\s*("(?:[^"\\]|\\.)*")/g,
    ': <span class="token-string">$1</span>'
  );

  // Numbers
  highlighted = highlighted.replace(
    /:\s*(-?\d+\.?\d*)/g,
    ': <span class="token-number">$1</span>'
  );

  // Booleans and null
  highlighted = highlighted.replace(
    /\b(true|false|null)\b/g,
    '<span class="token-keyword">$1</span>'
  );

  return highlighted;
};

const highlightedCode = computed(() => {
  if (!props.code) return "";

  switch (props.language) {
    case "shell":
    case "bash":
      return highlightShell(props.code);
    case "json":
      return highlightJson(props.code);
    default:
      return escapeHtml(props.code);
  }
});
</script>

<style scoped>
.syntax-highlight {
  margin: 0;
  padding: 0.5rem;
  font-family: "FiraCode Nerd Font", "Courier New", monospace;
  font-size: 0.75rem;
  line-height: 1.5;
  white-space: pre-wrap;
  word-wrap: break-word;
  overflow-x: auto;
  color: #e5e7eb;
}

.syntax-highlight :deep(.token-comment) {
  color: #6b7280;
  font-style: italic;
}

.syntax-highlight :deep(.token-string) {
  color: #86efac;
}

.syntax-highlight :deep(.token-variable) {
  color: #fbbf24;
}

.syntax-highlight :deep(.token-keyword) {
  color: #60a5fa;
  font-weight: 500;
}

.syntax-highlight :deep(.token-operator) {
  color: #f472b6;
}

.syntax-highlight :deep(.token-number) {
  color: #c084fc;
}

.syntax-highlight :deep(.token-flag) {
  color: #fdba74;
}

.syntax-highlight :deep(.token-property) {
  color: #818cf8;
}
</style>

