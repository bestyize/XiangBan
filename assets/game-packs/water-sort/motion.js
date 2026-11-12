(() => {
  'use strict';

  const clamp = value => Math.max(0, Math.min(1, value));
  const ease = value => value * value * (3 - 2 * value);

  function pour(source, target, { amount, color, onFlow }) {
    const a = source.getBoundingClientRect();
    const b = target.getBoundingClientRect();
    const width = source.offsetWidth;
    const height = source.offsetHeight;
    const ghost = source.cloneNode(true);
    ghost.classList.add('pour-ghost');
    ghost.setAttribute('aria-hidden', 'true');
    Object.assign(ghost.style, {
      left: `${a.left + (a.width - width) / 2}px`, top: `${a.top}px`,
      width: `${width}px`, height: `${height}px`, transformOrigin: '50% 3px',
    });

    const sourceStack = ghost.querySelector('.liquid-stack');
    const targetStack = target.querySelector('.liquid-stack');
    const targetArea = targetStack.getBoundingClientRect();
    const before = targetStack.children.length;
    const units = amount * 25;
    const layer = () => {
      const node = document.createElement('span');
      node.className = 'liquid flowing-layer';
      node.setAttribute('aria-hidden', 'true');
      node.style.setProperty('--flavor', color);
      return node;
    };
    const leaving = layer();
    const arriving = layer();
    [...sourceStack.children].slice(-amount).forEach(node => node.remove());
    sourceStack.append(leaving);
    leaving.style.height = `${units}%`;
    arriving.style.height = '0%';
    source.style.visibility = 'hidden';

    const stream = document.createElement('span');
    stream.className = 'pour-stream';
    stream.setAttribute('aria-hidden', 'true');
    stream.style.setProperty('--flavor', color);
    const x = b.left + b.width / 2;
    const side = x >= a.left + a.width / 2 ? 1 : -1;
    const angle = 58 * side;
    const lip = width / 2 - 2;
    const lipY = b.top - 16;
    const dx = x - (a.left + a.width / 2) - side * lip * Math.cos(Math.PI * 58 / 180);
    const dy = lipY - a.top - 3 - lip * Math.sin(Math.PI * 58 / 180);
    const restY = source.offsetParent.getBoundingClientRect().top + source.offsetTop - a.top;
    Object.assign(stream.style, { left: `${x - 2}px`, top: `${lipY}px`, opacity: '0' });
    document.body.append(stream, ghost);

    const travel = 125;
    const flow = 180 + amount * 25;
    const returnTime = 125;
    let frame, resolve, done = false, flowing = false;
    const finished = new Promise(end => { resolve = end; });
    const started = performance.now();

    function finish() {
      if (done) return;
      done = true;
      cancelAnimationFrame(frame);
      ghost.remove();
      stream.remove();
      arriving.remove();
      source.style.visibility = '';
      resolve();
    }

    function draw(now) {
      const elapsed = now - started;
      if (elapsed >= travel + flow + returnTime) { finish(); return; }
      const fill = clamp((elapsed - travel) / flow);
      const back = ease(clamp((elapsed - travel - flow) / returnTime));
      const lean = ease(clamp(elapsed / travel)) * (1 - back);
      ghost.style.transform = `translate(${dx * lean}px, ${dy * lean + restY * back}px) rotate(${angle * lean}deg)`;
      if (elapsed >= travel && !flowing) {
        flowing = true;
        targetStack.append(arriving);
        onFlow(flow / 1000);
      }
      leaving.style.height = `${units * (1 - fill)}%`;
      leaving.hidden = fill === 1;
      arriving.style.height = `${units * fill}%`;
      stream.style.height = `${Math.max(0, targetArea.bottom - targetArea.height * (before + amount * fill) / 4 - lipY)}px`;
      stream.style.opacity = `${Math.min(fill * 12, (1 - fill) * 12, .94)}`;
      frame = requestAnimationFrame(draw);
    }

    frame = requestAnimationFrame(draw);
    return { finished, finish };
  }

  globalThis.PourJoyMotion = Object.freeze({ pour });
})();
