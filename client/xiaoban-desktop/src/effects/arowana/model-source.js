const models = {
  arowana: {
    file: 'arowana.glb',
    url: 'https://res.xiangban.xyz/files/7b68680fe1757d50cd22ece7a157dc43.glb',
    sha256: 'bda5a32ecd70e6247e0b6946db7e1db4e2393e32aacad9416597f2dac8d714ad',
    bytes: 16247268,
  },
  guppy: {
    file: 'blue-guppy.glb',
    url: 'https://res.xiangban.xyz/files/892a1cdb58ffe09ae4b5d10582d0b8f8.glb',
    sha256: '67368dde88ab28b064e74bdd83e04bb9233f3fcbaef572a4438c8c180fc1d410',
    bytes: 7887380,
  },
};

async function checkedModelBytes(response, model) {
  if (!response.ok) throw new Error('鱼模型下载失败');
  const bytes = await response.arrayBuffer();
  if (bytes.byteLength !== model.bytes) throw new Error('鱼模型校验失败');
  const hash = Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256', bytes)), value => value.toString(16).padStart(2, '0')).join('');
  if (hash !== model.sha256) throw new Error('鱼模型校验失败');
  return bytes;
}

export async function loadModelBytes(species) {
  // The koi is small enough to ship with the client and works on the first offline launch.
  if (species === 'koi') {
    const response = await fetch('/effects/koi/kohaku-koi.glb');
    if (!response.ok) throw new Error('锦鲤模型加载失败');
    return response.arrayBuffer();
  }
  const model = models[species];
  if (!model) throw new Error('未知的鱼模型');
  if (import.meta.env?.DEV) {
    const response = await fetch(`/effects/arowana/${model.file}`);
    if (!response.ok) throw new Error('鱼模型加载失败');
    return response.arrayBuffer();
  }
  if (window.__TAURI__) {
    const cacheOrigin = location.protocol === 'tauri:' ? 'wallpaper-asset://localhost' : 'http://wallpaper-asset.localhost';
    try {
      const response = await fetch(`${cacheOrigin}/${model.sha256}`);
      return await checkedModelBytes(response, model);
    } catch { /* A missing or invalid cache can be fetched again. */ }
  }
  return checkedModelBytes(await fetch(model.url), model);
}
