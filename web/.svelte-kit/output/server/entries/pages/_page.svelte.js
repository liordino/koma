import { a as ensure_array_like, b as attr, e as escape_html } from "../../chunks/index.js";
function _page($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let items = [];
    $$renderer2.push(`<h1>Koma</h1> `);
    {
      $$renderer2.push("<!--[-1-->");
    }
    $$renderer2.push(`<!--]--> <ul><!--[-->`);
    const each_array = ensure_array_like(items);
    for (let $$index = 0, $$length = each_array.length; $$index < $$length; $$index++) {
      let m = each_array[$$index];
      $$renderer2.push(`<li><a${attr("href", `/reader/1`)}>${escape_html(m.title)} <small>(${escape_html(m.sourceKey)})</small></a></li>`);
    }
    $$renderer2.push(`<!--]--></ul>`);
  });
}
export {
  _page as default
};
