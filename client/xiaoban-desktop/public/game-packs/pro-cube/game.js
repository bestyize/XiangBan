(function () {
  'use strict';

  // ==========================================
  // 0. 小伴桌面宿主协议适配 (xiaban.game-pack protocol 1)
  // ==========================================
  const nonce = new URLSearchParams(location.search).get('hostNonce');
  let hostReady = false;

  function postToHost(type, payload = {}) {
    if (!nonce || window.parent === window) return;
    window.parent.postMessage({ channel: 'xiaban.game-pack', protocol: 1, nonce, type, ...payload }, '*');
  }

  window.addEventListener('message', (event) => {
    const data = event.data;
    if (!nonce || event.source !== window.parent || !data || data.channel !== 'xiaban.game-host' || data.protocol !== 1 || data.nonce !== nonce) return;
    if (data.type === 'host.init') {
      hostReady = true;
      postToHost('game.ready');
    }
  });

  // ==========================================
  // 1. Web Audio 机械磁力魔方音效合成器
  // ==========================================
  class CubeSoundEngine {
    constructor() {
      this.ctx = null;
      this.enabled = true;
    }
    init() {
      if (!this.ctx) {
        const AudioCtx = window.AudioContext || window.webkitAudioContext;
        if (AudioCtx) {
          this.ctx = new AudioCtx();
        }
      }
      if (this.ctx && this.ctx.state === 'suspended') {
        this.ctx.resume();
      }
    }
    playSnap(isFast = false) {
      if (!this.enabled) return;
      try {
        this.init();
        if (!this.ctx) return;
        const now = this.ctx.currentTime;

        const bufferSize = Math.floor(this.ctx.sampleRate * 0.042);
        const noiseBuffer = this.ctx.createBuffer(1, bufferSize, this.ctx.sampleRate);
        const output = noiseBuffer.getChannelData(0);
        for (let i = 0; i < bufferSize; i++) {
          output[i] = (Math.random() * 2 - 1) * Math.exp(-i / (bufferSize * 0.35));
        }
        const whiteNoise = this.ctx.createBufferSource();
        whiteNoise.buffer = noiseBuffer;

        const filter = this.ctx.createBiquadFilter();
        filter.type = 'bandpass';
        filter.frequency.setValueAtTime(isFast ? 1550 : 1200, now);
        filter.Q.setValueAtTime(2.4, now);

        const noiseGain = this.ctx.createGain();
        noiseGain.gain.setValueAtTime(0.2, now);
        noiseGain.gain.exponentialRampToValueAtTime(0.001, now + 0.04);

        whiteNoise.connect(filter);
        filter.connect(noiseGain);
        noiseGain.connect(this.ctx.destination);
        whiteNoise.start(now);

        const osc = this.ctx.createOscillator();
        const clickGain = this.ctx.createGain();
        osc.type = 'triangle';
        osc.frequency.setValueAtTime(isFast ? 340 : 270, now + 0.012);
        osc.frequency.exponentialRampToValueAtTime(75, now + 0.058);

        clickGain.gain.setValueAtTime(0.26, now + 0.012);
        clickGain.gain.exponentialRampToValueAtTime(0.001, now + 0.058);

        osc.connect(clickGain);
        clickGain.connect(this.ctx.destination);
        osc.start(now + 0.012);
        osc.stop(now + 0.06);
      } catch (e) {}
    }
    playVictory() {
      if (!this.enabled) return;
      try {
        this.init();
        if (!this.ctx) return;
        const notes = [523.25, 659.25, 783.99, 1046.5];
        notes.forEach((freq, idx) => {
          const now = this.ctx.currentTime + idx * 0.085;
          const osc = this.ctx.createOscillator();
          const gain = this.ctx.createGain();
          osc.type = 'sine';
          osc.frequency.setValueAtTime(freq, now);
          gain.gain.setValueAtTime(0.18, now);
          gain.gain.exponentialRampToValueAtTime(0.001, now + 0.34);
          osc.connect(gain);
          gain.connect(this.ctx.destination);
          osc.start(now);
          osc.stop(now + 0.35);
        });
      } catch (e) {}
    }
  }

  const sound = new CubeSoundEngine();

  // ==========================================
  // 2. 真实魔方材质与程序化高光/凹凸纹理生成
  // ==========================================
  const FACE_COLORS = {
    R: '#e11d48', // 右 (+X) 竞速红
    L: '#f97316', // 左 (-X) 荧光橙
    U: '#f8fafc', // 顶 (+Y) 象牙白
    D: '#facc15', // 底 (-Y) 柠檬黄
    F: '#10b981', // 前 (+Z) 翡翠绿
    B: '#2563eb'  // 后 (-Z) 宝石蓝
  };

  const FINISH_MODES = [
    { id: 'glossy', name: '亮面磁力贴片' },
    { id: 'frosted', name: '竞速磨砂无贴纸' },
    { id: 'carbon', name: '碳纤维纹理贴片' }
  ];
  let currentFinishIndex = 0;

  function createStudioEnvMap() {
    const size = 256;
    const canvases = [];
    for (let i = 0; i < 6; i++) {
      const cvs = document.createElement('canvas');
      cvs.width = size;
      cvs.height = size;
      const ctx = cvs.getContext('2d');

      const grad = ctx.createLinearGradient(0, 0, 0, size);
      if (i === 2) {
        grad.addColorStop(0, '#ffffff');
        grad.addColorStop(1, '#cbd5e1');
      } else if (i === 3) {
        grad.addColorStop(0, '#1e293b');
        grad.addColorStop(1, '#090d16');
      } else {
        grad.addColorStop(0, '#475569');
        grad.addColorStop(0.45, '#1e293b');
        grad.addColorStop(0.55, '#0f172a');
        grad.addColorStop(1, '#1e293b');
      }
      ctx.fillStyle = grad;
      ctx.fillRect(0, 0, size, size);

      ctx.fillStyle = 'rgba(255, 255, 255, 0.85)';
      if (i === 0 || i === 4) {
        ctx.beginPath();
        if (ctx.roundRect) {
          ctx.roundRect(36, 28, 180, 75, 20);
        } else {
          ctx.rect(36, 28, 180, 75);
        }
        ctx.fill();
      } else if (i === 2) {
        ctx.beginPath();
        ctx.arc(size / 2, size / 2, 90, 0, Math.PI * 2);
        ctx.fill();
      }
      canvases.push(cvs);
    }
    const cubeTexture = new THREE.CubeTexture(canvases);
    cubeTexture.needsUpdate = true;
    return cubeTexture;
  }

  function createBumpTextures() {
    const frostCanvas = document.createElement('canvas');
    frostCanvas.width = 128;
    frostCanvas.height = 128;
    const fCtx = frostCanvas.getContext('2d');
    const imgData = fCtx.createImageData(128, 128);
    for (let i = 0; i < imgData.data.length; i += 4) {
      const v = 115 + Math.floor(Math.random() * 35);
      imgData.data[i] = v;
      imgData.data[i + 1] = v;
      imgData.data[i + 2] = v;
      imgData.data[i + 3] = 255;
    }
    fCtx.putImageData(imgData, 0, 0);
    const frostTex = new THREE.CanvasTexture(frostCanvas);
    frostTex.wrapS = THREE.RepeatWrapping;
    frostTex.wrapT = THREE.RepeatWrapping;
    frostTex.repeat.set(3, 3);

    const carbonCanvas = document.createElement('canvas');
    carbonCanvas.width = 64;
    carbonCanvas.height = 64;
    const cCtx = carbonCanvas.getContext('2d');
    cCtx.fillStyle = '#505050';
    cCtx.fillRect(0, 0, 64, 64);
    for (let y = 0; y < 64; y += 8) {
      for (let x = 0; x < 64; x += 8) {
        const offset = (y / 8) % 2 === 0 ? 0 : 4;
        cCtx.fillStyle = '#dcdcdc';
        cCtx.fillRect((x + offset) % 64, y, 4, 7);
        cCtx.fillStyle = '#909090';
        cCtx.fillRect((x + offset + 4) % 64, y, 4, 7);
      }
    }
    const carbonTex = new THREE.CanvasTexture(carbonCanvas);
    carbonTex.wrapS = THREE.RepeatWrapping;
    carbonTex.wrapT = THREE.RepeatWrapping;
    carbonTex.repeat.set(4, 4);

    return { frostTex, carbonTex };
  }

  function createRoundedBoxGeometry(width, height, depth, radius, smoothness) {
    const shape = new THREE.Shape();
    const eps = 0.00001;
    const r = radius - eps;
    shape.absarc(eps, eps, eps, -Math.PI / 2, -Math.PI, true);
    shape.absarc(eps, height - r * 2, eps, Math.PI, Math.PI / 2, true);
    shape.absarc(width - r * 2, height - r * 2, eps, Math.PI / 2, 0, true);
    shape.absarc(width - r * 2, eps, eps, 0, -Math.PI / 2, true);

    const geometry = new THREE.ExtrudeGeometry(shape, {
      depth: depth - radius * 2,
      bevelEnabled: true,
      bevelSegments: smoothness * 2,
      steps: 1,
      bevelSize: r,
      bevelThickness: radius,
      curveSegments: smoothness
    });
    geometry.center();
    return geometry;
  }

  function createStickerGeometry(size, radius, thickness) {
    const shape = new THREE.Shape();
    const half = size / 2;
    shape.moveTo(-half + radius, -half);
    shape.lineTo(half - radius, -half);
    shape.quadraticCurveTo(half, -half, half, -half + radius);
    shape.lineTo(half, half - radius);
    shape.quadraticCurveTo(half, half, half - radius, half);
    shape.lineTo(-half + radius, half);
    shape.quadraticCurveTo(-half, half, -half, half - radius);
    shape.lineTo(-half, -half + radius);
    shape.quadraticCurveTo(-half, -half, -half + radius, -half);

    const geom = new THREE.ExtrudeGeometry(shape, {
      depth: thickness,
      bevelEnabled: true,
      bevelSegments: 2,
      steps: 1,
      bevelSize: 0.016,
      bevelThickness: thickness * 0.8,
      curveSegments: 6
    });
    geom.center();
    return geom;
  }

  // ==========================================
  // 3. Three.js 场景与灯光系统
  // ==========================================
  const container = document.getElementById('canvas-container');
  const scene = new THREE.Scene();

  const camera = new THREE.PerspectiveCamera(
    38,
    window.innerWidth / window.innerHeight,
    0.1,
    100
  );

  const renderer = new THREE.WebGLRenderer({
    antialias: true,
    alpha: true,
    powerPreference: 'high-performance'
  });
  renderer.setSize(window.innerWidth, window.innerHeight);
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
  renderer.shadowMap.enabled = true;
  renderer.shadowMap.type = THREE.PCFSoftShadowMap;

  if (THREE.SRGBColorSpace) {
    renderer.outputColorSpace = THREE.SRGBColorSpace;
  } else if (THREE.sRGBEncoding) {
    renderer.outputEncoding = THREE.sRGBEncoding;
  }

  renderer.toneMapping = THREE.ACESFilmicToneMapping;
  renderer.toneMappingExposure = 1.18;
  container.appendChild(renderer.domElement);

  const envMap = createStudioEnvMap();
  scene.environment = envMap;

  const { frostTex, carbonTex } = createBumpTextures();

  const ambientLight = new THREE.AmbientLight(0xffffff, 0.58);
  scene.add(ambientLight);

  const mainLight = new THREE.DirectionalLight(0xfffbeb, 1.45);
  mainLight.position.set(7, 11, 9);
  mainLight.castShadow = true;
  mainLight.shadow.mapSize.width = 1024;
  mainLight.shadow.mapSize.height = 1024;
  mainLight.shadow.camera.near = 2;
  mainLight.shadow.camera.far = 28;
  const d = 4.8;
  mainLight.shadow.camera.left = -d;
  mainLight.shadow.camera.right = d;
  mainLight.shadow.camera.top = d;
  mainLight.shadow.camera.bottom = -d;
  mainLight.shadow.bias = -0.001;
  scene.add(mainLight);

  const fillLight = new THREE.DirectionalLight(0x93c5fd, 0.75);
  fillLight.position.set(-9, -6, 6);
  scene.add(fillLight);

  const rimLight = new THREE.DirectionalLight(0xfde68a, 0.65);
  rimLight.position.set(0, 7, -10);
  scene.add(rimLight);

  const shadowPlaneGeo = new THREE.PlaneGeometry(14, 14);
  const shadowPlaneMat = new THREE.ShadowMaterial({ opacity: 0.28 });
  const shadowPlane = new THREE.Mesh(shadowPlaneGeo, shadowPlaneMat);
  shadowPlane.rotation.x = -Math.PI / 2;
  shadowPlane.position.y = -2.95;
  shadowPlane.receiveShadow = true;
  scene.add(shadowPlane);

  // ==========================================
  // 4. 支持 3~9 阶的高性能魔方构建引擎
  // ==========================================
  let cubeOrder = 3;
  let selectedLayerDepth = 1;

  const cubeRoot = new THREE.Group();
  scene.add(cubeRoot);

  const pivotGroup = new THREE.Group();
  cubeRoot.add(pivotGroup);

  const CUBIE_SIZE = 0.962;
  const cubieGeo = createRoundedBoxGeometry(CUBIE_SIZE, CUBIE_SIZE, CUBIE_SIZE, 0.088, 3);
  const stickerGeo = createStickerGeometry(0.80, 0.10, 0.022);

  const innerShieldGeo = new THREE.BoxGeometry(1, 1, 1);
  const innerShieldMat = new THREE.MeshBasicMaterial({ color: 0x090b10 });
  let innerShieldMesh = null;

  const coreMaterial = new THREE.MeshPhysicalMaterial({
    color: 0x111318,
    roughness: 0.28,
    metalness: 0.06,
    clearcoat: 0.6,
    clearcoatRoughness: 0.2,
    envMap: envMap,
    envMapIntensity: 0.9
  });

  const faceMaterials = {};
  Object.keys(FACE_COLORS).forEach((key) => {
    faceMaterials[key] = new THREE.MeshPhysicalMaterial({
      color: new THREE.Color(FACE_COLORS[key]),
      roughness: 0.12,
      metalness: 0.02,
      clearcoat: 1.0,
      clearcoatRoughness: 0.06,
      reflectivity: 0.95,
      envMap: envMap,
      envMapIntensity: 1.25
    });
  });

  function applyFinishMode(modeId) {
    Object.keys(faceMaterials).forEach((key) => {
      const mat = faceMaterials[key];
      if (modeId === 'glossy') {
        mat.roughness = 0.1;
        mat.clearcoat = 1.0;
        mat.clearcoatRoughness = 0.05;
        mat.bumpMap = null;
        mat.bumpScale = 0;
        coreMaterial.color.setHex(0x111318);
        coreMaterial.roughness = 0.26;
      } else if (modeId === 'frosted') {
        mat.roughness = 0.36;
        mat.clearcoat = 0.35;
        mat.clearcoatRoughness = 0.35;
        mat.bumpMap = frostTex;
        mat.bumpScale = 0.008;
        coreMaterial.color.setHex(0x1e293b);
        coreMaterial.roughness = 0.42;
      } else if (modeId === 'carbon') {
        mat.roughness = 0.22;
        mat.clearcoat = 0.9;
        mat.clearcoatRoughness = 0.12;
        mat.bumpMap = carbonTex;
        mat.bumpScale = 0.018;
        coreMaterial.color.setHex(0x090b0e);
        coreMaterial.roughness = 0.2;
      }
      mat.needsUpdate = true;
    });
    coreMaterial.needsUpdate = true;
  }

  const cubies = [];
  const pickableMeshes = [];

  const FACE_DEFS = [
    { key: 'R', axis: 'x', isMax: true, dir: new THREE.Vector3(1, 0, 0), rot: [0, Math.PI / 2, 0] },
    { key: 'L', axis: 'x', isMax: false, dir: new THREE.Vector3(-1, 0, 0), rot: [0, -Math.PI / 2, 0] },
    { key: 'U', axis: 'y', isMax: true, dir: new THREE.Vector3(0, 1, 0), rot: [-Math.PI / 2, 0, 0] },
    { key: 'D', axis: 'y', isMax: false, dir: new THREE.Vector3(0, -1, 0), rot: [Math.PI / 2, 0, 0] },
    { key: 'F', axis: 'z', isMax: true, dir: new THREE.Vector3(0, 0, 1), rot: [0, 0, 0] },
    { key: 'B', axis: 'z', isMax: false, dir: new THREE.Vector3(0, 0, -1), rot: [0, Math.PI, 0] }
  ];

  function buildCube(N) {
    cubeOrder = N;
    cubies.forEach((c) => cubeRoot.remove(c));
    cubies.length = 0;
    pickableMeshes.length = 0;

    if (innerShieldMesh) {
      cubeRoot.remove(innerShieldMesh);
      innerShieldMesh = null;
    }

    const visualWidth = 3.0 + (N - 3) * 0.06;
    const unitScale = visualWidth / N;
    cubeRoot.scale.setScalar(unitScale);

    if (N >= 3) {
      innerShieldMesh = new THREE.Mesh(innerShieldGeo, innerShieldMat);
      innerShieldMesh.scale.setScalar(N - 1.25);
      cubeRoot.add(innerShieldMesh);
    }

    const half = (N - 1) / 2;
    const enablePieceShadow = N <= 6;

    for (let ix = 0; ix < N; ix++) {
      for (let iy = 0; iy < N; iy++) {
        for (let iz = 0; iz < N; iz++) {
          const isSurface =
            ix === 0 || ix === N - 1 ||
            iy === 0 || iy === N - 1 ||
            iz === 0 || iz === N - 1;

          if (!isSurface) continue;

          const x = ix - half;
          const y = iy - half;
          const z = iz - half;

          const cubie = new THREE.Group();
          cubie.position.set(x, y, z);
          cubie.userData = {
            homePos: new THREE.Vector3(x, y, z),
            gridIndex: { x: ix, y: iy, z: iz }
          };

          const bodyMesh = new THREE.Mesh(cubieGeo, coreMaterial);
          bodyMesh.castShadow = enablePieceShadow;
          bodyMesh.receiveShadow = enablePieceShadow;
          bodyMesh.userData.cubie = cubie;
          cubie.add(bodyMesh);
          pickableMeshes.push(bodyMesh);

          FACE_DEFS.forEach((face) => {
            const idxVal = face.axis === 'x' ? ix : face.axis === 'y' ? iy : iz;
            const isOuterFace = face.isMax ? idxVal === N - 1 : idxVal === 0;

            if (isOuterFace) {
              const sticker = new THREE.Mesh(stickerGeo, faceMaterials[face.key]);
              const offset = CUBIE_SIZE / 2 + 0.006;
              sticker.position.set(
                face.dir.x * offset,
                face.dir.y * offset,
                face.dir.z * offset
              );
              sticker.rotation.set(face.rot[0], face.rot[1], face.rot[2]);
              sticker.castShadow = false;
              sticker.receiveShadow = enablePieceShadow;
              sticker.userData = {
                cubie: cubie,
                localNormal: face.dir.clone(),
                faceKey: face.key
              };
              cubie.add(sticker);
              pickableMeshes.push(sticker);
            }
          });

          cubeRoot.add(cubie);
          cubies.push(cubie);
        }
      }
    }

    document.getElementById('current-order-badge').textContent = `${N}×${N}`;
    const maxDepth = Math.floor(N / 2);
    if (selectedLayerDepth > maxDepth) selectedLayerDepth = 1;
    renderDepthSelector(maxDepth);
    loadBestTimeForOrder(N);
  }

  function renderDepthSelector(maxDepth) {
    const containerEl = document.getElementById('depth-selector');
    containerEl.innerHTML = '';
    if (maxDepth <= 1) return;

    for (let d = 1; d <= maxDepth; d++) {
      const btn = document.createElement('button');
      btn.type = 'button';
      btn.className = `depth-btn btn-action ${d === selectedLayerDepth ? 'active' : ''}`;
      btn.textContent = `第${d}层`;
      btn.title = `操作从外往里第 ${d} 层`;
      btn.addEventListener('click', () => {
        selectedLayerDepth = d;
        renderDepthSelector(maxDepth);
        updateFaceButtonLabels();
      });
      containerEl.appendChild(btn);
    }
  }

  // ==========================================
  // 5. 全景球面相机控制
  // ==========================================
  const camState = {
    theta: Math.PI / 4,
    phi: Math.PI / 3.1,
    radius: 8.6,
    targetTheta: Math.PI / 4,
    targetPhi: Math.PI / 3.1,
    targetRadius: 8.6
  };

  function fitCameraToScreen() {
    const aspect = window.innerWidth / window.innerHeight;
    camera.aspect = aspect;
    camera.updateProjectionMatrix();
    renderer.setSize(window.innerWidth, window.innerHeight);
    const baseRadius = aspect < 0.75 ? 10.8 : aspect < 1.0 ? 9.6 : 8.4;
    camState.targetRadius = baseRadius;
  }

  fitCameraToScreen();
  camState.radius = camState.targetRadius;

  function updateCameraTransform() {
    camState.theta += (camState.targetTheta - camState.theta) * 0.16;
    camState.phi += (camState.targetPhi - camState.phi) * 0.16;
    camState.radius += (camState.targetRadius - camState.radius) * 0.16;

    camState.phi = Math.max(0.18, Math.min(Math.PI - 0.18, camState.phi));
    camState.targetPhi = Math.max(0.18, Math.min(Math.PI - 0.18, camState.targetPhi));

    const x = camState.radius * Math.sin(camState.phi) * Math.sin(camState.theta);
    const y = camState.radius * Math.cos(camState.phi);
    const z = camState.radius * Math.sin(camState.phi) * Math.cos(camState.theta);

    camera.position.set(x, y, z);
    camera.lookAt(0, -0.08, 0);
  }

  window.addEventListener('resize', fitCameraToScreen);

  // ==========================================
  // 6. 魔方旋转动画队列、计时器与复原检测
  // ==========================================
  let isAnimating = false;
  const moveQueue = [];
  const moveHistory = [];

  let gameState = 'IDLE';
  let moveCount = 0;
  let startTime = 0;
  let elapsedTime = 0;
  let bestTime = null;

  function formatTime(ms) {
    const totalTenths = Math.floor(ms / 100);
    const tenths = totalTenths % 10;
    const totalSeconds = Math.floor(totalTenths / 10);
    const secs = totalSeconds % 60;
    const mins = Math.floor(totalSeconds / 60);
    return (
      String(mins).padStart(2, '0') +
      ':' +
      String(secs).padStart(2, '0') +
      '.' +
      tenths
    );
  }

  function loadBestTimeForOrder(N) {
    bestTime = null;
    try {
      const saved = localStorage.getItem(`real_cube_3d_best_ms_${N}`);
      if (saved) {
        bestTime = parseFloat(saved);
      }
    } catch (e) {}
    document.getElementById('best-display').textContent =
      bestTime !== null ? formatTime(bestTime) : '--:--.-';
  }

  function updateStatusUI() {
    const badge = document.getElementById('status-badge');
    const undoBtn = document.getElementById('btn-undo');
    undoBtn.disabled =
      moveHistory.length === 0 || gameState === 'SOLVING' || gameState === 'SCRAMBLING';

    document.getElementById('moves-display').textContent = String(moveCount);

    if (gameState === 'SOLVED') {
      badge.textContent = '复原成功!';
      badge.className = 'status-badge solved-badge';
    } else if (gameState === 'PLAYING') {
      badge.textContent = '挑战计时中';
      badge.className = 'status-badge';
      badge.style.background = 'rgba(245, 158, 11, 0.2)';
      badge.style.color = '#fcd34d';
      badge.style.borderColor = 'rgba(245, 158, 11, 0.4)';
    } else if (gameState === 'SCRAMBLING') {
      badge.textContent = '正在打乱...';
      badge.className = 'status-badge';
      badge.style.background = 'rgba(56, 189, 248, 0.2)';
      badge.style.color = '#7dd3fc';
      badge.style.borderColor = 'rgba(56, 189, 248, 0.4)';
    } else if (gameState === 'SOLVING') {
      badge.textContent = '智能还原中...';
      badge.className = 'status-badge';
      badge.style.background = 'rgba(168, 85, 247, 0.2)';
      badge.style.color = '#d8b4fe';
      badge.style.borderColor = 'rgba(168, 85, 247, 0.4)';
    } else {
      const solved = checkIsSolved();
      badge.textContent = solved ? '已复原' : '自由练习';
      badge.className = 'status-badge';
      if (solved) {
        badge.style.background = 'rgba(16, 185, 129, 0.2)';
        badge.style.color = '#6ee7b7';
        badge.style.borderColor = 'rgba(16, 185, 129, 0.3)';
      } else {
        badge.style.background = 'rgba(100, 116, 139, 0.2)';
        badge.style.color = '#cbd5e1';
        badge.style.borderColor = 'rgba(100, 116, 139, 0.3)';
      }
    }
  }

  function checkIsSolved() {
    if (cubies.length === 0) return true;
    const refQuat = cubies[0].quaternion;
    for (let i = 0; i < cubies.length; i++) {
      const c = cubies[i];
      if (Math.abs(c.quaternion.dot(refQuat)) < 0.998) {
        return false;
      }
      const expectedPos = c.userData.homePos.clone().applyQuaternion(refQuat);
      if (expectedPos.distanceTo(c.position) > 0.15) {
        return false;
      }
    }
    return true;
  }

  function enqueueMove({
    axis,
    layerIndex,
    angle,
    recordHistory = true,
    isUserMove = false,
    duration = 175
  }) {
    moveQueue.push({ axis, layerIndex, angle, recordHistory, isUserMove, duration });
    processMoveQueue();
  }

  let activeAnimation = null;

  function processMoveQueue() {
    if (isAnimating || moveQueue.length === 0) return;

    const move = moveQueue.shift();
    isAnimating = true;

    if (move.isUserMove && gameState === 'IDLE') {
      if (!checkIsSolved()) {
        gameState = 'PLAYING';
        startTime = performance.now() - elapsedTime;
        postToHost('game.start');
      }
    }

    if (move.isUserMove && (gameState === 'PLAYING' || gameState === 'IDLE')) {
      moveCount++;
      postToHost('game.progress', { score: moveCount });
    }

    if (move.recordHistory) {
      const last = moveHistory[moveHistory.length - 1];
      if (
        last &&
        last.axis === move.axis &&
        last.layerIndex === move.layerIndex &&
        Math.abs(last.angle + move.angle) < 0.001
      ) {
        moveHistory.pop();
      } else {
        moveHistory.push({
          axis: move.axis,
          layerIndex: move.layerIndex,
          angle: move.angle
        });
      }
    }

    sound.playSnap(move.duration < 120);

    pivotGroup.rotation.set(0, 0, 0);
    pivotGroup.updateMatrixWorld();

    const targetCubies = [];
    cubies.forEach((c) => {
      if (c.userData.gridIndex[move.axis] === move.layerIndex) {
        targetCubies.push(c);
        pivotGroup.attach(c);
      }
    });

    activeAnimation = {
      move,
      targetCubies,
      startTime: performance.now()
    };
    updateStatusUI();
  }

  function updateActiveAnimation(now) {
    if (!activeAnimation) return;

    const { move, targetCubies, startTime: animStart } = activeAnimation;
    const progress = Math.min(1, (now - animStart) / move.duration);
    const c1 = 1.28;
    const c3 = c1 + 1;
    const eased =
      progress < 1
        ? 1 + c3 * Math.pow(progress - 1, 3) + c1 * Math.pow(progress - 1, 2)
        : 1;

    pivotGroup.rotation[move.axis] = move.angle * eased;
    pivotGroup.updateMatrixWorld();

    if (progress >= 1) {
      pivotGroup.rotation[move.axis] = move.angle;
      pivotGroup.updateMatrixWorld();

      const half = (cubeOrder - 1) / 2;

      targetCubies.forEach((c) => {
        cubeRoot.attach(c);

        const ix = Math.max(0, Math.min(cubeOrder - 1, Math.round(c.position.x + half)));
        const iy = Math.max(0, Math.min(cubeOrder - 1, Math.round(c.position.y + half)));
        const iz = Math.max(0, Math.min(cubeOrder - 1, Math.round(c.position.z + half)));

        c.userData.gridIndex.x = ix;
        c.userData.gridIndex.y = iy;
        c.userData.gridIndex.z = iz;

        c.position.set(ix - half, iy - half, iz - half);

        const euler = new THREE.Euler().setFromQuaternion(c.quaternion);
        const snap = Math.PI / 2;
        euler.x = Math.round(euler.x / snap) * snap;
        euler.y = Math.round(euler.y / snap) * snap;
        euler.z = Math.round(euler.z / snap) * snap;
        c.quaternion.setFromEuler(euler);
        c.updateMatrix();
      });

      activeAnimation = null;
      isAnimating = false;

      if (moveQueue.length === 0) {
        if (gameState === 'SCRAMBLING') {
          gameState = 'PLAYING';
          moveCount = 0;
          elapsedTime = 0;
          startTime = performance.now();
          postToHost('game.start');
        } else if (gameState === 'SOLVING') {
          gameState = 'IDLE';
          moveHistory.length = 0;
          sound.playVictory();
        } else if (gameState === 'PLAYING' && checkIsSolved()) {
          gameState = 'SOLVED';
          elapsedTime = performance.now() - startTime;
          document.getElementById('timer-display').textContent = formatTime(elapsedTime);
          if (bestTime === null || elapsedTime < bestTime) {
            bestTime = elapsedTime;
            document.getElementById('best-display').textContent = formatTime(bestTime);
            try {
              localStorage.setItem(`real_cube_3d_best_ms_${cubeOrder}`, String(bestTime));
            } catch (e) {}
          }
          moveHistory.length = 0;
          sound.playVictory();
          postToHost('game.result', { score: Math.max(1, Math.round(100000 / (elapsedTime / 1000 + moveCount))) });
        }
      }

      updateStatusUI();
      processMoveQueue();
    }
  }

  // ==========================================
  // 7. 3D 射线拾取 + 任意阶切面投影手势系统
  // ==========================================
  const raycaster = new THREE.Raycaster();
  const pointer = new THREE.Vector2();

  let interactionMode = null;
  let startPointer = { x: 0, y: 0 };
  let lastPointer = { x: 0, y: 0 };
  let hitInfo = null;
  let pinchStartDist = 0;
  let pinchStartRadius = 8.6;

  function getNormalizedPointer(clientX, clientY) {
    const rect = renderer.domElement.getBoundingClientRect();
    pointer.x = ((clientX - rect.left) / rect.width) * 2 - 1;
    pointer.y = -((clientY - rect.top) / rect.height) * 2 + 1;
  }

  function onPointerDown(clientX, clientY, isRightClick = false) {
    sound.init();
    startPointer = { x: clientX, y: clientY };
    lastPointer = { x: clientX, y: clientY };

    if (isRightClick || isAnimating || gameState === 'SCRAMBLING' || gameState === 'SOLVING') {
      interactionMode = 'ORBIT';
      return;
    }

    getNormalizedPointer(clientX, clientY);
    raycaster.setFromCamera(pointer, camera);
    const intersects = raycaster.intersectObjects(pickableMeshes, false);

    if (intersects.length > 0) {
      const hit = intersects[0];
      const cubie = hit.object.userData.cubie;
      if (cubie) {
        const normalMatrix = new THREE.Matrix3().getNormalMatrix(hit.object.matrixWorld);
        const worldNormal = hit.face.normal.clone().applyMatrix3(normalMatrix).normalize();

        const absX = Math.abs(worldNormal.x);
        const absY = Math.abs(worldNormal.y);
        const absZ = Math.abs(worldNormal.z);
        const snappedNormal = new THREE.Vector3();
        if (absX >= absY && absX >= absZ) {
          snappedNormal.set(Math.sign(worldNormal.x), 0, 0);
        } else if (absY >= absX && absY >= absZ) {
          snappedNormal.set(0, Math.sign(worldNormal.y), 0);
        } else {
          snappedNormal.set(0, 0, Math.sign(worldNormal.z));
        }

        hitInfo = {
          cubie,
          point: hit.point.clone(),
          normal: snappedNormal
        };
        interactionMode = 'SLICE';
        return;
      }
    }

    interactionMode = 'ORBIT';
  }

  function onPointerMove(clientX, clientY) {
    if (!interactionMode) return;

    const dx = clientX - startPointer.x;
    const dy = clientY - startPointer.y;

    if (interactionMode === 'ORBIT') {
      const moveX = clientX - lastPointer.x;
      const moveY = clientY - lastPointer.y;
      lastPointer = { x: clientX, y: clientY };

      const sensitivity = 0.0072;
      camState.targetTheta -= moveX * sensitivity;
      camState.targetPhi -= moveY * sensitivity;
      return;
    }

    if (interactionMode === 'SLICE' && hitInfo) {
      const dist = Math.hypot(dx, dy);
      if (dist < 12) return;

      const dragVec2D = new THREE.Vector2(dx, dy).normalize();
      const { cubie, point, normal } = hitInfo;

      const axes = [
        { name: 'x', vec: new THREE.Vector3(1, 0, 0) },
        { name: 'y', vec: new THREE.Vector3(0, 1, 0) },
        { name: 'z', vec: new THREE.Vector3(0, 0, 1) }
      ];

      const screenOrigin = projectToScreen(point);
      let bestMatch = null;
      let maxDot = -1;

      axes.forEach((tangentAxis) => {
        if (Math.abs(tangentAxis.vec.dot(normal)) > 0.5) return;

        for (const dirSign of [1, -1]) {
          const tangentDir = tangentAxis.vec.clone().multiplyScalar(dirSign);
          const testWorldPt = point.clone().add(tangentDir.clone().multiplyScalar(0.5));
          const screenTestPt = projectToScreen(testWorldPt);
          const screenVec = new THREE.Vector2(
            screenTestPt.x - screenOrigin.x,
            screenTestPt.y - screenOrigin.y
          );

          if (screenVec.lengthSq() < 0.0001) continue;
          screenVec.normalize();

          const alignment = screenVec.dot(dragVec2D);
          if (alignment > maxDot) {
            maxDot = alignment;
            const rotAxisVec = new THREE.Vector3().crossVectors(normal, tangentDir);
            let axisName = 'x';
            let angleSign = 1;
            if (Math.abs(rotAxisVec.x) > 0.5) {
              axisName = 'x';
              angleSign = Math.sign(rotAxisVec.x);
            } else if (Math.abs(rotAxisVec.y) > 0.5) {
              axisName = 'y';
              angleSign = Math.sign(rotAxisVec.y);
            } else {
              axisName = 'z';
              angleSign = Math.sign(rotAxisVec.z);
            }

            bestMatch = {
              axis: axisName,
              layerIndex: cubie.userData.gridIndex[axisName],
              angle: angleSign * (Math.PI / 2)
            };
          }
        }
      });

      if (bestMatch) {
        enqueueMove({
          axis: bestMatch.axis,
          layerIndex: bestMatch.layerIndex,
          angle: bestMatch.angle,
          recordHistory: true,
          isUserMove: true,
          duration: 165
        });
        const hint = document.getElementById('gesture-hint');
        if (hint) hint.style.opacity = '0';
      }

      interactionMode = null;
      hitInfo = null;
    }
  }

  function onPointerUp() {
    interactionMode = null;
    hitInfo = null;
  }

  function projectToScreen(worldVec) {
    const v = worldVec.clone().project(camera);
    return {
      x: ((v.x + 1) * window.innerWidth) / 2,
      y: ((-v.y + 1) * window.innerHeight) / 2
    };
  }

  const domElem = renderer.domElement;
  domElem.addEventListener('contextmenu', (e) => e.preventDefault());
  domElem.addEventListener('mousedown', (e) => {
    onPointerDown(e.clientX, e.clientY, e.button === 2);
  });
  window.addEventListener('mousemove', (e) => {
    onPointerMove(e.clientX, e.clientY);
  });
  window.addEventListener('mouseup', onPointerUp);

  domElem.addEventListener(
    'wheel',
    (e) => {
      e.preventDefault();
      camState.targetRadius = Math.max(
        5.2,
        Math.min(15.0, camState.targetRadius + e.deltaY * 0.005)
      );
    },
    { passive: false }
  );

  domElem.addEventListener(
    'touchstart',
    (e) => {
      e.preventDefault();
      if (e.touches.length === 1) {
        const t = e.touches[0];
        onPointerDown(t.clientX, t.clientY, false);
      } else if (e.touches.length === 2) {
        interactionMode = 'PINCH';
        const dx = e.touches[0].clientX - e.touches[1].clientX;
        const dy = e.touches[0].clientY - e.touches[1].clientY;
        pinchStartDist = Math.hypot(dx, dy);
        pinchStartRadius = camState.targetRadius;
      }
    },
    { passive: false }
  );

  domElem.addEventListener(
    'touchmove',
    (e) => {
      e.preventDefault();
      if (e.touches.length === 1 && interactionMode !== 'PINCH') {
        const t = e.touches[0];
        onPointerMove(t.clientX, t.clientY);
      } else if (e.touches.length === 2 && interactionMode === 'PINCH') {
        const dx = e.touches[0].clientX - e.touches[1].clientX;
        const dy = e.touches[0].clientY - e.touches[1].clientY;
        const dist = Math.hypot(dx, dy);
        if (pinchStartDist > 10) {
          const scale = pinchStartDist / dist;
          camState.targetRadius = Math.max(5.2, Math.min(15.0, pinchStartRadius * scale));
        }
      }
    },
    { passive: false }
  );

  domElem.addEventListener('touchend', () => {
    onPointerUp();
  });

  // ==========================================
  // 8. 3~9阶切换与全阶层深按钮控制
  // ==========================================
  let isCounterClockwise = false;
  const btnDirToggle = document.getElementById('btn-dir-toggle');

  function getFaceSpec(face, depth, order) {
    const d = Math.max(1, Math.min(Math.floor(order / 2), depth)) - 1;
    switch (face) {
      case 'U':
        return { axis: 'y', layerIndex: order - 1 - d, angle: -Math.PI / 2 };
      case 'D':
        return { axis: 'y', layerIndex: d, angle: Math.PI / 2 };
      case 'R':
        return { axis: 'x', layerIndex: order - 1 - d, angle: -Math.PI / 2 };
      case 'L':
        return { axis: 'x', layerIndex: d, angle: Math.PI / 2 };
      case 'F':
        return { axis: 'z', layerIndex: order - 1 - d, angle: -Math.PI / 2 };
      case 'B':
        return { axis: 'z', layerIndex: d, angle: Math.PI / 2 };
      default:
        return null;
    }
  }

  function updateFaceButtonLabels() {
    btnDirToggle.textContent = isCounterClockwise ? "逆时针 (')" : '顺时针';
    btnDirToggle.className = isCounterClockwise
      ? 'depth-btn btn-action active'
      : 'depth-btn btn-action';
    if (isCounterClockwise) {
      btnDirToggle.style.background = 'rgba(244, 63, 94, 0.25)';
      btnDirToggle.style.borderColor = 'rgba(244, 63, 94, 0.45)';
      btnDirToggle.style.color = '#fda4af';
    } else {
      btnDirToggle.style.background = 'rgba(245, 158, 11, 0.2)';
      btnDirToggle.style.borderColor = 'rgba(245, 158, 11, 0.4)';
      btnDirToggle.style.color = '#fcd34d';
    }

    document.querySelectorAll('.face-btn').forEach((btn) => {
      const face = btn.getAttribute('data-face');
      const labelSpan = btn.querySelector('.face-label');
      if (labelSpan) {
        const prefix = selectedLayerDepth > 1 ? String(selectedLayerDepth) : '';
        labelSpan.textContent = prefix + face + (isCounterClockwise ? "'" : '');
      }
    });
  }

  btnDirToggle.addEventListener('click', () => {
    isCounterClockwise = !isCounterClockwise;
    updateFaceButtonLabels();
  });

  document.querySelectorAll('.face-btn').forEach((btn) => {
    btn.addEventListener('click', () => {
      if (gameState === 'SCRAMBLING' || gameState === 'SOLVING') return;
      const face = btn.getAttribute('data-face');
      const spec = getFaceSpec(face, selectedLayerDepth, cubeOrder);
      if (!spec) return;
      const sign = isCounterClockwise ? -1 : 1;
      enqueueMove({
        axis: spec.axis,
        layerIndex: spec.layerIndex,
        angle: spec.angle * sign,
        recordHistory: true,
        isUserMove: true,
        duration: 175
      });
    });
  });

  document.querySelectorAll('.order-btn').forEach((btn) => {
    btn.addEventListener('click', () => {
      const newOrder = parseInt(btn.getAttribute('data-order'), 10);
      if (!newOrder || newOrder === cubeOrder) return;

      document.querySelectorAll('.order-btn').forEach((b) => b.classList.remove('active'));
      btn.classList.add('active');

      moveQueue.length = 0;
      moveHistory.length = 0;
      activeAnimation = null;
      isAnimating = false;
      gameState = 'IDLE';
      moveCount = 0;
      elapsedTime = 0;
      document.getElementById('timer-display').textContent = '00:00.0';

      buildCube(newOrder);
      updateFaceButtonLabels();
      updateStatusUI();
      sound.playSnap(true);
    });
  });

  window.addEventListener('keydown', (e) => {
    if (e.target && (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA')) return;
    const key = e.key.toUpperCase();
    if (['1', '2', '3', '4'].includes(key)) {
      const d = parseInt(key, 10);
      const maxD = Math.floor(cubeOrder / 2);
      if (d <= maxD) {
        selectedLayerDepth = d;
        renderDepthSelector(maxD);
        updateFaceButtonLabels();
      }
      return;
    }
    if (['U', 'D', 'L', 'R', 'F', 'B'].includes(key) && gameState !== 'SCRAMBLING' && gameState !== 'SOLVING') {
      const spec = getFaceSpec(key, selectedLayerDepth, cubeOrder);
      if (!spec) return;
      const sign = e.shiftKey ? -1 : 1;
      enqueueMove({
        axis: spec.axis,
        layerIndex: spec.layerIndex,
        angle: spec.angle * sign,
        recordHistory: true,
        isUserMove: true,
        duration: 165
      });
    }
  });

  document.getElementById('btn-scramble').addEventListener('click', () => {
    if (isAnimating || moveQueue.length > 0) return;
    gameState = 'SCRAMBLING';
    moveCount = 0;
    elapsedTime = 0;
    document.getElementById('timer-display').textContent = '00:00.0';

    const axes = ['x', 'y', 'z'];
    const angles = [Math.PI / 2, -Math.PI / 2];
    const scrambleCount = 16 + cubeOrder * 3;
    const stepDuration = Math.max(45, 95 - cubeOrder * 5);

    let lastAxis = null;
    let lastLayer = null;

    for (let i = 0; i < scrambleCount; i++) {
      let axis, layerIndex;
      do {
        axis = axes[Math.floor(Math.random() * axes.length)];
        layerIndex = Math.floor(Math.random() * cubeOrder);
      } while (
        (axis === lastAxis && layerIndex === lastLayer) ||
        (cubeOrder % 2 === 1 && layerIndex === Math.floor(cubeOrder / 2))
      );

      lastAxis = axis;
      lastLayer = layerIndex;
      const angle = angles[Math.floor(Math.random() * angles.length)];

      enqueueMove({
        axis,
        layerIndex,
        angle,
        recordHistory: true,
        isUserMove: false,
        duration: stepDuration
      });
    }
    updateStatusUI();
  });

  document.getElementById('btn-solve').addEventListener('click', () => {
    if (isAnimating || moveQueue.length > 0) return;
    if (moveHistory.length === 0 || checkIsSolved()) {
      resetCubeInstantly();
      return;
    }
    gameState = 'SOLVING';
    const stepsToReverse = moveHistory.slice().reverse();
    moveHistory.length = 0;
    const solveSpeed = Math.max(45, Math.min(90, Math.floor(1800 / stepsToReverse.length)));

    stepsToReverse.forEach((step) => {
      enqueueMove({
        axis: step.axis,
        layerIndex: step.layerIndex,
        angle: -step.angle,
        recordHistory: false,
        isUserMove: false,
        duration: solveSpeed
      });
    });
    updateStatusUI();
  });

  document.getElementById('btn-undo').addEventListener('click', () => {
    if (isAnimating || moveQueue.length > 0 || moveHistory.length === 0) return;
    const lastStep = moveHistory.pop();
    if (moveCount > 0) moveCount--;
    enqueueMove({
      axis: lastStep.axis,
      layerIndex: lastStep.layerIndex,
      angle: -lastStep.angle,
      recordHistory: false,
      isUserMove: false,
      duration: 150
    });
    updateStatusUI();
  });

  document.getElementById('btn-camera-reset').addEventListener('click', () => {
    camState.targetTheta = Math.PI / 4;
    camState.targetPhi = Math.PI / 3.1;
    fitCameraToScreen();
  });

  function resetCubeInstantly() {
    moveQueue.length = 0;
    moveHistory.length = 0;
    activeAnimation = null;
    isAnimating = false;
    gameState = 'IDLE';
    moveCount = 0;
    elapsedTime = 0;
    document.getElementById('timer-display').textContent = '00:00.0';
    buildCube(cubeOrder);
    updateStatusUI();
  }

  document.getElementById('btn-reset').addEventListener('click', resetCubeInstantly);

  const btnExit = document.getElementById('btn-exit');
  if (btnExit) {
    btnExit.addEventListener('click', () => {
      postToHost('game.exit');
    });
  }

  document.getElementById('btn-finish').addEventListener('click', () => {
    currentFinishIndex = (currentFinishIndex + 1) % FINISH_MODES.length;
    const mode = FINISH_MODES[currentFinishIndex];
    document.getElementById('finish-label').textContent = mode.name;
    applyFinishMode(mode.id);
    sound.playSnap(true);
  });

  document.getElementById('btn-sound').addEventListener('click', () => {
    sound.enabled = !sound.enabled;
    document.getElementById('icon-sound-on').classList.toggle('hidden', !sound.enabled);
    document.getElementById('icon-sound-off').classList.toggle('hidden', sound.enabled);
    if (sound.enabled) sound.playSnap(true);
  });

  const modalOverlay = document.getElementById('modal-overlay');
  function closeModal() {
    modalOverlay.classList.remove('active');
  }
  document.getElementById('btn-help').addEventListener('click', () => {
    modalOverlay.classList.add('active');
  });
  document.getElementById('btn-modal-close').addEventListener('click', closeModal);
  document.getElementById('btn-modal-ok').addEventListener('click', closeModal);
  modalOverlay.addEventListener('click', (e) => {
    if (e.target === modalOverlay) closeModal();
  });

  buildCube(3);
  applyFinishMode(FINISH_MODES[currentFinishIndex].id);
  updateFaceButtonLabels();

  // ==========================================
  // 9. 主渲染循环
  // ==========================================
  function animate(now) {
    requestAnimationFrame(animate);

    if (gameState === 'PLAYING') {
      elapsedTime = now - startTime;
      document.getElementById('timer-display').textContent = formatTime(elapsedTime);
    }

    updateActiveAnimation(now);
    updateCameraTransform();

    if (!isAnimating && interactionMode !== 'SLICE') {
      cubeRoot.position.y = Math.sin(now * 0.0018) * 0.04;
    }

    renderer.render(scene, camera);
  }

  window.addEventListener('load', () => {
    updateStatusUI();
    requestAnimationFrame(animate);
    // If not framed or already initialized, send ready
    postToHost('game.ready');
  });
})();
