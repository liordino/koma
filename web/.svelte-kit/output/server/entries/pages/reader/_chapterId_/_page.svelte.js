import { e as escape_html, a as ensure_array_like, b as attr, d as derived } from "../../../../chunks/index.js";
import "../../../../chunks/state.svelte.js";
import "@sveltejs/kit/internal";
import "../../../../chunks/exports.js";
import "../../../../chunks/utils2.js";
import "@sveltejs/kit/internal/server";
import "../../../../chunks/root.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let blobs = [];
    let rendered = derived(
      () => []
    );
    $$renderer2.push(`<p style="position:sticky;top:0;background:#111;margin:0;padding:4px 8px;">${escape_html("loading…")}</p> <!--[-->`);
    const each_array = ensure_array_like([]);
    for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
      let i = each_array[$$index];
      $$renderer2.push(`<div style="height:1200px;display:flex;align-items:center;justify-content:center;background:#0a0a0a;">`);
      if (rendered().includes(i)) {
        $$renderer2.push(`<!--[0--><img${attr("src", blobs[i])}${attr("alt", `page ${i + 1}`)} style="max-width:100%;max-height:100%;"/>`);
      } else {
        $$renderer2.push(`<!--[-1--><span style="color:#333">·</span>`);
      }
      $$renderer2.push(`<!--]--></div>`);
    }
    $$renderer2.push(`<!--]-->`);
  });
}
export {
  _page as default
};
