export function renderFallback(
  directiveName: string,
  kind: "inline" | "block" | "container" = "block"
): string {
  const wrapper = kind === "inline" ? "span" : "div";
  return `<${wrapper} class="etch-missing-plugin"><span>Unknown directive: <code>${escapeHtml(
    directiveName
  )}</code></span><span>No plugin installed for this directive.</span></${wrapper}>`;
}

function escapeHtml(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll("\"", "&quot;");
}
