(() => {
  const options = new URLSearchParams(location.search);
  const scene = options.get('scene') === 'galaxy' ? 'galaxy' : 'aquarium';
  const fps = options.get('fps') === '15' ? 15 : 30;
  const width = Number(options.get('width'));
  const height = Number(options.get('height'));
  const resolution = width > 0 && height > 0 ? {width, height} : undefined;
  window.xbWallpaper = window.createWallpaperScene(document.querySelector('canvas'), scene, fps, resolution);
})();
