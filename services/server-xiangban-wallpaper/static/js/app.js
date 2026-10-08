// WallpaperCollect Frontend Client
class WallpaperApp {
  constructor() {
    this.currentTab = 'all'; // 'all', 'videos', 'favorites'
    this.currentPage = 1;
    this.perPage = 24;
    this.hasMore = false;
    this.isLoading = false;
    this.currentQuery = '';
    this.wallpapers = [];
    this.favorites = new Set();
    this.currentModalItem = null;

    this.init();
  }

  async init() {
    this.loadFavoritesFromStorage();
    await this.fetchProvidersInfo();
    await this.fetchFavorites();
    this.triggerSearch();

    // Setup input listeners
    const searchInput = document.getElementById('search-input');
    const clearBtn = document.getElementById('clear-search-btn');
    if (searchInput) {
      searchInput.addEventListener('input', (e) => {
        clearBtn.style.display = e.target.value ? 'block' : 'none';
      });
    }

    // Keyboard shortcuts
    window.addEventListener('keydown', (e) => {
      if (e.key === 'Escape') {
        this.closeModal();
        this.closeSettingsModal();
      }
    });
  }

  loadFavoritesFromStorage() {
    try {
      const stored = localStorage.getItem('wc_favorites');
      if (stored) {
        const arr = JSON.parse(stored);
        this.favorites = new Set(arr);
        this.updateFavBadge();
      }
    } catch (_) {}
  }

  saveFavoritesToStorage() {
    try {
      localStorage.setItem('wc_favorites', JSON.stringify(Array.from(this.favorites)));
    } catch (_) {}
  }

  updateFavBadge() {
    const badge = document.getElementById('fav-count-badge');
    if (badge) {
      badge.textContent = this.favorites.size;
    }
  }

  switchTab(tab) {
    this.currentTab = tab;
    document.querySelectorAll('.tab-btn').forEach((btn) => {
      btn.classList.toggle('active', btn.dataset.tab === tab);
    });

    const searchInput = document.getElementById('search-input');
    const titleEl = document.getElementById('section-title');

    if (tab === 'all') {
      titleEl.textContent = this.currentQuery ? `"${this.currentQuery}" 搜索结果` : '今日精选壁纸';
    } else if (tab === 'videos') {
      titleEl.textContent = '4K/1080p 动态视频壁纸';
    } else if (tab === 'favorites') {
      titleEl.textContent = '我的收藏画廊';
    }

    this.currentPage = 1;
    this.wallpapers = [];
    this.triggerSearch();
  }

  searchTag(tag) {
    const searchInput = document.getElementById('search-input');
    if (searchInput) {
      searchInput.value = tag;
      document.getElementById('clear-search-btn').style.display = 'block';
    }
    this.triggerSearch();
  }

  clearSearch() {
    const searchInput = document.getElementById('search-input');
    if (searchInput) {
      searchInput.value = '';
      document.getElementById('clear-search-btn').style.display = 'none';
    }
    this.triggerSearch();
  }

  resetHome() {
    const searchInput = document.getElementById('search-input');
    if (searchInput) searchInput.value = '';
    const provSelect = document.getElementById('provider-select');
    if (provSelect) provSelect.value = 'all';
    const orientSelect = document.getElementById('orientation-select');
    if (orientSelect) orientSelect.value = 'all';
    this.switchTab('all');
  }

  handleSearchSubmit(e) {
    e.preventDefault();
    this.triggerSearch();
  }

  async triggerSearch(resetPage = true) {
    if (resetPage) {
      this.currentPage = 1;
      this.wallpapers = [];
    }

    if (this.currentTab === 'favorites') {
      await this.loadFavoritesView();
      return;
    }

    const searchInput = document.getElementById('search-input');
    const provSelect = document.getElementById('provider-select');
    const orientSelect = document.getElementById('orientation-select');

    const query = searchInput ? searchInput.value.trim() : '';
    const provider = provSelect ? provSelect.value : 'all';
    const orientation = orientSelect ? orientSelect.value : 'all';

    this.currentQuery = query;

    let url = '';
    if (this.currentTab === 'videos') {
      url = `/api/v1/wallpapers/videos?page=${this.currentPage}&per_page=${this.perPage}`;
      if (query) url += `&q=${encodeURIComponent(query)}`;
    } else if (query) {
      url = `/api/v1/wallpapers/search?q=${encodeURIComponent(query)}&page=${this.currentPage}&per_page=${this.perPage}`;
      if (provider !== 'all') url += `&provider=${provider}`;
      if (orientation !== 'all') url += `&orientation=${orientation}`;
    } else {
      url = `/api/v1/wallpapers/curated?page=${this.currentPage}&per_page=${this.perPage}`;
    }

    await this.fetchWallpapers(url);
  }

