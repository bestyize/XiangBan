(() => {
'use strict';
  const icons = {
    sound: '<path d="M11 5 6 9H3v6h3l5 4zM15 8c3 2 3 6 0 8m3-11c5 4 5 10 0 14"/>',
    mute: '<path d="M11 5 6 9H3v6h3l5 4zM16 9l5 6m0-6-5 6"/>',
    undo: '<path d="M8 5 3 10l5 5M3 10h11a6 6 0 0 1 0 12" transform="translate(1 -2)"/>',
    hint: '<path d="M9 18h6m-5 3h4M8 14a6 6 0 1 1 8 0c-1 1-1 2-1 2H9s0-1-1-2Z"/>',
    reset: '<path d="M3 10a9 9 0 1 1 2 8M3 4v6h6"/>',
    cup: '<path d="M5 7h14l-2 14H7zM4 4h16M13 4l3-3"/>',
    addCup: '<path d="M4 5h10l-1 15H5zM18 8v8m-4-4h8"/>',
    book: '<path d="M12 5v16M12 5C9 2 4 3 2 4v15c4-2 7-1 10 2 3-3 6-4 10-2V4c-2-1-7-2-10 1Z"/>',
    arrow: '<path d="M4 12h16m-6-6 6 6-6 6"/>',
    download: '<path d="M12 3v12m-5-5 5 5 5-5M4 16v5h16v-5"/>',
    home: '<path d="m3 10 9-7 9 7v11H3zM9 21v-8h6v8"/>',
  };
  const icon = (name) => `<svg class="icon" viewBox="0 0 24 24" aria-hidden="true">${icons[name] || icons.cup}</svg>`;
  const drink = (color = '#ff6891', label = 'joy') => `<div class="drink" style="--drink-color:${color}" aria-hidden="true"><div class="straw"></div><div class="drink-body"><div class="pearls"></div></div><div class="lid"></div><div class="cup-label">${label}<small>POUR JOY</small></div></div>`;
  function shopSVG() { return `<svg viewBox="0 0 390 280" role="img" aria-label="街角饮品店，珊瑚红遮阳棚与薄荷绿橱窗"><defs><linearGradient id="wall" x2="0" y2="1"><stop stop-color="#f8e8cd"/><stop offset="1" stop-color="#ebd6b9"/></linearGradient><linearGradient id="window" x2="0" y2="1"><stop stop-color="#63c9c7"/><stop offset="1" stop-color="#baf1dc"/></linearGradient></defs><ellipse cx="196" cy="254" rx="148" ry="17" fill="#e1d1b4" opacity=".45"/><path d="M54 245h286" stroke="#c5b291"/><path d="M92 87h209v157H92z" fill="url(#wall)" stroke="#cfb392"/><path d="M102 83h188v-22q0-15-15-15H117q-15 0-15 15z" fill="#f1dcc0" stroke="#d4b794"/><rect x="129" y="55" width="134" height="29" rx="5" fill="#fff9e9"/><text x="196" y="75" text-anchor="middle" fill="#e95135" font-size="16" font-weight="700" font-family="Microsoft YaHei,sans-serif">倒杯快乐</text><path d="m87 85-14 27h244l-14-27z" fill="#f9edd8" stroke="#bd907d"/><path d="m87 85-14 27h25l11-27m25 0-7 27h25l3-27m25 0v27h25l-3-27m25 0 7 27h25l-11-27m25 0 14 27h-25l-11-27" fill="#f45b47"/><path d="M73 112h25v9q-12 13-25 0zm50 0h25v9q-12 13-25 0zm50 0h25v9q-12 13-25 0zm50 0h25v9q-12 13-25 0zm50 0h25v9q-12 13-25 0z" fill="#f45b47"/><path d="M98 112h25v9q-12 13-25 0zm50 0h25v9q-12 13-25 0zm50 0h25v9q-12 13-25 0zm50 0h25v9q-12 13-25 0zm50 0h19v9q-8 13-19 0z" fill="#fff2dc"/><rect x="109" y="141" width="104" height="72" rx="3" fill="url(#window)" stroke="#b69c7c" stroke-width="4"/><path d="M161 141v72m-52-34h104" stroke="#f7edd6" stroke-width="3"/><path d="m117 153 18-8m-18 16 30-13m24 43 30-14" stroke="#ffffff50" stroke-width="5"/><rect x="225" y="141" width="58" height="103" rx="2" fill="#baa58b"/><rect x="230" y="146" width="48" height="90" fill="#ede0c7"/><rect x="235" y="151" width="38" height="50" rx="18" fill="#a6b19a"/><circle cx="270" cy="215" r="3" fill="#8b6250"/><rect x="102" y="211" width="120" height="7" rx="2" fill="#b3926e"/><path d="M125 210v-18h13v18m15 0v-25h14v25m17 0v-17h13v17" fill="#e6b1a4" stroke="#f6dfc5" stroke-width="2"/><path d="m130 192 3-10m27 3 3-10m26 18 3-10" stroke="#f45b47" stroke-width="2"/><path d="m230 244-4 7h60l-3-7" fill="#c6ae8e"/><path d="m64 201-8 45m31-45 9 45" stroke="#a17e5f" stroke-width="3"/><rect x="58" y="195" width="36" height="45" rx="4" fill="#627968" stroke="#a98866" stroke-width="3"/><text x="76" y="212" text-anchor="middle" fill="#fff2d5" font-size="7">TODAY</text><path d="M68 222h16m-13 5h10" stroke="#eee4c8" stroke-width="2"/><path d="M313 224h26l-4 21h-18z" fill="#b9816a"/><path d="M324 225v-30m0 21q-24-1-13-20 14 2 13 20m2-5q-2-25 16-16 3 15-16 16m-1-12q-15-12-3-24 12 8 3 24" fill="#6dbd43"/><circle cx="310" cy="53" r="18" fill="#ffbd2f" opacity=".8"/><path d="M49 82h14m-7-7v14m275 76h10m-5-5v10" stroke="#c8a06e" stroke-width="1.5"/><text x="195" y="270" text-anchor="middle" fill="#a78f72" font-size="7" letter-spacing="3"></text></svg>`; }

globalThis.PourJoyArt = Object.freeze({ icon, drink, shopSVG });
})();
