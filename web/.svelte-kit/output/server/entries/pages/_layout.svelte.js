import { s as slot } from "../../chunks/index.js";
function _layout($$renderer, $$props) {
  $$renderer.push(`<main style="font-family: system-ui; background: #111; color: #eee; min-height: 100vh;"><!--[-->`);
  slot($$renderer, $$props, "default", {});
  $$renderer.push(`<!--]--></main>`);
}
export {
  _layout as default
};
