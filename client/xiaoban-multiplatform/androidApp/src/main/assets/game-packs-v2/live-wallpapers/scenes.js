(() => {
  function random(seed) {
    return () => {
      seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0;
      return seed / 4294967296;
    };
  }

  function waterBackground() {
    const layer = document.createElement('canvas');
    layer.width = 320;
    layer.height = 560;
    const ctx = layer.getContext('2d');
    const water = ctx.createLinearGradient(0, 0, 55, 560);
    water.addColorStop(0, '#266d6a');
    water.addColorStop(.53, '#10393b');
    water.addColorStop(1, '#061b25');
    ctx.fillStyle = water;
    ctx.fillRect(0, 0, 320, 560);
    const light = ctx.createRadialGradient(133, 5, 12, 132, 4, 360);
    light.addColorStop(0, '#baf2c299');
    light.addColorStop(1, '#4b9f7b00');
    ctx.fillStyle = light;
    ctx.fillRect(0, 0, 320, 560);

    const rng = random(35);
    for (let i = 0; i < 12; i++) {
      const x = rng() * 320;
      ctx.fillStyle = 'rgba(195,255,202,' + (.025 + rng() * .045) + ')';
      ctx.beginPath();
      ctx.moveTo(x, 0);
      ctx.lineTo(x + rng() * 45, 0);
      ctx.lineTo(x + 56, 350);
      ctx.fill();
    }
    for (let i = 0; i < 95; i++) {
      const x = rng() * 320;
      const y = 556 - rng() * 112 + Math.sin(x * .028) * 22;
      ctx.fillStyle = ['#173735', '#234443', '#284d46', '#31514a'][i % 4];
      ctx.beginPath();
      ctx.ellipse(x, y, 5 + rng() * 35, 3 + rng() * 19, rng() - .5, 0, Math.PI * 2);
      ctx.fill();
    }
    for (let i = 0; i < 54; i++) {
      const x = i < 27 ? rng() * 75 : 238 + rng() * 85;
      const y = 518 + rng() * 44;
      const dx = (rng() - .5) * 74;
      const h = 35 + rng() * 142;
      ctx.fillStyle = ['#153a33', '#235046', '#386456', '#47775d'][i % 4];
      ctx.beginPath();
      ctx.moveTo(x, y);
      ctx.quadraticCurveTo(x + dx * .7 - 8, y - h * .57, x + dx, y - h);
      ctx.quadraticCurveTo(x + dx * .75 + 11, y - h * .47, x + 3.5, y);
      ctx.fill();
    }
    return layer;
  }

  function aquarium(canvas, fps) {
    const ctx = canvas.getContext('2d', { alpha: false });
    const background = waterBackground();
    const rng = random(17);
    const fish = Array.from({ length: fps === 15 ? 8 : 13 }, (_, i) => ({
      x: rng(), y: .22 + rng() * .54, speed: .023 + rng() * .03, phase: rng() * 7,
      scale: .45 + rng() * .68, direction: i % 3 === 0 ? -1 : 1,
      color: ['#ffbd86', '#ff875f', '#f5e5ba', '#afd5c6'][i % 4],
    }));
    const bubbles = Array.from({ length: 18 }, () => ({ x: rng(), y: rng(), r: 1 + rng() * 2 }));
    const ripples = [];
    let time = 0;

    return {
      draw(t) {
        const w = canvas.width, h = canvas.height;
        const unit = Math.min(w, h) / 330;
        time = t;
        ctx.drawImage(background, 0, 0, w, h);
        for (const bubble of bubbles) {
          const y = 1 - (bubble.y + t * .041) % 1;
          ctx.strokeStyle = 'rgba(201,249,233,' + (.12 + y * .22) + ')';
          ctx.lineWidth = unit;
          ctx.beginPath();
          ctx.arc((bubble.x + Math.sin(t + bubble.y * 12) * .016) * w, y * h, bubble.r * unit, 0, Math.PI * 2);
          ctx.stroke();
        }
        for (const fishBody of fish) {
          const path = (fishBody.x + t * fishBody.speed) % 1 * 1.24 - .12;
          const x = fishBody.direction === 1 ? path : 1 - path;
          const y = fishBody.y + Math.sin(t * .9 + fishBody.phase) * .038;
          const size = fishBody.scale * unit;
          ctx.save();
          ctx.translate(x * w, y * h);
          ctx.rotate(Math.cos(t * .9 + fishBody.phase) * .10 * fishBody.direction);
          ctx.scale(fishBody.direction * size, size);
          ctx.fillStyle = fishBody.color;
          ctx.globalAlpha = .79;
          const tail = Math.sin(t * 12 + fishBody.phase) * 3.5;
          ctx.beginPath();
          ctx.moveTo(-10, 0);
          ctx.quadraticCurveTo(-18, -8 + tail, -24, -5 + tail);
          ctx.quadraticCurveTo(-19, tail, -24, 5 + tail);
          ctx.quadraticCurveTo(-16, 7 + tail, -10, 0);
          ctx.fill();
          ctx.globalAlpha = 1;
          ctx.beginPath();
          ctx.moveTo(-15, 0);
          ctx.bezierCurveTo(-7, -7, 8, -9, 16, -1);
          ctx.bezierCurveTo(14, 6, -6, 7, -15, 0);
          ctx.fill();
          ctx.fillStyle = '#fff4cfb0';
          ctx.beginPath();
          ctx.ellipse(0, 2, 6, 1.7, .38, 0, Math.PI * 2);
          ctx.fill();
          ctx.fillStyle = '#163132';
          ctx.beginPath();
          ctx.arc(11, -1.6, 1.35, 0, Math.PI * 2);
          ctx.fill();
          ctx.restore();
        }
        while (ripples.length && time - ripples[0].start > 1.6) ripples.shift();
        ctx.lineWidth = 1.5 * unit;
        for (const ripple of ripples) {
          const age = (time - ripple.start) / 1.6;
          ctx.strokeStyle = 'rgba(220,255,233,' + ((1 - age) * .68) + ')';
          for (let i = 0; i < 2; i++) {
            const radius = (12 + age * 112 + i * 12) * unit;
            ctx.beginPath();
            ctx.ellipse(ripple.x * w, ripple.y * h, radius, radius * .54, 0, 0, Math.PI * 2);
            ctx.stroke();
          }
        }
      },
      tap(x, y) {
        if (ripples.length === 6) ripples.shift();
        ripples.push({ x, y, start: time });
      },
    };
  }

  function starData(count) {
    const rng = random(245);
    const stars = new Float32Array(count * 4);
    for (let i = 0; i < count; i++) {
      const core = i < count * .25;
      const radius = core ? .29 * Math.pow(rng(), 1.75) : .09 + .89 * Math.pow(rng(), 1.3);
      const spread = (rng() + rng() - 1) * .74;
      const angle = core ? rng() * Math.PI * 2 : i % 4 * Math.PI / 2 + Math.log(radius + .14) * 2.25 + spread;
      stars[i * 4] = Math.cos(angle) * radius;
      stars[i * 4 + 1] = Math.sin(angle) * radius;
      stars[i * 4 + 2] = (rng() + rng() - 1) * (core ? .22 : .075);
      stars[i * 4 + 3] = rng();
    }
    return stars;
  }

  function galaxyFallback(canvas) {
    const ctx = canvas.getContext('2d', { alpha: false });
    const stars = starData(630);
    let offset = 0;

    return {
      draw(t) {
        const w = canvas.width, h = canvas.height;
        const angle = t * .09 + offset;
        const c = Math.cos(angle), s = Math.sin(angle);
        ctx.fillStyle = '#0b1020';
        ctx.fillRect(0, 0, w, h);
        const scale = Math.min(w, h) * .52;
        for (let i = 0; i < stars.length; i += 4) {
          const x = stars[i], y = stars[i + 1], z = stars[i + 2];
          ctx.fillStyle = Math.hypot(x, y) < .3 ? '#ffce9f' : '#a1bce7';
          ctx.globalAlpha = .25 + stars[i + 3] * .62;
          ctx.fillRect(w * .5 + scale * (x * c - y * s), h * .49 + scale * ((x * s + y * c) * .61 + z), 1.6, 1.6);
        }
        ctx.globalAlpha = 1;
      },
      tap(x) { offset += (x - .5) * .5; },
    };
  }

  function galaxy(canvas, fps) {
    const gl = canvas.getContext('webgl', { alpha: false, antialias: false, depth: false, stencil: false });
    if (!gl) return galaxyFallback(canvas);
    const count = fps === 15 ? 3400 : 7600;
    const stars = starData(count);
    let offset = 0, target = 0;

    function compile(type, source) {
      const shader = gl.createShader(type);
      gl.shaderSource(shader, source);
      gl.compileShader(shader);
      if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
        const message = gl.getShaderInfoLog(shader);
        gl.deleteShader(shader);
        throw new Error(message || '壁纸着色器编译失败');
      }
      return shader;
    }

    function setup() {
      const vs = compile(gl.VERTEX_SHADER, [
        'attribute vec4 star;',
        'uniform vec2 size;',
        'uniform float angle;',
        'varying vec4 color;',
        'void main() {',
        '  float c = cos(angle), s = sin(angle);',
        '  vec2 p = mat2(c, s, -s, c) * star.xy;',
        '  float f = 1.0 / (1.0 + p.y * 0.28);',
        '  vec2 axis = vec2(p.x, p.y * .62 + star.z * .95);',
        '  gl_Position = vec4(axis * min(size.x, size.y) / size * f * .97, 0.0, 1.0);',
        '  gl_PointSize = min(4.5, max(1.2, size.x / 460.0) * (1.0 + star.w * 1.5) * f);',
        '  float rim = smoothstep(.12, .89, length(star.xy));',
        '  vec3 light = mix(vec3(1.0, .55, .27), vec3(.39, .65, .99), rim);',
        '  color = vec4(mix(light, vec3(1.0, .94, .85), pow(star.w, 4.0)), .17 + star.w * .50);',
        '}',
      ].join('\n'));
      const fs = compile(gl.FRAGMENT_SHADER, [
        'precision mediump float;',
        'varying vec4 color;',
        'void main() {',
        '  float fade = 1.0 - smoothstep(.0, .5, length(gl_PointCoord - .5));',
        '  gl_FragColor = vec4(color.rgb, color.a * fade);',
        '}',
      ].join('\n'));
      const program = gl.createProgram();
      gl.attachShader(program, vs);
      gl.attachShader(program, fs);
      gl.linkProgram(program);
      gl.deleteShader(vs);
      gl.deleteShader(fs);
      if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
        const message = gl.getProgramInfoLog(program);
        gl.deleteProgram(program);
        throw new Error(message || '壁纸着色器链接失败');
      }
      gl.useProgram(program);
      const buffer = gl.createBuffer();
      gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
      gl.bufferData(gl.ARRAY_BUFFER, stars, gl.STATIC_DRAW);
      const star = gl.getAttribLocation(program, 'star');
      gl.enableVertexAttribArray(star);
      gl.vertexAttribPointer(star, 4, gl.FLOAT, false, 0, 0);
      gl.clearColor(.031, .045, .08, 1);
      gl.enable(gl.BLEND);
      gl.blendFunc(gl.SRC_ALPHA, gl.ONE);
      return { program, buffer, size: gl.getUniformLocation(program, 'size'), angle: gl.getUniformLocation(program, 'angle') };
    }

    let state = setup();
    const onLost = event => { event.preventDefault(); state = null; };
    const onRestored = () => { state = setup(); };
    canvas.addEventListener('webglcontextlost', onLost);
    canvas.addEventListener('webglcontextrestored', onRestored);

    return {
      draw(t) {
        if (!state) return;
        const w = canvas.width, h = canvas.height;
        offset += (target - offset) * .10;
        gl.viewport(0, 0, w, h);
        gl.uniform2f(state.size, w, h);
        gl.uniform1f(state.angle, t * .11 + offset);
        gl.clear(gl.COLOR_BUFFER_BIT);
        gl.drawArrays(gl.POINTS, 0, count);
      },
      tap(x) { target += (x - .5) * 1.1; },
      destroy() {
        canvas.removeEventListener('webglcontextlost', onLost);
        canvas.removeEventListener('webglcontextrestored', onRestored);
        if (state) {
          gl.deleteProgram(state.program);
          gl.deleteBuffer(state.buffer);
          gl.getExtension('WEBGL_lose_context')?.loseContext();
        }
      },
    };
  }

  window.createWallpaperScene = (canvas, kind, fps) => {
    const view = kind === 'galaxy' ? galaxy(canvas, fps) : aquarium(canvas, fps);
    let active = false, resize = true, frame = 0, elapsed = 0, previous = 0, next = 0;
    const interval = 1000 / fps;
    const observer = new ResizeObserver(() => { resize = true; });
    observer.observe(canvas);

    function draw(now) {
      if (!active) return;
      if (now + .5 >= next) {
        if (resize) {
          const ratio = Math.min(devicePixelRatio || 1, 2);
          canvas.width = Math.max(1, Math.round(canvas.clientWidth * ratio));
          canvas.height = Math.max(1, Math.round(canvas.clientHeight * ratio));
          resize = false;
        }
        if (previous) elapsed += Math.min(.1, (now - previous) / 1000);
        previous = now;
        next = next && now - next < interval ? next + interval : now + interval;
        view.draw(elapsed);
      }
      frame = requestAnimationFrame(draw);
    }

    function setActive(visible) {
      if (active === visible) return;
      active = visible;
      if (visible) {
        previous = next = 0;
        frame = requestAnimationFrame(draw);
      } else {
        cancelAnimationFrame(frame);
      }
    }

    return {
      setActive,
      tap(x, y) { view.tap(x, y); },
      destroy() {
        setActive(false);
        observer.disconnect();
        view.destroy?.();
      },
    };
  };
})();
