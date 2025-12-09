type FormValues = { [key: string]: unknown };

type ValidatorFn = (value: unknown, params: string[], allValues: FormValues) => string | undefined;

const validators: Record<string, ValidatorFn> = {
  required: (value) => {
    if (
      value === null ||
      value === undefined ||
      (typeof value === "string" && value.trim() === "") ||
      (Array.isArray(value) && value.length === 0)
    ) {
      return "This field is required.";
    }
  },
  min: (value, params) => {
    if (String(value).length < Number(params[0])) {
      return `Must be at least ${params[0]} characters.`;
    }
  },
  max: (value, params) => {
    if (String(value).length > Number(params[0])) {
      return `Must not exceed ${params[0]} characters.`;
    }
  },
  between: (value, params) => {
    const len = String(value).length;
    if (len < Number(params[0]) || len > Number(params[1])) {
      return `Must be between ${params[0]} and ${params[1]} characters.`;
    }
  },
  password: (value) => {
    const passwordRegex = /^(?=.*[a-z])(?=.*[A-Z])(?=.*\d).{8,}$/;
    if (!passwordRegex.test(String(value))) {
      return "Password must be at least 8 characters long and include uppercase, lowercase letters, and numbers.";
    }
  },
  same: (value, params, allValues) => {
    const otherValue = allValues[params[0]];
    if (value !== otherValue) {
      return `Values must match with ${params[0]}.`;
    }
  },
  different: (value, params, allValues) => {
    const otherValue = allValues[params[0]];
    if (value === otherValue) {
      return `Values must be different from ${params[0]}.`;
    }
  },
};

export function validate(
  value: unknown,
  rules: string,
  allValues: FormValues,
): string {
  const ruleParts = rules.split("|");

  for (const rule of ruleParts) {
    if (!rule) continue;

    const [ruleName, ...params] = rule.split(":");
    const validator = validators[ruleName];

    if (validator) {
      const error = validator(value, params, allValues);
      if (error) return error;
    }
  }

  return ""; // No errors
}