  async fetchWallpapers(url) {
    this.setLoading(true);

    try {
      const res = await fetch(url);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);
      const data = await res.json();

      if (this.currentPage === 1) {
        this.wallpapers = data.items || [];
      } else {
        this.wallpapers = [...this.wallpapers, ...(data.items || [])];
      }

      this.hasMore = data.has_more;

      // Update Meta
      const countEl = document.getElementById('results-count');
      if (countEl) countEl.textContent = `共找到 ${data.total || this.wallpapers.len || 0} 张`;

      const timeEl = document.getElementById('exec-time');
      if (timeEl && data.execution_time_ms !== undefined) {
        timeEl.textContent = `耗时 ${data.execution_time_ms}ms · 数据源: ${(data.providers || []).join(', ')}`;
      }

      this.renderGrid();
    } catch (err) {
      console.error('Fetch wallpapers error:', err);
      this.showToast('获取壁纸失败，请检查网络或配置');
    } finally {
      this.setLoading(false);
    }
  }

  async loadMore() {
    if (this.isLoading || !this.hasMore) return;
    this.currentPage += 1;
    await this.triggerSearch(false);
  }

  async fetchFavorites() {
    try {
      const res = await fetch('/api/v1/favorites');
      if (res.ok) {
        const data = await res.json();
        if (data.items) {
          data.items.forEach((item) => this.favorites.add(item.id));
          this.saveFavoritesToStorage();
          this.updateFavBadge();
        }
      }
    } catch (_) {}
  }

  async loadFavoritesView() {
    this.setLoading(true);
    try {
      const res = await fetch('/api/v1/favorites');
      const data = await res.json();
      this.wallpapers = data.items || [];
      this.hasMore = false;

      const countEl = document.getElementById('results-count');
      if (countEl) countEl.textContent = `收藏夹共 ${this.wallpapers.length} 张`;

      const timeEl = document.getElementById('exec-time');
      if (timeEl) timeEl.textContent = '本地与 PostgreSQL 持久化收藏';

      this.renderGrid();
    } catch (e) {
      this.showToast('加载收藏失败');
    } finally {
      this.setLoading(false);
    }
  }

  setLoading(loading) {
    this.isLoading = loading;
    const loadingEl = document.getElementById('state-loading');
    if (loadingEl) {
      loadingEl.style.display = loading && this.currentPage === 1 ? 'block' : 'none';
    }

    const loadMoreBtn = document.getElementById('load-more-btn');
    if (loadMoreBtn) {
      loadMoreBtn.textContent = loading ? '加载中...' : '加载更多壁纸';
      loadMoreBtn.disabled = loading;
    }
  }

  renderGrid() {
    const grid = document.getElementById('wallpapers-grid');
    const emptyEl = document.getElementById('state-empty');
    const loadMoreWrapper = document.getElementById('load-more-wrapper');

    if (!grid) return;

    if (this.wallpapers.length === 0 && !this.isLoading) {
      grid.innerHTML = '';
      if (emptyEl) emptyEl.style.display = 'block';
      if (loadMoreWrapper) loadMoreWrapper.style.display = 'none';
      return;
    }

    if (emptyEl) emptyEl.style.display = 'none';
    if (loadMoreWrapper) {
      loadMoreWrapper.style.display = this.hasMore ? 'block' : 'none';
    }

    grid.innerHTML = this.wallpapers.map((item) => this.createCardHtml(item)).join('');
  }

  createCardHtml(item) {
    const isFav = this.favorites.has(item.id);
    const isVideo = item.media_type === 'video';
    const isPortrait = item.orientation === 'portrait';

    const resLabel = `${item.resolution.width}×${item.resolution.height}`;

    let videoBadge = '';
    if (isVideo) {
      const duration = item.video ? `${item.video.duration_secs}s` : 'Live';
      videoBadge = `
        <div class="video-badge">
          <span class="pulse-dot"></span>
          <span>4K Live (${duration})</span>
        </div>
      `;
    }

    const providerClass = `provider-tag ${item.provider}`;

    return `
      <div class="wallpaper-card" onclick="app.openDetailModal('${item.id}')">
        <div class="card-media-wrapper ${isPortrait ? 'portrait' : ''}">
          <img class="card-img" src="${item.preview_url}" alt="${this.escapeHtml(item.title)}" loading="lazy" />
          ${videoBadge}
          <div class="resolution-badge">${resLabel}</div>

          <!-- Hover Overlay -->
          <div class="card-hover-actions" onclick="event.stopPropagation()">
            <button class="action-icon-btn ${isFav ? 'favorited' : ''}" title="${isFav ? '取消收藏' : '加入收藏'}" onclick="app.toggleFavorite('${item.id}')">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="${isFav ? 'currentColor' : 'none'}" stroke="currentColor" stroke-width="2">
                <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z"></path>
              </svg>
            </button>
            <a href="${item.downloads.original}" target="_blank" download class="action-icon-btn" title="直接下载原图">
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
                <polyline points="7 10 12 15 17 10"></polyline>
                <line x1="12" y1="15" x2="12" y2="3"></line>
              </svg>
            </a>
          </div>
        </div>

        <div class="card-info">
          <div class="card-title" title="${this.escapeHtml(item.title)}">${this.escapeHtml(item.title)}</div>
          <div class="card-bottom-row">
            <span class="${providerClass}">${item.provider}</span>
            <div class="card-author">
              <span>👤 ${this.escapeHtml(item.author.name || 'Anonymous')}</span>
            </div>
          </div>
        </div>
      </div>
    `;
  }

  async toggleFavorite(itemId) {
    const item = this.wallpapers.find((w) => w.id === itemId);
    const isFav = this.favorites.has(itemId);

    if (isFav) {
      // Remove
      this.favorites.delete(itemId);
      this.saveFavoritesToStorage();
      this.updateFavBadge();
      this.renderGrid();

      try {
        await fetch(`/api/v1/favorites/${itemId}`, { method: 'DELETE' });
      } catch (_) {}

      this.showToast('已从收藏夹移除');
    } else if (item) {
      // Add
      this.favorites.add(itemId);
      this.saveFavoritesToStorage();
      this.updateFavBadge();
      this.renderGrid();

      try {
        await fetch('/api/v1/favorites', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify(item),
        });
      } catch (_) {}

      this.showToast('已保存至收藏夹');
    }
  }

  // Detail Lightbox Modal
  openDetailModal(itemId) {
    const item = this.wallpapers.find((w) => w.id === itemId);
    if (!item) return;

    this.currentModalItem = item;

    const modal = document.getElementById('detail-modal');
    const mediaContainer = document.getElementById('modal-media-container');
    const pill = document.getElementById('modal-provider-pill');
    const title = document.getElementById('modal-title');
    const desc = document.getElementById('modal-desc');
    const tagsContainer = document.getElementById('modal-tags');
    const mediaTypeEl = document.getElementById('modal-media-type');
    const resEl = document.getElementById('modal-resolution');
    const aspectEl = document.getElementById('modal-aspect');
    const authorLink = document.getElementById('modal-author-link');
    const downloadBtns = document.getElementById('modal-download-buttons');
    const favBtn = document.getElementById('modal-fav-btn');
    const sourceBtn = document.getElementById('modal-source-btn');

    // Populate media (Image or Video)
    if (item.media_type === 'video' && item.video) {
      const bestVideo = item.video.qualities[0] || { url: item.downloads.original };
      mediaContainer.innerHTML = `
        <video controls autoplay loop muted style="width: 100%; height: 100%;">
          <source src="${bestVideo.url}" type="video/mp4">
          您的浏览器不支持视频播放
        </video>
      `;
    } else {
      mediaContainer.innerHTML = `
        <img src="${item.regular_url}" alt="${this.escapeHtml(item.title)}" />
      `;
    }

    pill.textContent = item.provider;
    pill.className = `provider-pill ${item.provider}`;
    title.textContent = item.title;
    desc.textContent = item.description || '无附加描述';

    // Tags
    tagsContainer.innerHTML = item.tags
      .map(
        (t) =>
          `<button class="chip" onclick="app.closeModal(); app.searchTag('${t}')">${t}</button>`
      )
      .join('');

    mediaTypeEl.textContent = item.media_type === 'video' ? '🎬 4K Live 动态视频' : '🖼️ 高清静态壁纸';
    resEl.textContent = `${item.resolution.width} × ${item.resolution.height}`;
    aspectEl.textContent = item.resolution.aspect_ratio || '16:9';

    authorLink.textContent = item.author.name;
    authorLink.href = item.author.profile_url || '#';

    sourceBtn.href = item.source_page_url;

    // Download options
    let buttonsHtml = '';
    if (item.media_type === 'video' && item.video) {
      buttonsHtml = item.video.qualities
        .map(
          (q) => `
        <a href="${q.url}" target="_blank" download class="btn-download-opt">
          <span>下载视频 ${q.quality} (${q.width}×${q.height})</span>
          <span class="download-tag">MP4</span>
        </a>
      `
        )
        .join('');
    } else {
      buttonsHtml += `
        <a href="${item.downloads.original}" target="_blank" download class="btn-download-opt">
          <span>下载最高原图分辨率 (${item.resolution.width}×${item.resolution.height})</span>
          <span class="download-tag">Original</span>
        </a>
      `;
      if (item.downloads.large) {
        buttonsHtml += `
          <a href="${item.downloads.large}" target="_blank" download class="btn-download-opt">
            <span>下载 4K 超高清标准版</span>
            <span class="download-tag">4K UHD</span>
          </a>
        `;
      }
      if (item.downloads.medium) {
        buttonsHtml += `
          <a href="${item.downloads.medium}" target="_blank" download class="btn-download-opt">
            <span>下载 1080p 桌面版</span>
            <span class="download-tag">1080p</span>
          </a>
        `;
      }
    }
    downloadBtns.innerHTML = buttonsHtml;

    // Favorite button state
    const isFav = this.favorites.has(item.id);
    favBtn.classList.toggle('active', isFav);
    favBtn.querySelector('span').textContent = isFav ? '已在收藏夹中' : '加入收藏';

    modal.style.display = 'flex';
  }

  toggleModalFavorite() {
    if (!this.currentModalItem) return;
    this.toggleFavorite(this.currentModalItem.id);

    const favBtn = document.getElementById('modal-fav-btn');
    const isFav = this.favorites.has(this.currentModalItem.id);
    favBtn.classList.toggle('active', isFav);
    favBtn.querySelector('span').textContent = isFav ? '已在收藏夹中' : '加入收藏';
  }

  closeModal() {
    const modal = document.getElementById('detail-modal');
    if (modal) {
      modal.style.display = 'none';
      const mediaContainer = document.getElementById('modal-media-container');
      if (mediaContainer) mediaContainer.innerHTML = '';
    }
    this.currentModalItem = null;
  }

  closeModalOnBackdrop(e) {
    if (e.target.id === 'detail-modal') {
      this.closeModal();
    }
  }

  // Settings & Providers Modal
  async openSettingsModal() {
    const modal = document.getElementById('settings-modal');
    await this.fetchProvidersInfo();
    if (modal) modal.style.display = 'flex';
  }

  closeSettingsModal() {
    const modal = document.getElementById('settings-modal');
    if (modal) modal.style.display = 'none';
  }

  closeSettingsOnBackdrop(e) {
    if (e.target.id === 'settings-modal') {
      this.closeSettingsModal();
    }
  }

  async fetchProvidersInfo() {
    try {
      const res = await fetch('/api/v1/providers');
      if (!res.ok) return;
      const data = await res.json();
      const listEl = document.getElementById('providers-list');

      if (listEl && data.providers) {
        listEl.innerHTML = data.providers
          .map((p) => {
            const statusClass = p.is_configured ? 'ready' : 'missing';
            const statusText = p.is_configured ? '已就绪 (Ready)' : '未配置 API Key';
            return `
            <div class="provider-row">
              <div>
                <div class="provider-title">${p.name}</div>
                <div style="font-size:0.8rem; color:var(--text-muted);">${p.description}</div>
              </div>
              <span class="status-badge ${statusClass}">${statusText}</span>
            </div>
          `;
          })
          .join('');
      }
    } catch (_) {}
  }

  async triggerCollectorSync() {
    const btn = document.getElementById('trigger-sync-btn');
    const resultBox = document.getElementById('sync-result');

    if (btn) {
      btn.disabled = true;
      btn.textContent = '正在采集 Unsplash、Pixabay & Pexels 壁纸中...';
    }

    try {
      const res = await fetch('/api/v1/collector/sync', { method: 'POST' });
      const data = await res.json();

      if (resultBox) {
        resultBox.style.display = 'block';
        resultBox.innerHTML = `
          <strong>采集任务完成！</strong><br>
          本次新增采集: <strong>${data.report.total_collected}</strong> 张壁纸与动态视频<br>
          已处理关键词: ${data.report.keywords_processed.join(', ')}<br>
          执行数据源: ${data.report.providers_used.join(', ')}
        `;
      }
      this.showToast(`采集成功！新增 ${data.report.total_collected} 张壁纸`);
      this.triggerSearch();
    } catch (e) {
      this.showToast('采集任务执行失败');
    } finally {
      if (btn) {
        btn.disabled = false;
        btn.textContent = '立即执行一键采集';
      }
    }
  }

  showToast(msg) {
    const toast = document.getElementById('toast');
    if (!toast) return;
    toast.textContent = msg;
    toast.classList.add('show');
    setTimeout(() => {
      toast.classList.remove('show');
    }, 3000);
  }

  refreshCurrentView() {
    this.triggerSearch();
    this.showToast('壁纸流已刷新');
  }

  escapeHtml(str) {
    if (!str) return '';
    return str
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#039;');
  }
}

// Initialize Global App
const app = new WallpaperApp();
