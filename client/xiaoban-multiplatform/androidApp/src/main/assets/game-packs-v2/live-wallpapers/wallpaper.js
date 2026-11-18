(() => {
  const options = new URLSearchParams(location.search);
  const scene = options.get('scene') === 'galaxy' ? 'galaxy' : 'aquarium';
  const fps = options.get('fps') === '15' ? 15 : 30;
  window.xbWallpaper = window.createWallpaperScene(document.querySelector('canvas'), scene, fps);
})();
