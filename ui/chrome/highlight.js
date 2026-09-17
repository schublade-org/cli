function highlight(code) {
  return escapeHtml(code).replace(
    /(&lt;\/?[A-Za-z][\w.-]*)|(\s[\w:-]+=)|(&quot;.*?&quot;)/g,
    (match, tag, attr, str) => {
      if (tag) return `<span class="tok-tag">${tag}</span>`;
      if (attr) return `<span class="tok-attr">${attr}</span>`;
      return `<span class="tok-str">${str}</span>`;
    }
  );
}
function escapeHtml(value) {
  return String(value).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}
export {
  escapeHtml,
  highlight
};
