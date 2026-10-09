export function looksLikeUrl(text: string): boolean {
  const t = text.trim();
  if (/^https?:\/\//i.test(t)) {
    try {
      const u = new URL(t);
      return u.protocol === "http:" || u.protocol === "https:";
    } catch {
      return false;
    }
  }
  return /^www\.[^\s/]+\.\S+$/i.test(t);
}
