const TOKEN_PATTERN = /(\{[^}]+\})/g;

const DIRECT_SYMBOLS = {
  W: "ms-w",
  U: "ms-u",
  B: "ms-b",
  R: "ms-r",
  G: "ms-g",
  C: "ms-c",
  S: "ms-s",
  X: "ms-x",
  Y: "ms-y",
  Z: "ms-z",
  T: "ms-tap",
  Q: "ms-untap",
  E: "ms-e",
  CHAOS: "ms-chaos",
  P: "ms-p",
};

function tokenToClass(token) {
  const normalized = token.replace(/[{}]/g, "").trim().toUpperCase();
  if (!normalized) {
    return null;
  }

  if (DIRECT_SYMBOLS[normalized]) {
    return DIRECT_SYMBOLS[normalized];
  }

  if (/^\d+$/.test(normalized)) {
    return `ms-${normalized}`;
  }

  if (normalized === "HALF") {
    return "ms-half";
  }

  if (normalized === "INFINITY") {
    return "ms-infinity";
  }

  const parts = normalized.split("/");
  if (parts.length === 2) {
    const [first, second] = parts;
    if (second === "P" && /^[WUBRG]$/.test(first)) {
      return `ms-${first.toLowerCase()}p`;
    }
    if ((first === "2" || first === "C") && /^[WUBRG]$/.test(second)) {
      return `ms-${first.toLowerCase()}${second.toLowerCase()}`;
    }
    if (/^[WUBRG]{2}$/.test(first + second)) {
      return `ms-${first.toLowerCase()}${second.toLowerCase()}`;
    }
  }

  if (parts.length === 3) {
    const [first, second, third] = parts;
    if (third === "P" && /^[WUBRG]$/.test(first) && /^[WUBRG]$/.test(second)) {
      return `ms-${first.toLowerCase()}${second.toLowerCase()}p`;
    }
  }

  return null;
}

export function tokenizeManaText(text) {
  if (!text) {
    return [];
  }

  return text
    .split(TOKEN_PATTERN)
    .filter((segment) => segment.length > 0)
    .map((segment) => {
      if (!segment.startsWith("{") || !segment.endsWith("}")) {
        return { type: "text", value: segment };
      }

      return {
        type: "symbol",
        raw: segment,
        className: tokenToClass(segment),
      };
    });
}
