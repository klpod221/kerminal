// Copyright (C) 2026 Bùi Thanh Xuân (klpod221)
// SPDX-License-Identifier: GPL-3.0-or-later

export interface SecretFinding {
  type: string;
  description: string;
  match: string;
  startIndex: number;
  endIndex: number;
  recommendation: string;
}

interface SecretRule {
  type: string;
  description: string;
  regex: RegExp;
  recommendation: string;
}

const SECRET_RULES: SecretRule[] = [
  {
    type: "Private Key",
    description: "Embedded cryptographic private key",
    regex: /-----BEGIN\s+(?:RSA|OPENSSH|DSA|EC|PGP)?\s*PRIVATE\s+KEY[^-]*-----[\s\S]*?-----END[^-]*-----/gi,
    recommendation: "Use SSH keys managed via Kerminal SSH Key Manager or ssh-agent.",
  },
  {
    type: "AWS Access Key",
    description: "Amazon Web Services Access Key ID",
    regex: /\b(AKIA[0-9A-Z]{16})\b/g,
    recommendation: "Use AWS credentials file or environment variables ($AWS_ACCESS_KEY_ID).",
  },
  {
    type: "GitHub Token",
    description: "GitHub Personal Access Token or App Token",
    regex: /\b(gh[pousr]_[A-Za-z0-9_]{36,82}|github_pat_[A-Za-z0-9_]{82})\b/g,
    recommendation: "Store tokens in system credential helper or environment variables.",
  },
  {
    type: "Generic API Key / Token",
    description: "Hardcoded API key or bearer token",
    regex: /(?:api[_-]?key|token|auth|bearer)\s*[:=]\s*["']?([a-zA-Z0-9_\-\.]{24,})["']?/gi,
    recommendation: "Replace with environment variable (e.g., $API_KEY or {{API_KEY}}).",
  },
  {
    type: "OpenAI / LLM API Key",
    description: "OpenAI or similar AI provider secret key",
    regex: /\b(sk-[a-zA-Z0-9]{20,48})\b/g,
    recommendation: "Use environment variable (export OPENAI_API_KEY=...).",
  },
  {
    type: "Inline Password",
    description: "Plain text password assignment or flag in command",
    regex: /(?:(?:password|passwd|pass|pwd)\s*[:=]\s*["']?([^\s"']{4,})["']?|(?:-p|--password)[=\s]["']?([^\s"']{4,})["']?)/gi,
    recommendation: "Prompt for password interactively or use credentials manager.",
  },
  {
    type: "Database Connection URI with Password",
    description: "Database connection string containing credentials",
    regex: /(?:postgres|postgresql|mysql|mongodb|redis|amqp):\/\/[^:\s]+:([^@\s]+)@[^\s]+/gi,
    recommendation: "Use connection profiles or vault secrets instead of raw connection strings.",
  },
];

/**
 * Scan a command string for potential embedded secrets or credentials
 */
export function scanForSecrets(commandText: string): SecretFinding[] {
  if (!commandText || commandText.trim().length === 0) return [];

  const findings: SecretFinding[] = [];

  for (const rule of SECRET_RULES) {
    const regex = new RegExp(rule.regex);
    let match: RegExpExecArray | null;

    while ((match = regex.exec(commandText)) !== null) {
      const fullMatch = match[0];
      findings.push({
        type: rule.type,
        description: rule.description,
        match: fullMatch.length > 30 ? `${fullMatch.slice(0, 15)}...${fullMatch.slice(-10)}` : fullMatch,
        startIndex: match.index,
        endIndex: match.index + fullMatch.length,
        recommendation: rule.recommendation,
      });

      // Avoid infinite loop on zero-width match
      if (regex.lastIndex === match.index) {
        regex.lastIndex++;
      }
    }
  }

  return findings;
}

/**
 * Mask detected secrets by replacing them with safe {{PLACEHOLDER}} syntax
 */
export function maskSecrets(commandText: string): string {
  let masked = commandText;

  // Mask private keys
  masked = masked.replace(
    /-----BEGIN\s+(?:RSA|OPENSSH|DSA|EC|PGP)?\s*PRIVATE\s+KEY[^-]*-----[\s\S]*?-----END[^-]*-----/gi,
    "{{PRIVATE_KEY}}",
  );

  // Mask AWS Keys
  masked = masked.replace(/\b(AKIA[0-9A-Z]{16})\b/g, "{{AWS_ACCESS_KEY_ID}}");

  // Mask GitHub tokens
  masked = masked.replace(
    /\b(gh[pousr]_[A-Za-z0-9_]{36,82}|github_pat_[A-Za-z0-9_]{82})\b/g,
    "{{GITHUB_TOKEN}}",
  );

  // Mask OpenAI keys
  masked = masked.replace(/\b(sk-[a-zA-Z0-9]{20,48})\b/g, "{{OPENAI_API_KEY}}");

  // Mask inline passwords
  masked = masked.replace(
    /((?:password|passwd|pass|pwd)\s*[:=]\s*["']?)[^\s"']+(["']?)/gi,
    "$1{{PASSWORD}}$2",
  );
  masked = masked.replace(
    /(-p|--password)[=\s]["']?[^\s"']+["']?/gi,
    "$1 \"{{PASSWORD}}\"",
  );

  // Mask database URIs
  masked = masked.replace(
    /((?:postgres|postgresql|mysql|mongodb|redis|amqp):\/\/[^:\s]+:)[^@\s]+(@[^\s]+)/gi,
    "$1{{DB_PASSWORD}}$2",
  );

  return masked;
}
