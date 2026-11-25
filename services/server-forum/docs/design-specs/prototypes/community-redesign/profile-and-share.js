(() => {
  'use strict';

  const esc = str => String(str ?? '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');

  const svgUri = m => 'data:image/svg+xml;utf8,' + encodeURIComponent(m.trim());

  const AVATAR_SELF = svgUri(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 128 128"><defs><linearGradient id="sky" x1="0" y1="0" x2="0" y2="1"><stop offset="0%" stop-color="#c9956b"/><stop offset="48%" stop-color="#dfba91"/><stop offset="100%" stop-color="#5d4d44"/></linearGradient></defs><rect width="128" height="128" fill="url(#sky)"/><g fill="#453d3b" opacity="0.72"><polygon points="10,96 24,38 38,96"/><polygon points="30,96 46,28 62,96"/><polygon points="64,96 80,32 96,96"/><polygon points="88,96 104,42 120,96"/></g><g fill="#2b2626"><polygon points="-4,114 14,44 32,114"/><polygon points="18,118 36,52 54,118"/><polygon points="74,118 92,46 110,118"/><polygon points="96,118 114,56 132,118"/></g><path d="M0 102 Q64 90 128 104 L128 128 L0 128 Z" fill="#dcd6d0" opacity="0.88"/><g transform="translate(36,80)"><rect x="6" y="16" width="44" height="22" rx="1.5" fill="#4f3629"/><polygon points="0,18 28,0 56,18" fill="#3a2820"/><rect x="23" y="23" width="10" height="15" rx="1" fill="#2e1d15"/><rect x="11" y="22" width="7" height="7" rx="1" fill="#f7c873"/><rect x="38" y="22" width="7" height="7" rx="1" fill="#f7c873"/></g></svg>`);
  const AVATAR_OTHER = svgUri(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 128 128"><defs><linearGradient id="s2" x1="0" y1="0" x2="0" y2="1"><stop offset="0%" stop-color="#87b5ba"/><stop offset="55%" stop-color="#cfe0db"/><stop offset="100%" stop-color="#35555c"/></linearGradient></defs><rect width="128" height="128" fill="url(#s2)"/><circle cx="88" cy="34" r="14" fill="#fff7e4" opacity="0.85"/><path d="M0 84 L34 46 L68 78 L96 52 L128 84 L128 128 L0 128 Z" fill="#4f757a"/><path d="M0 94 Q40 76 82 92 T128 90 L128 128 L0 128 Z" fill="#2e4a52"/></svg>`);

  const SVG = {
    user: '<circle cx="12" cy="8.5" r="3.5"/><path d="M5.2 19.8c.9-3.4 3.4-5.3 6.8-5.3s5.9 1.9 6.8 5.3"/>',
    copy: '<rect x="9" y="9" width="11" height="11" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>',
    bookmark: '<path d="M6 4h12a1 1 0 0 1 1 1v16l-7-4.5L5 21V5a1 1 0 0 1 1-1z"/>',
    folder: '<path d="M4 7a2 2 0 0 1 2-2h4l2 2h6a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2z"/>',
    bell: '<path d="M18 8a6 6 0 0 0-12 0c0 7-3 9-3 9h18s-3-2-3-9"/><path d="M13.73 21a2 2 0 0 1-3.46 0"/>',
    cpu: '<rect x="6" y="6" width="12" height="12" rx="2.5"/><rect x="9.5" y="9.5" width="5" height="5" rx="1"/><path d="M9 2v4M15 2v4M9 18v4M15 18v4M2 9h4M2 15h4M18 9h4M18 15h4"/>',
    monitor: '<rect x="3" y="4" width="18" height="12" rx="2"/><path d="M8 20h8M12 16v4"/>',
    help: '<circle cx="12" cy="12" r="9"/><path d="M9.5 9.2a2.6 2.6 0 0 1 5 1c0 1.6-2.5 2-2.5 3.3"/><circle cx="12" cy="16.8" r=".7" fill="currentColor" stroke="none"/>',
    chatGroup: '<path d="M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z"/>',
    gear: '<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>',
    chev: '<path d="M9 6l6 6-6 6"/>',
    back: '<path d="M15 6l-6 6 6 6"/>',
    more: '<circle cx="6" cy="12" r="1.4" fill="currentColor" stroke="none"/><circle cx="12" cy="12" r="1.4" fill="currentColor" stroke="none"/><circle cx="18" cy="12" r="1.4" fill="currentColor" stroke="none"/>',
    block: '<circle cx="12" cy="12" r="9"/><line x1="5.6" y1="5.6" x2="18.4" y2="18.4"/>',
    gamepad: '<rect x="3" y="7.5" width="18" height="10" rx="5"/><path d="M8 10.5v4M6 12.5h4"/><circle cx="15.5" cy="11.5" r=".8" fill="currentColor" stroke="none"/><circle cx="17.8" cy="13.5" r=".8" fill="currentColor" stroke="none"/>',
    box: '<rect x="4" y="4" width="16" height="16" rx="3"/><path d="M9 9h6M9 13h4"/>',
    play: '<polygon points="7 5 19 12 7 19 7 5"/>',
    edit: '<path d="M12 20h9"/><path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4z"/>',
    upload: '<path d="M12 16V4M7 9l5-5 5 5"/><path d="M4 16v3a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-3"/>',
    eye: '<path d="M2 12s3.8-6.5 10-6.5S22 12 22 12s-3.8 6.5-10 6.5S2 12 2 12z"/><circle cx="12" cy="12" r="3"/>',
    eyeOff: '<path d="M17.94 17.94A10.94 10.94 0 0 1 12 19c-6.2 0-10-7-10-7a18.4 18.4 0 0 1 5.06-5.94"/><path d="M9.9 4.24A10.6 10.6 0 0 1 12 4c6.2 0 10 8 10 8a18.5 18.5 0 0 1-2.16 3.19"/><line x1="3" y1="3" x2="21" y2="21"/>',
    spark: '<path d="M12 3l2.2 5.4L20 11l-5.8 2.2L12 19l-2.2-5.8L4 11l5.8-2.6z"/>',
    link: '<path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>',
    card: '<rect x="3" y="4" width="18" height="16" rx="3"/><path d="M7 9h6M7 13h10"/><circle cx="16.5" cy="9.5" r="1.5"/>',
    copyImg: '<rect x="9" y="9" width="11" height="11" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/><path d="M11 17l2.5-2.5 2 2 1.5-1.5 2 2"/>',
    download: '<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><path d="M7 10l5 5 5-5"/><path d="M12 15V3"/>',
    quote: '<path d="M4 6h16M4 12h12M4 18h8"/>',
    ext: '<path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><path d="M15 3h6v6"/><path d="M10 14L21 3"/>',
    trash: '<path d="M3 6h18"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/>',
    check: '<polyline points="20 6 9 17 4 12"/>',
    send: '<path d="M22 2 11 13"/><polygon points="22 2 15 22 11 13 2 9 22 2"/>',
    x: '<path d="M18 6 6 18M6 6l12 12"/>',
    plus: '<path d="M12 5v14M5 12h14"/>',
  };

  const ic = (name, size = 18) =>
    `<svg width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${SVG[name] || ''}</svg>`;

  const QR_SVG = `<svg viewBox="0 0 25 25" shape-rendering="crispEdges"><path fill="currentColor" d="M0 0h7v7H0zM1 1v5h5V1zM2 2h3v3H2zM18 0h7v7h-7zM19 1v5h5V1zM20 2h3v3h-3zM0 18h7v7H0zM1 19v5h5v-5zM2 20h3v3H2z"/><path fill="currentColor" d="M9 0h2v2H9zM13 0h3v2h-3zM9 3h3v2H9zM14 4h2v3h-2zM9 7h2v2H9zM12 7h4v2h-4zM0 9h3v2H0zM5 9h3v2H5zM10 10h2v3h-2zM14 10h3v2h-3zM19 9h2v3h-2zM23 9h2v2h-2zM1 12h2v3H1zM5 13h4v2H5zM11 14h3v2h-3zM16 13h3v3h-3zM21 13h4v2h-4zM0 16h2v1H0zM4 16h3v1H4zM9 17h3v2H9zM14 17h2v3h-2zM18 17h3v2h-3zM23 16h2v3h-2zM9 21h2v4H9zM13 21h3v2h-3zM17 20h2v5h-2zM21 20h4v2h-4zM12 23h4v2h-4zM20 23h2v2h-2zM23 23h2v2h-2z"/></svg>`;

  const ps = {
    loggedIn: true,
    checkedIn: false,
    profileView: 'self', // 'self' | 'other' | 'empty'
    profileTab: 'topics', // 'topics' | 'replies' | 'following' | 'followers'
    profileMoreOpen: false,
    otherFollowing: false,
    otherBlocked: false,
    worksKind: 'packs', // 'packs' | 'schemes'
    worksFilter: 'all', // 'all' | 'draft' | 'live' | 'review' | 'removed'
    worksEmpty: false,
    pcWorkMenuId: null,
    h5WorkSheet: null, // { kind, workId }
    h5ProfileSheet: false,
    notifTab: 'all', // 'all' | 'unread'
    notifEmpty: false,
    modal: null, // { device, action: 'remove'|'delete'|'block', kind?, workId? }
    works: [
      { id: 4, title: '像素弹球实验室', kind: '游戏', kindKey: 'game', version: null, status: 'draft', statusText: '草稿', storage: '仅本机', uses: 0, updated: '11-23' },
      { id: 1, title: '万磁搜', kind: '应用', kindKey: 'app', version: 'v3', status: 'review', statusText: '审核中', uses: 128, updated: '11-09' },
      { id: 2, title: '纸飞机滑翔', kind: '游戏', kindKey: 'game', version: 'v2', status: 'live', statusText: '已上线', uses: 540, updated: '11-18' },
      { id: 3, title: '网页 V2 预览验证', kind: '应用', kindKey: 'app', version: 'v1', status: 'removed', statusText: '已下架', uses: 12, updated: '11-22' },
    ],
    schemes: [
      { id: 101, title: '极简樱花书桌 · 桌面主题方案', kind: '桌面方案', kindKey: 'scheme', version: 'v1', status: 'live', statusText: '已上线', uses: 86, updated: '11-15' },
    ],
    notifs: [
      { id: 1, unread: true, actor: '林深见鹿', init: '林', color: '#4d7a7f', text: '评论了你的作品「万磁搜」', quote: '这个搜索响应很快！建议加一个按体积排序的选项会更好用。', time: '12 分钟前', tag: '评论' },
      { id: 2, unread: true, actor: '像素匠人', init: '像', color: '#b57452', text: '赞了你的作品「纸飞机滑翔」', quote: '', time: '1 小时前', tag: '点赞' },
      { id: 3, unread: false, actor: null, sys: true, text: '你的内容包「纸飞机滑翔」v2 通过了审核', quote: '', time: '昨天', tag: '审核通知' },
      { id: 4, unread: false, actor: '阿南不熬夜', init: '南', color: '#7a6899', text: '回复了你在「第三方模型接入踩坑记录」里的帖子', quote: '感谢整理，把上下文上限改成 128k 后确实稳了很多。', time: '3 天前', tag: '回复' },
    ],
    people: [
      { uid: 'u1', name: '林深见鹿', init: '林', color: '#4d7a7f', note: '活跃成员 · 24 帖子', followed: true },
      { uid: 'u2', name: '阿南不熬夜', init: '南', color: '#7a6899', note: '社区成员 · 11 帖子', followed: true },
      { uid: 'u3', name: '像素匠人', init: '像', color: '#b57452', note: '核心成员 · 36 帖子', followed: true },
      { uid: 'u4', name: '青禾笔记', init: '青', color: '#4d8c68', note: '社区成员 · 8 帖子', followed: false },
    ],
    topics: [
      { id: 103, title: '用手机端小伴 Agent 做了个纸飞机滑翔小游戏，分享完整提示词', excerpt: '从 0 到 1 迭代了三轮物理风场参数，补上了横屏安全区避让和本地存档，欢迎试玩提建议。', cat: '创作', time: '2 小时前', replies: 18, likes: 42, thumb: './assets/starry-nightrider.webp' },
      { id: 101, title: '第三方模型接入踩坑记录：Anthropic 代理与上下文窗口配置', excerpt: '实测几个常用中转站的响应速度和工具调用稳定性，附带我现在的默认参数配置清单。', cat: '学习与教程', time: '昨天', replies: 9, likes: 27, thumb: null },
      { id: 102, title: '给小伴「我的」页做了一版清爽通栏改版，大家觉得怎样？', excerpt: '统一了我的、个人主页、我的作品与通知四页的视觉层级，去掉外露积分与重复入口。', cat: '休闲', time: '3 天前', replies: 14, likes: 31, thumb: './assets/micro-gallery.webp' },
    ],
    replies: [
      { id: 21, topic: '银河动态壁纸 Shader 参数调校：低功耗下保持 30 帧丝滑星轨', excerpt: '实测把采样层数从 6 降到 3 后在小米 14 上几乎不发热，星轨层次感也没丢。', time: '3 小时前' },
      { id: 22, topic: '整理了 12 个适合新手拆解的 V2 协议小游戏开源范例', excerpt: '很赞的合集！建议补一个带横屏安全区示例的模板，新手很容易踩胶囊遮挡的坑。', time: '2 天前' },
    ],
  };

  const ds = {
    openDevice: null, // 'desktop' | 'mobile' | 'both' | null
    h5Mode: 'sheet', // 'sheet' | 'card'
    cardTheme: 'white', // 'white' | 'warm' | 'ink'
    showExcerpt: true,
    showQr: true,
    confirmDelete: false,
    copiedKey: '',
  };

  function unreadCount() {
    return ps.notifEmpty ? 0 : ps.notifs.filter(n => n.unread).length;
  }

  /* ==================== 01 · 「我的」页（方案 1 通栏精修） ==================== */
  function renderMePage(device) {
    const unread = unreadCount();
    return `
      <div class="ps-page">
        <div class="ps-demo-strip">
          <span class="ps-demo-title">「我的」· 方案 1 通栏精修</span>
          <div class="ps-seg" role="group" aria-label="登录状态">
            <button type="button" class="${ps.loggedIn ? 'is-active' : ''}" data-action="ps-login" data-val="1">已登录</button>
            <button type="button" class="${!ps.loggedIn ? 'is-active' : ''}" data-action="ps-login" data-val="0">未登录</button>
          </div>
        </div>

        <!-- 一体化顶区：名片 + 分割线 + 收藏/作品/通知三宫格 -->
        <section class="ps-slab">
          ${ps.loggedIn ? `
            <div class="ps-profile-row">
              <button type="button" class="ps-avatar-btn" data-action="go-page" data-page="profile" title="进入个人主页">
                <img src="${AVATAR_SELF}" alt="read">
              </button>
              <div class="ps-profile-info">
                <div class="ps-name-row">
                  <span class="ps-name">read</span>
                  <span class="ps-level-tag">活跃成员</span>
                </div>
                <div style="display:flex;align-items:center;gap:10px;flex-wrap:wrap">
                  <button type="button" class="ps-id-copy" data-action="ps-copy-id">
                    <span>ID 100482</span>${ic('copy', 12)}
                  </button>
                  <button type="button" class="ps-id-copy" data-action="ps-checkin" style="color:${ps.checkedIn ? 'var(--ink-3)' : 'var(--ink-2)'}">
                    ${ps.checkedIn ? `${ic('check', 12)} 今日已签到` : '每日签到'}
                  </button>
                </div>
              </div>
              <button type="button" class="ps-home-entry" data-action="go-page" data-page="profile">
                <span>主页</span>${ic('chev', 15)}
              </button>
            </div>
          ` : `
            <div class="ps-profile-row">
              <div class="ps-avatar-btn">${ic('user', 26)}</div>
              <div class="ps-profile-info">
                <div class="ps-name-row"><span class="ps-name">未登录</span></div>
                <div class="ps-bio">登录后同步收藏、作品草稿与通知</div>
              </div>
              <button type="button" class="ps-pill-act is-primary" data-action="ps-login" data-val="1">登录</button>
            </div>
          `}

          <div class="ps-slab-divider"></div>

          <div class="ps-a-quick">
            <button type="button" class="ps-quick-tile" data-action="go-page" data-page="opensource">
              <span class="ps-quick-ico">${ic('bookmark', 20)}</span>
              <span class="ps-quick-label">我的收藏</span>
            </button>
            <button type="button" class="ps-quick-tile" data-action="go-page" data-page="works">
              <span class="ps-quick-ico">${ic('folder', 20)}</span>
              <span class="ps-quick-label">我的作品</span>
            </button>
            <button type="button" class="ps-quick-tile" data-action="go-page" data-page="notifications">
              <span class="ps-quick-ico">
                ${ic('bell', 20)}
                ${unread > 0 ? `<span class="ps-badge-count">${unread}</span>` : ''}
              </span>
              <span class="ps-quick-label">通知</span>
            </button>
          </div>
        </section>

        <!-- 组 1：模型配置 / 我的设备 -->
        <section class="ps-slab">
          <button type="button" class="ps-menu-row" data-action="toast" data-msg="打开模型配置（小伴 AI / 第三方服务商）">
            <span class="ps-menu-ico">${ic('cpu', 19)}</span>
            <span class="ps-menu-title">模型配置</span>
            <span class="ps-menu-val">小伴 AI</span>
            <span class="ps-chev">${ic('chev', 15)}</span>
          </button>
          <button type="button" class="ps-menu-row" data-action="toast" data-msg="查看我的设备（桌面 / 手机互联）">
            <span class="ps-menu-ico">${ic('monitor', 19)}</span>
            <span class="ps-menu-title">我的设备</span>
            <span class="ps-menu-val">暂未连接</span>
            <span class="ps-chev">${ic('chev', 15)}</span>
          </button>
        </section>

        <!-- 组 2：帮助与反馈 / 交流群 -->
        <section class="ps-slab">
          <button type="button" class="ps-menu-row" data-action="toast" data-msg="打开帮助与反馈">
            <span class="ps-menu-ico">${ic('help', 19)}</span>
            <span class="ps-menu-title">帮助与反馈</span>
            <span class="ps-chev">${ic('chev', 15)}</span>
          </button>
          <button type="button" class="ps-menu-row" data-action="toast" data-msg="查看 QQ 群 / 微信交流群">
            <span class="ps-menu-ico">${ic('chatGroup', 19)}</span>
            <span class="ps-menu-title">交流群</span>
            <span class="ps-chev">${ic('chev', 15)}</span>
          </button>
        </section>
      </div>
    `;
  }

  /* ==================== 02 · 「个人主页」（本人 / 他人 / 空态） ==================== */
  function renderProfilePage(device) {
    const isOther = ps.profileView === 'other';
    const isEmpty = ps.profileView === 'empty';
    const name = isOther ? '林深见鹿' : 'read';
    const uid = isOther ? 'ID 102819' : 'ID 100482';
    const level = isOther ? '核心成员' : '活跃成员';
    const avatar = isOther ? AVATAR_OTHER : AVATAR_SELF;
    const counts = isEmpty ? [0, 0, 0, 0] : isOther ? [24, 61, 18, 132] : [3, 2, 3, 28];

    let tabBody = '';
    if (isOther && ps.otherBlocked) {
      tabBody = `
        <div class="ps-empty-box">
          <div class="ps-empty-ico">${ic('block', 24)}</div>
          <strong>已屏蔽该用户</strong>
          <p>你已屏蔽「${name}」，其发布的帖子和回复将不再向你展示。</p>
          <button type="button" class="ps-pill-act is-secondary" style="margin-top:10px" data-action="ps-unblock">解除屏蔽</button>
        </div>
      `;
    } else if (isEmpty) {
      tabBody = `
        <div class="ps-empty-box">
          <div class="ps-empty-ico">${ic('folder', 24)}</div>
          <strong>这里还没有内容</strong>
          <p>发布第一篇帖子或作品后，会在这里展示你的创作足迹。</p>
          <button type="button" class="ps-cta-pill" data-action="open-publish">${ic('plus', 16)} 发布新内容</button>
        </div>
      `;
    } else if (ps.profileTab === 'topics') {
      tabBody = ps.topics.map(t => `
        <article class="ps-topic-item" data-action="open-topic" data-id="${t.id}">
          <div class="ps-topic-main">
            <div class="ps-topic-copy">
              <div class="ps-topic-title">${esc(t.title)}</div>
              <div class="ps-topic-excerpt">${esc(t.excerpt)}</div>
            </div>
            ${t.thumb ? `<img class="ps-topic-thumb" src="${t.thumb}" alt="" loading="lazy">` : ''}
          </div>
          <div class="ps-topic-foot">
            <span style="display:inline-flex;align-items:center;gap:8px">
              <span class="ps-cat-chip">${esc(t.cat)}</span>
              <span>${esc(t.time)}</span>
            </span>
            <span>${t.replies} 回复 · ${t.likes} 赞</span>
          </div>
        </article>
      `).join('');
    } else if (ps.profileTab === 'replies') {
      tabBody = ps.replies.map(r => `
        <div class="ps-topic-item">
          <div style="font-size:12px;color:var(--ink-3);margin-bottom:4px">回复了「${esc(r.topic)}」</div>
          <div class="ps-topic-excerpt" style="color:var(--ink-1);margin-top:0">${esc(r.excerpt)}</div>
          <div class="ps-topic-foot"><span>${esc(r.time)}</span></div>
        </div>
      `).join('');
    } else {
      const list = ps.profileTab === 'following' ? ps.people.filter(p => p.followed) : ps.people;
      tabBody = list.map(p => `
        <div class="ps-person-row">
          <span class="ps-person-av" style="background:${p.color}">${esc(p.init)}</span>
          <div class="ps-person-info">
            <strong>${esc(p.name)}</strong>
            <span>${esc(p.note)}</span>
          </div>
          <button type="button" class="ps-pill-act ${p.followed ? 'is-secondary' : 'is-primary'}" style="height:30px;padding:0 12px;font-size:12.5px" data-action="ps-toggle-person" data-uid="${p.uid}">
            ${p.followed ? '已关注' : '关注'}
          </button>
        </div>
      `).join('');
    }

    return `
      <div class="ps-page">
        <div class="ps-demo-strip">
          <div class="ps-demo-title">
            ${device === 'desktop' ? `<button type="button" class="ps-back-link" data-action="go-page" data-page="me">${ic('back', 14)} 我的</button>` : ''}
            <span>个人主页</span>
          </div>
          <div class="ps-seg" role="group" aria-label="主页视角">
            <button type="button" class="${ps.profileView === 'self' ? 'is-active' : ''}" data-action="ps-profile-view" data-val="self">本人视角</button>
            <button type="button" class="${ps.profileView === 'other' ? 'is-active' : ''}" data-action="ps-profile-view" data-val="other">他人视角</button>
            <button type="button" class="${ps.profileView === 'empty' ? 'is-active' : ''}" data-action="ps-profile-view" data-val="empty">空态</button>
          </div>
        </div>

        <section class="ps-slab">
          <div class="ps-profile-row">
            <div class="ps-avatar-btn is-lg"><img src="${avatar}" alt="${name}"></div>
            <div class="ps-profile-info">
              <div class="ps-name-row">
                <span class="ps-name">${name}</span>
                <span class="ps-level-tag">${level}</span>
              </div>
              <button type="button" class="ps-id-copy" data-action="ps-copy-id">
                <span>${uid}</span>${ic('copy', 12)}
              </button>
            </div>
            ${isOther ? `
              <div style="display:flex;align-items:center;gap:6px;flex:none">
                ${!ps.otherBlocked ? `
                  <button type="button" class="ps-pill-act ${ps.otherFollowing ? 'is-secondary' : 'is-primary'}" data-action="ps-follow-other">
                    ${ps.otherFollowing ? '已关注' : '关注'}
                  </button>
                ` : ''}
                <div class="ps-more-wrap">
                  <button type="button" class="ps-work-more" aria-label="更多" data-action="ps-open-profile-more" data-device="${device}">${ic('more', 18)}</button>
                  ${device === 'desktop' && ps.profileMoreOpen ? `
                    <div class="ps-menu-pop">
                      <button type="button" data-action="toast" data-msg="已复制主页链接">${ic('link', 15)} 复制主页链接</button>
                      <button type="button" class="is-danger" data-action="ps-ask-block" data-device="desktop">
                        ${ic('block', 15)} ${ps.otherBlocked ? '解除屏蔽' : '屏蔽该用户'}
                      </button>
                    </div>
                  ` : ''}
                </div>
              </div>
            ` : `
              <button type="button" class="ps-pill-act is-secondary" data-action="toast" data-msg="打开更换头像 / 编辑资料">换头像</button>
            `}
          </div>

          <div class="ps-slab-divider"></div>

          <div class="ps-stats-bar">
            <button type="button" class="ps-stat-cell" data-action="ps-profile-tab" data-val="topics"><span class="ps-stat-num">${counts[0]}</span><span class="ps-stat-lbl">帖子</span></button>
            <button type="button" class="ps-stat-cell" data-action="ps-profile-tab" data-val="replies"><span class="ps-stat-num">${counts[1]}</span><span class="ps-stat-lbl">回复</span></button>
            <button type="button" class="ps-stat-cell" data-action="ps-profile-tab" data-val="following"><span class="ps-stat-num">${counts[2]}</span><span class="ps-stat-lbl">关注</span></button>
            <button type="button" class="ps-stat-cell" data-action="ps-profile-tab" data-val="followers"><span class="ps-stat-num">${counts[3]}</span><span class="ps-stat-lbl">粉丝</span></button>
          </div>
        </section>

        <section class="ps-slab">
          <div class="ps-tabs-bar">
            <div class="ps-tabs-scroll" role="tablist">
              ${[
                ['topics', '帖子', counts[0]],
                ['replies', '回复', counts[1]],
                ['following', '关注', counts[2]],
                ['followers', '粉丝', counts[3]],
              ].map(([k, lbl, c]) => `
                <button type="button" class="ps-tab-btn ${ps.profileTab === k ? 'is-active' : ''}" data-action="ps-profile-tab" data-val="${k}">
                  <span>${lbl}</span><span class="cnt">${c}</span>
                </button>
              `).join('')}
            </div>
          </div>
          <div>${tabBody}</div>
        </section>
      </div>
    `;
  }

  /* ==================== 03 · 「我的作品」（通栏行 + ··· 菜单，零按钮堆砌） ==================== */
  function renderWorksPage(device) {
    const source = ps.worksKind === 'packs' ? ps.works : ps.schemes;
    const counts = {
      all: source.length,
      draft: source.filter(w => w.status === 'draft').length,
      live: source.filter(w => w.status === 'live').length,
      review: source.filter(w => w.status === 'review').length,
      removed: source.filter(w => w.status === 'removed').length,
    };
    const filtered = ps.worksEmpty ? [] : source.filter(w => ps.worksFilter === 'all' || w.status === ps.worksFilter);

    return `
      <div class="ps-page">
        <div class="ps-demo-strip">
          <div class="ps-demo-title">
            ${device === 'desktop' ? `<button type="button" class="ps-back-link" data-action="go-page" data-page="me">${ic('back', 14)} 我的</button>` : ''}
            <span>我的作品</span>
          </div>
          <div style="display:flex;gap:6px;flex-wrap:wrap">
            <div class="ps-seg">
              <button type="button" class="${ps.worksKind === 'packs' ? 'is-active' : ''}" data-action="ps-works-kind" data-val="packs">内容包</button>
              <button type="button" class="${ps.worksKind === 'schemes' ? 'is-active' : ''}" data-action="ps-works-kind" data-val="schemes">方案</button>
            </div>
            <div class="ps-seg">
              <button type="button" class="${!ps.worksEmpty ? 'is-active' : ''}" data-action="ps-works-empty" data-val="0">有作品</button>
              <button type="button" class="${ps.worksEmpty ? 'is-active' : ''}" data-action="ps-works-empty" data-val="1">空态</button>
            </div>
          </div>
        </div>

        <section class="ps-slab">
          <div class="ps-tabs-bar">
            <div class="ps-tabs-scroll" role="tablist" aria-label="作品状态筛选">
              ${[
                ['all', '全部'],
                ['draft', '草稿'],
                ['live', '已上线'],
                ['review', '审核中'],
                ['removed', '已下架'],
              ].map(([k, lbl]) => `
                <button type="button" class="ps-tab-btn ${ps.worksFilter === k ? 'is-active' : ''}" data-action="ps-works-filter" data-val="${k}">
                  <span>${lbl}</span><span class="cnt">${ps.worksEmpty ? 0 : counts[k]}</span>
                </button>
              `).join('')}
            </div>
          </div>

          ${filtered.length ? filtered.map(w => {
            const isDraft = w.status === 'draft';
            const isLiveOrReview = w.status === 'live' || w.status === 'review';
            const menuOpen = device === 'desktop' && ps.pcWorkMenuId === w.id;
            return `
              <div class="ps-work-row">
                <button type="button" class="ps-work-hit" data-action="toast" data-msg="${isDraft ? `试玩本机草稿「${esc(w.title)}」` : `打开作品详情「${esc(w.title)}」`}">
                  <span class="ps-work-icon ${isDraft ? 'is-draft' : ''}">${ic(w.kindKey === 'game' ? 'gamepad' : 'box', 21)}</span>
                  <div class="ps-work-body">
                    <div class="ps-work-head">
                      <span class="ps-work-title">${esc(w.title)}</span>
                      <span class="ps-status-chip ${isDraft ? 'is-draft' : `has-dot s-${w.status}`}">${esc(w.statusText)}</span>
                    </div>
                    <div class="ps-work-meta">
                      ${esc(w.kind)}${w.version ? ` · ${esc(w.version)}` : ` · ${esc(w.storage || '仅本机')}`} · ${esc(w.updated)}${w.uses ? ` · ${w.uses} 次使用` : ''}
                    </div>
                  </div>
                </button>
                <div class="ps-work-more-wrap">
                  <button type="button" class="ps-work-more" aria-label="作品操作" data-action="ps-open-work-menu" data-id="${w.id}" data-device="${device}">
                    ${ic('more', 18)}
                  </button>
                  ${menuOpen ? `
                    <div class="ps-menu-pop">
                      ${isDraft ? `
                        <button type="button" data-action="toast" data-msg="试玩草稿「${esc(w.title)}」">${ic('play', 15)} 试玩预览</button>
                        <button type="button" data-action="toast" data-msg="继续修改「${esc(w.title)}」">${ic('edit', 15)} 继续修改</button>
                        <button type="button" data-action="toast" data-msg="提交「${esc(w.title)}」到社区审核">${ic('upload', 15)} 发布到社区</button>
                        <button type="button" class="is-danger" data-action="ps-ask-work" data-act="delete" data-id="${w.id}" data-device="desktop">${ic('trash', 15)} 删除草稿</button>
                      ` : isLiveOrReview ? `
                        <button type="button" data-action="toast" data-msg="查看作品详情「${esc(w.title)}」">${ic('eye', 15)} 查看详情</button>
                        <button type="button" data-action="ps-ask-work" data-act="remove" data-id="${w.id}" data-device="desktop">${ic('eyeOff', 15)} 下架作品</button>
                        <button type="button" class="is-danger" data-action="ps-ask-work" data-act="delete" data-id="${w.id}" data-device="desktop">${ic('trash', 15)} 永久删除</button>
                      ` : `
                        <button type="button" data-action="toast" data-msg="重新提交「${esc(w.title)}」审核">${ic('upload', 15)} 重新提交</button>
                        <button type="button" class="is-danger" data-action="ps-ask-work" data-act="delete" data-id="${w.id}" data-device="desktop">${ic('trash', 15)} 永久删除</button>
                      `}
                    </div>
                  ` : ''}
                </div>
              </div>
            `;
          }).join('') : `
            <div class="ps-empty-box">
              <div class="ps-empty-ico">${ic('folder', 24)}</div>
              <strong>还没有作品</strong>
              <p>用小伴创作的小游戏、小程序或桌面方案，草稿与已发布内容都会集中在这里管理。</p>
              <button type="button" class="ps-cta-pill" data-action="open-publish">${ic('plus', 16)} 去创作发布</button>
            </div>
          `}
        </section>
      </div>
    `;
  }

  /* ==================== 04 · 「通知」（顶栏右上角全部已读 + 40px 头像角标红点） ==================== */
  function renderNotificationsPage(device) {
    const unread = unreadCount();
    const list = ps.notifEmpty ? [] : ps.notifs.filter(n => ps.notifTab === 'all' || n.unread);
    return `
      <div class="ps-page">
        <div class="ps-demo-strip">
          <div class="ps-demo-title">
            ${device === 'desktop' ? `<button type="button" class="ps-back-link" data-action="go-page" data-page="me">${ic('back', 14)} 我的</button>` : ''}
            <span>通知</span>
          </div>
          <div class="ps-seg">
            <button type="button" class="${!ps.notifEmpty ? 'is-active' : ''}" data-action="ps-notif-empty" data-val="0">有通知</button>
            <button type="button" class="${ps.notifEmpty ? 'is-active' : ''}" data-action="ps-notif-empty" data-val="1">空态</button>
          </div>
        </div>

        <section class="ps-slab">
          <div class="ps-tabs-bar">
            <div class="ps-tabs-scroll" role="tablist">
              <button type="button" class="ps-tab-btn ${ps.notifTab === 'all' ? 'is-active' : ''}" data-action="ps-notif-tab" data-val="all">全部</button>
              <button type="button" class="ps-tab-btn ${ps.notifTab === 'unread' ? 'is-active' : ''}" data-action="ps-notif-tab" data-val="unread">
                <span>未读</span>${unread > 0 ? `<span class="cnt">${unread}</span>` : ''}
              </button>
            </div>
            <button type="button" class="ps-bar-text-btn" data-action="ps-read-all" ${unread === 0 ? 'disabled' : ''}>全部已读</button>
          </div>

          ${list.length ? list.map(n => `
            <button type="button" class="ps-notif-row ${n.unread ? 'is-unread' : ''}" data-action="ps-read-one" data-id="${n.id}">
              <span class="ps-notif-av-wrap">
                <span class="ps-notif-av ${n.sys ? 'is-sys' : ''}" style="${n.sys ? '' : `background:${n.color}`}">
                  ${n.sys ? ic('check', 18) : esc(n.init)}
                </span>
                ${n.unread ? '<span class="ps-notif-dot"></span>' : ''}
              </span>
              <div class="ps-notif-body">
                <div class="ps-notif-headline">
                  ${n.actor ? `<strong>${esc(n.actor)}</strong> ` : ''}${esc(n.text)}
                </div>
                ${n.quote ? `<div class="ps-notif-quote">${esc(n.quote)}</div>` : ''}
                <div class="ps-notif-foot">${esc(n.tag)} · ${esc(n.time)}</div>
              </div>
              <span class="ps-chev" style="margin-top:10px">${ic('chev', 15)}</span>
            </button>
          `).join('') : `
            <div class="ps-empty-box">
              <div class="ps-empty-ico">${ic('bell', 24)}</div>
              <strong>${ps.notifTab === 'unread' ? '没有未读通知' : '还没有收到通知'}</strong>
              <p>有人评论、点赞或审核你的作品时，会第一时间出现在这里。</p>
            </div>
          `}
        </section>
      </div>
    `;
  }

  /* ==================== 底部抽屉（H5） & 24px 居中二次确认弹窗 ==================== */
  function renderProfileModals(device) {
    let html = '';

    // H5 底部操作抽屉（我的作品 / 他人主页更多）
    if (device === 'mobile' && ps.h5WorkSheet) {
      const list = ps.h5WorkSheet.kind === 'packs' ? ps.works : ps.schemes;
      const w = list.find(x => x.id === ps.h5WorkSheet.workId);
      if (w) {
        const isDraft = w.status === 'draft';
        const isLiveOrReview = w.status === 'live' || w.status === 'review';
        html += `
          <div class="ps-sheet-mask" data-action="ps-close-sheet">
            <div class="ps-sheet" role="dialog" aria-modal="true">
              <div class="ps-sheet-handle"></div>
              <div class="ps-sheet-head">
                <h4>${esc(w.title)}</h4>
                <p>${esc(w.kind)} · ${esc(w.statusText)}</p>
              </div>
              ${isDraft ? `
                <button type="button" class="ps-sheet-btn" data-action="ps-sheet-toast" data-msg="试玩草稿「${esc(w.title)}」">${ic('play', 19)}<span class="ps-sheet-btn-copy"><strong>试玩预览</strong><small>在本地直接运行当前草稿</small></span></button>
                <button type="button" class="ps-sheet-btn" data-action="ps-sheet-toast" data-msg="继续修改「${esc(w.title)}」">${ic('edit', 19)}<span class="ps-sheet-btn-copy"><strong>继续修改</strong><small>回到对话继续完善作品</small></span></button>
                <button type="button" class="ps-sheet-btn" data-action="ps-sheet-toast" data-msg="提交「${esc(w.title)}」到社区审核">${ic('upload', 19)}<span class="ps-sheet-btn-copy"><strong>发布到社区</strong><small>提交审核后公开给其他用户</small></span></button>
                <button type="button" class="ps-sheet-btn is-danger" data-action="ps-ask-work" data-act="delete" data-id="${w.id}" data-device="mobile">${ic('trash', 19)}<span class="ps-sheet-btn-copy"><strong>删除草稿</strong><small>永久移除本机草稿，无法恢复</small></span></button>
              ` : isLiveOrReview ? `
                <button type="button" class="ps-sheet-btn" data-action="ps-sheet-toast" data-msg="查看作品详情「${esc(w.title)}」">${ic('eye', 19)}<span class="ps-sheet-btn-copy"><strong>查看详情</strong></span></button>
                <button type="button" class="ps-sheet-btn" data-action="ps-ask-work" data-act="remove" data-id="${w.id}" data-device="mobile">${ic('eyeOff', 19)}<span class="ps-sheet-btn-copy"><strong>下架作品</strong><small>暂不公开，后续可重新提交</small></span></button>
                <button type="button" class="ps-sheet-btn is-danger" data-action="ps-ask-work" data-act="delete" data-id="${w.id}" data-device="mobile">${ic('trash', 19)}<span class="ps-sheet-btn-copy"><strong>永久删除</strong><small>彻底删除作品与记录，无法恢复</small></span></button>
              ` : `
                <button type="button" class="ps-sheet-btn" data-action="ps-sheet-toast" data-msg="重新提交「${esc(w.title)}」">${ic('upload', 19)}<span class="ps-sheet-btn-copy"><strong>重新提交审核</strong></span></button>
                <button type="button" class="ps-sheet-btn is-danger" data-action="ps-ask-work" data-act="delete" data-id="${w.id}" data-device="mobile">${ic('trash', 19)}<span class="ps-sheet-btn-copy"><strong>永久删除</strong><small>彻底删除记录，无法恢复</small></span></button>
              `}
              <button type="button" class="ps-sheet-cancel" data-action="ps-close-sheet">取消</button>
            </div>
          </div>
        `;
      }
    }

    if (device === 'mobile' && ps.h5ProfileSheet) {
      html += `
        <div class="ps-sheet-mask" data-action="ps-close-sheet">
          <div class="ps-sheet" role="dialog" aria-modal="true">
            <div class="ps-sheet-handle"></div>
            <button type="button" class="ps-sheet-btn" data-action="ps-sheet-toast" data-msg="已复制主页链接">${ic('link', 19)}<span>复制主页链接</span></button>
            <button type="button" class="ps-sheet-btn is-danger" data-action="ps-ask-block" data-device="mobile">${ic('block', 19)}<span>${ps.otherBlocked ? '解除屏蔽' : '屏蔽该用户'}</span></button>
            <button type="button" class="ps-sheet-cancel" data-action="ps-close-sheet">取消</button>
          </div>
        </div>
      `;
    }

    // 居中 24px 二次确认弹窗
    if (ps.modal && ps.modal.device === device) {
      if (ps.modal.action === 'block') {
        html += `
          <div class="ps-modal-backdrop" data-action="ps-close-modal">
            <div class="ps-modal-card" role="dialog" aria-modal="true">
              <h3>屏蔽「林深见鹿」？</h3>
              <p>屏蔽后将不再向你展示对方发布的帖子、评论与作品，你可以随时在对方主页解除屏蔽。</p>
              <div class="ps-modal-actions">
                <button type="button" class="ps-modal-btn is-danger" data-action="ps-confirm-modal">确认屏蔽</button>
                <button type="button" class="ps-modal-btn is-cancel" data-action="ps-close-modal">取消</button>
              </div>
            </div>
          </div>
        `;
      } else {
        const source = ps.modal.kind === 'packs' ? ps.works : ps.schemes;
        const item = source.find(w => w.id === ps.modal.workId);
        if (item) {
          const isDelete = ps.modal.action === 'delete';
          html += `
            <div class="ps-modal-backdrop" data-action="ps-close-modal">
              <div class="ps-modal-card" role="dialog" aria-modal="true">
                <h3>${isDelete ? `永久删除「${esc(item.title)}」？` : `下架「${esc(item.title)}」？`}</h3>
                <p>${isDelete ? '删除后无法恢复，本机草稿或已发布记录将被彻底移除。' : '下架后社区其他用户将无法访问该作品，你仍可在「已下架」中查看或永久删除。'}</p>
                <div class="ps-modal-actions">
                  <button type="button" class="ps-modal-btn is-danger" data-action="ps-confirm-modal">${isDelete ? '永久删除' : '确认下架'}</button>
                  <button type="button" class="ps-modal-btn is-cancel" data-action="ps-close-modal">取消</button>
                </div>
              </div>
            </div>
          `;
        }
      }
    }

    return html;
  }

  /* ==================== 详情页分享面板（PC + H5） ==================== */
  function renderShareCardHtml(item) {
    const shareUrl = `https://chat.xiangban.xyz/${item.key.replace(':', '/')}`;
    return `
      <div class="ds-card theme-${ds.cardTheme}">
        <div class="ds-card-brand">
          <span class="ds-card-brand-l">
            <span class="ds-brand-dot">${ic('spark', 11)}</span>
            <span>超级小伴社区</span>
          </span>
          <span class="ds-card-kind">${esc(item.badge || '社区内容')}</span>
        </div>
        ${item.cover ? `<div class="ds-card-cover"><img src="${item.cover}" alt=""></div>` : ''}
        <div class="ds-card-title">${esc(item.title)}</div>
        ${ds.showExcerpt ? `<div class="ds-card-excerpt">${esc((item.body || '').replace(/\n+/g, ' '))}</div>` : ''}
        <div class="ds-card-author">
          <span class="mini-avatar" style="background:${item.avatarBg || '#2f6fdb'};width:20px;height:20px;font-size:10.5px">${esc((item.author || '伴')[0].replace('@', '').toUpperCase())}</span>
          <strong>${esc(item.author)}</strong>
          <span>· ${esc(item.date || '刚刚')}</span>
        </div>
        ${ds.showQr ? `
          <div class="ds-card-foot">
            <div class="ds-card-foot-copy">
              <strong>扫码查看完整内容</strong>
              <span>${esc(shareUrl)}</span>
            </div>
            <div class="ds-qr">${QR_SVG}</div>
          </div>
        ` : ''}
      </div>
    `;
  }

  function renderShareOverlay(device, item, role) {
    if (!item || !ds.openDevice) return '';
    if (ds.openDevice !== 'both' && ds.openDevice !== device) return '';
    const canDelete = role === 'author' || role === 'admin';
    const isProject = item.key.startsWith('project:');
    const extraLabel = isProject ? '复制仓库地址' : item.actionLabel ? '新标签页打开体验' : '复制全文内容';
    const extraSub = isProject ? '复制 GitHub 仓库链接' : item.actionLabel ? '全屏进入体验页面' : '复制标题与完整正文';

    if (device === 'desktop') {
      return `
        <div class="ds-backdrop" data-action="ds-close">
          <div class="ds-pc-dialog" role="dialog" aria-modal="true" aria-label="分享面板">
            <div class="ds-pc-left">
              ${renderShareCardHtml(item)}
              <div class="ds-customize">
                <button type="button" class="ds-tone-dot t-white ${ds.cardTheme === 'white' ? 'is-active' : ''}" data-action="ds-tone" data-val="white" title="极简白"></button>
                <button type="button" class="ds-tone-dot t-warm ${ds.cardTheme === 'warm' ? 'is-active' : ''}" data-action="ds-tone" data-val="warm" title="暖纸笺"></button>
                <button type="button" class="ds-tone-dot t-ink ${ds.cardTheme === 'ink' ? 'is-active' : ''}" data-action="ds-tone" data-val="ink" title="墨夜"></button>
                <button type="button" class="ds-toggle-pill ${ds.showExcerpt ? 'is-on' : ''}" data-action="ds-toggle-excerpt">摘录</button>
                <button type="button" class="ds-toggle-pill ${ds.showQr ? 'is-on' : ''}" data-action="ds-toggle-qr">二维码</button>
              </div>
            </div>
            <div class="ds-pc-right">
              <div class="ds-pc-head">
                <strong>分享内容</strong>
                <button type="button" class="ds-close-btn" data-action="ds-close" aria-label="关闭">×</button>
              </div>
              <div class="ds-act-list">
                <button type="button" class="ds-act-btn is-primary" data-action="ds-act" data-key="link">
                  <span class="ds-act-icon">${ic(ds.copiedKey === 'link' ? 'check' : 'link', 15)}</span>
                  <span class="ds-act-copy">
                    <strong>${ds.copiedKey === 'link' ? '已复制分享链接' : '复制链接'}</strong>
                    <span class="ds-act-sub">未安装小伴也能在浏览器直接打开</span>
                  </span>
                </button>
                <button type="button" class="ds-act-btn" data-action="ds-act" data-key="copy-img">
                  <span class="ds-act-icon">${ic(ds.copiedKey === 'copy-img' ? 'check' : 'copyImg', 15)}</span>
                  <span class="ds-act-copy">
                    <strong>${ds.copiedKey === 'copy-img' ? '已复制卡片图片' : '复制卡片图片'}</strong>
                    <span class="ds-act-sub">直接粘贴发送进微信、飞书或 QQ</span>
                  </span>
                </button>
                <button type="button" class="ds-act-btn" data-action="ds-act" data-key="save-png">
                  <span class="ds-act-icon">${ic('download', 15)}</span>
                  <span class="ds-act-copy">
                    <strong>保存 PNG 图片</strong>
                    <span class="ds-act-sub">导出高清分享卡片到本地</span>
                  </span>
                </button>
                <button type="button" class="ds-act-btn" data-action="ds-act" data-key="quote">
                  <span class="ds-act-icon">${ic(ds.copiedKey === 'quote' ? 'check' : 'quote', 15)}</span>
                  <span class="ds-act-copy">
                    <strong>${ds.copiedKey === 'quote' ? '已复制推荐语' : '复制推荐语'}</strong>
                    <span class="ds-act-sub">带标题、摘要与链接的纯文本</span>
                  </span>
                </button>
                <button type="button" class="ds-act-btn" data-action="ds-act" data-key="extra">
                  <span class="ds-act-icon">${ic('ext', 15)}</span>
                  <span class="ds-act-copy">
                    <strong>${extraLabel}</strong>
                    <span class="ds-act-sub">${extraSub}</span>
                  </span>
                </button>
                <button type="button" class="ds-act-btn" data-action="ds-act" data-key="ask-ai">
                  <span class="ds-act-icon">${ic('spark', 15)}</span>
                  <span class="ds-act-copy">
                    <strong>问问小伴</strong>
                    <span class="ds-act-sub">带着这篇内容直接向小伴 AI 提问</span>
                  </span>
                </button>
              </div>
              ${canDelete ? `
                <div class="ds-del-row">
                  ${ds.confirmDelete ? `
                    <div class="ds-confirm-strip">
                      <span>确定删除当前内容？</span>
                      <div class="ds-confirm-btns">
                        <button type="button" class="c-no" data-action="ds-cancel-del">取消</button>
                        <button type="button" class="c-yes" data-action="ds-confirm-del">确认删除</button>
                      </div>
                    </div>
                  ` : `
                    <button type="button" class="ds-del-trigger" data-action="ds-ask-del">
                      ${ic('trash', 15)} <span>删除当前内容</span>
                    </button>
                  `}
                </div>
              ` : ''}
            </div>
          </div>
        </div>
      `;
    }

    if (ds.h5Mode === 'card') {
      return `
        <div class="ds-h5-stage" role="dialog" aria-modal="true" aria-label="分享卡片">
          <div class="ds-h5-stage-top">
            <button type="button" class="ds-h5-stage-btn" data-action="ds-h5-back" aria-label="返回">${ic('back', 18)}</button>
            <strong>分享卡片</strong>
            <button type="button" class="ds-h5-stage-btn" data-action="ds-close" aria-label="关闭">${ic('x', 16)}</button>
          </div>
          <div class="ds-h5-stage-body">
            ${renderShareCardHtml(item)}
            <div class="ds-customize">
              <button type="button" class="ds-tone-dot t-white ${ds.cardTheme === 'white' ? 'is-active' : ''}" data-action="ds-tone" data-val="white" title="极简白"></button>
              <button type="button" class="ds-tone-dot t-warm ${ds.cardTheme === 'warm' ? 'is-active' : ''}" data-action="ds-tone" data-val="warm" title="暖纸笺"></button>
              <button type="button" class="ds-tone-dot t-ink ${ds.cardTheme === 'ink' ? 'is-active' : ''}" data-action="ds-tone" data-val="ink" title="墨夜"></button>
              <button type="button" class="ds-toggle-pill ${ds.showExcerpt ? 'is-on' : ''}" data-action="ds-toggle-excerpt">摘录</button>
              <button type="button" class="ds-toggle-pill ${ds.showQr ? 'is-on' : ''}" data-action="ds-toggle-qr">二维码</button>
            </div>
          </div>
          <div class="ds-h5-stage-dock">
            <button type="button" class="ds-h5-stage-act" data-action="ds-act" data-key="save-png">
              <span class="ic">${ic('download', 19)}</span><span>保存图片</span>
            </button>
            <button type="button" class="ds-h5-stage-act" data-action="ds-act" data-key="send-friend">
              <span class="ic">${ic('send', 18)}</span><span>发送给好友</span>
            </button>
            <button type="button" class="ds-h5-stage-act" data-action="ds-act" data-key="copy-img">
              <span class="ic">${ic('copyImg', 18)}</span><span>复制图片</span>
            </button>
          </div>
        </div>
      `;
    }

    return `
      <div class="ds-h5-scrim" data-action="ds-close">
        <div class="ds-h5-sheet" role="dialog" aria-modal="true" aria-label="分享面板">
          <div class="ds-h5-preview">
            ${item.cover ? `<img class="ds-h5-thumb" src="${item.cover}" alt="">` : `<span class="ds-h5-thumb">${ic('card', 18)}</span>`}
            <div class="ds-h5-preview-txt">
              <strong>${esc(item.title)}</strong>
              <span>${esc(item.badge || '社区内容')} · ${esc(item.author)}</span>
            </div>
          </div>
          ${ds.confirmDelete ? `
            <div class="ds-confirm-strip">
              <span>确定删除这条内容？</span>
              <div class="ds-confirm-btns">
                <button type="button" class="c-no" data-action="ds-cancel-del">取消</button>
                <button type="button" class="c-yes" data-action="ds-confirm-del">确认删除</button>
              </div>
            </div>
          ` : `
            <div class="ds-h5-rail">
              <button type="button" class="ds-h5-item" data-action="ds-act" data-key="link">
                <span class="ds-h5-tile">${ic(ds.copiedKey === 'link' ? 'check' : 'link', 20)}</span>
                <span>${ds.copiedKey === 'link' ? '已复制' : '复制链接'}</span>
              </button>
              <button type="button" class="ds-h5-item" data-action="ds-h5-card">
                <span class="ds-h5-tile">${ic('card', 20)}</span>
                <span>分享卡片</span>
              </button>
              <button type="button" class="ds-h5-item" data-action="ds-act" data-key="ask-ai">
                <span class="ds-h5-tile">${ic('spark', 20)}</span>
                <span>问问小伴</span>
              </button>
              <button type="button" class="ds-h5-item" data-action="ds-act" data-key="extra">
                <span class="ds-h5-tile">${ic('ext', 19)}</span>
                <span>${isProject ? '复制仓库' : '浏览器打开'}</span>
              </button>
              ${canDelete ? `
                <button type="button" class="ds-h5-item is-danger" data-action="ds-ask-del">
                  <span class="ds-h5-tile">${ic('trash', 19)}</span>
                  <span>删除</span>
                </button>
              ` : ''}
            </div>
          `}
          <button type="button" class="ds-h5-cancel" data-action="ds-close">取消</button>
        </div>
      </div>
    `;
  }

  function handleAction(action, btn, event, ctx) {
    if (action === 'ps-login') {
      ps.loggedIn = btn.getAttribute('data-val') === '1';
      ctx.render(false);
      return true;
    }
    if (action === 'ps-copy-id') {
      ctx.showToast('已复制账号 ID：100482');
      return true;
    }
    if (action === 'ps-checkin') {
      if (!ps.checkedIn) {
        ps.checkedIn = true;
        ctx.showToast('今日已签到');
        ctx.render(false);
      }
      return true;
    }
    if (action === 'ps-profile-view') {
      ps.profileView = btn.getAttribute('data-val');
      ps.profileMoreOpen = false;
      ps.h5ProfileSheet = false;
      ctx.render(false);
      return true;
    }
    if (action === 'ps-profile-tab') {
      ps.profileTab = btn.getAttribute('data-val');
      ctx.render(false);
      return true;
    }
    if (action === 'ps-follow-other') {
      ps.otherFollowing = !ps.otherFollowing;
      ctx.showToast(ps.otherFollowing ? '已关注林深见鹿' : '已取消关注');
      ctx.render(false);
      return true;
    }
    if (action === 'ps-open-profile-more') {
      const dev = btn.getAttribute('data-device');
      if (dev === 'mobile') {
        ps.h5ProfileSheet = true;
      } else {
        ps.profileMoreOpen = !ps.profileMoreOpen;
      }
      ctx.render(false);
      return true;
    }
    if (action === 'ps-open-work-menu') {
      const id = Number(btn.getAttribute('data-id'));
      const dev = btn.getAttribute('data-device');
      if (dev === 'mobile') {
        ps.h5WorkSheet = { kind: ps.worksKind, workId: id };
      } else {
        ps.pcWorkMenuId = ps.pcWorkMenuId === id ? null : id;
      }
      ctx.render(false);
      return true;
    }
    if (action === 'ps-close-sheet') {
      if (btn.classList.contains('ps-sheet-mask') && event.target !== btn) return true;
      ps.h5WorkSheet = null;
      ps.h5ProfileSheet = false;
      ctx.render(false);
      return true;
    }
    if (action === 'ps-sheet-toast') {
      ps.h5WorkSheet = null;
      ps.h5ProfileSheet = false;
      ctx.render(false);
      ctx.showToast(btn.getAttribute('data-msg') || '演示操作');
      return true;
    }
    if (action === 'ps-ask-block') {
      ps.profileMoreOpen = false;
      ps.h5ProfileSheet = false;
      if (ps.otherBlocked) {
        ps.otherBlocked = false;
        ctx.showToast('已解除屏蔽');
      } else {
        ps.modal = { device: btn.getAttribute('data-device') || 'desktop', action: 'block' };
      }
      ctx.render(false);
      return true;
    }
    if (action === 'ps-unblock') {
      ps.otherBlocked = false;
      ctx.showToast('已解除屏蔽');
      ctx.render(false);
      return true;
    }
    if (action === 'ps-toggle-person') {
      const uid = btn.getAttribute('data-uid');
      const p = ps.people.find(x => x.uid === uid);
      if (p) {
        p.followed = !p.followed;
        ctx.showToast(p.followed ? `已关注 ${p.name}` : `已取消关注 ${p.name}`);
        ctx.render(false);
      }
      return true;
    }
    if (action === 'ps-works-kind') {
      ps.worksKind = btn.getAttribute('data-val');
      ps.worksFilter = 'all';
      ps.pcWorkMenuId = null;
      ctx.render(false);
      return true;
    }
    if (action === 'ps-works-empty') {
      ps.worksEmpty = btn.getAttribute('data-val') === '1';
      ps.pcWorkMenuId = null;
      ctx.render(false);
      return true;
    }
    if (action === 'ps-works-filter') {
      ps.worksFilter = btn.getAttribute('data-val');
      ps.pcWorkMenuId = null;
      ctx.render(false);
      return true;
    }
    if (action === 'ps-ask-work') {
      ps.pcWorkMenuId = null;
      ps.h5WorkSheet = null;
      ps.modal = {
        device: btn.getAttribute('data-device') || 'desktop',
        action: btn.getAttribute('data-act'),
        kind: ps.worksKind,
        workId: Number(btn.getAttribute('data-id')),
      };
      ctx.render(false);
      return true;
    }
    if (action === 'ps-close-modal') {
      if (btn.classList.contains('ps-modal-backdrop') && event.target !== btn) return true;
      ps.modal = null;
      ctx.render(false);
      return true;
    }
    if (action === 'ps-confirm-modal') {
      if (!ps.modal) return true;
      if (ps.modal.action === 'block') {
        ps.otherBlocked = true;
        ps.otherFollowing = false;
        ctx.showToast('已屏蔽该用户');
      } else {
        const list = ps.modal.kind === 'packs' ? ps.works : ps.schemes;
        const idx = list.findIndex(w => w.id === ps.modal.workId);
        if (idx >= 0) {
          if (ps.modal.action === 'remove') {
            list[idx].status = 'removed';
            list[idx].statusText = '已下架';
            ctx.showToast(`已下架「${list[idx].title}」`);
          } else {
            const title = list[idx].title;
            list.splice(idx, 1);
            ctx.showToast(`已永久删除「${title}」`);
          }
        }
      }
      ps.modal = null;
      ctx.render(false);
      return true;
    }
    if (action === 'ps-notif-empty') {
      ps.notifEmpty = btn.getAttribute('data-val') === '1';
      ctx.render(false);
      return true;
    }
    if (action === 'ps-notif-tab') {
      ps.notifTab = btn.getAttribute('data-val');
      ctx.render(false);
      return true;
    }
    if (action === 'ps-read-all') {
      ps.notifs.forEach(n => { n.unread = false; });
      ctx.showToast('已全部标为已读');
      ctx.render(false);
      return true;
    }
    if (action === 'ps-read-one') {
      const id = Number(btn.getAttribute('data-id'));
      const n = ps.notifs.find(x => x.id === id);
      if (n && n.unread) {
        n.unread = false;
        ctx.render(false);
      }
      return true;
    }

    // Detail Share Actions
    if (action === 'open-share') {
      ds.openDevice = btn.getAttribute('data-device') || 'desktop';
      ds.h5Mode = 'sheet';
      ds.confirmDelete = false;
      ds.copiedKey = '';
      ctx.closeDetailMenus();
      ctx.render(false);
      return true;
    }
    if (action === 'ds-close') {
      if ((btn.classList.contains('ds-backdrop') || btn.classList.contains('ds-h5-scrim')) && event.target !== btn) return true;
      ds.openDevice = null;
      ds.confirmDelete = false;
      ctx.render(false);
      return true;
    }
    if (action === 'ds-tone') { ds.cardTheme = btn.getAttribute('data-val'); ctx.render(false); return true; }
    if (action === 'ds-toggle-excerpt') { ds.showExcerpt = !ds.showExcerpt; ctx.render(false); return true; }
    if (action === 'ds-toggle-qr') { ds.showQr = !ds.showQr; ctx.render(false); return true; }
    if (action === 'ds-h5-card') { ds.h5Mode = 'card'; ctx.render(false); return true; }
    if (action === 'ds-h5-back') { ds.h5Mode = 'sheet'; ctx.render(false); return true; }
    if (action === 'ds-ask-del') { ds.confirmDelete = true; ctx.render(false); return true; }
    if (action === 'ds-cancel-del') { ds.confirmDelete = false; ctx.render(false); return true; }
    if (action === 'ds-confirm-del') {
      ds.openDevice = null;
      ds.confirmDelete = false;
      ctx.showToast('演示：已删除当前内容并返回列表');
      ctx.closeDetail();
      return true;
    }
    if (action === 'ds-act') {
      const key = btn.getAttribute('data-key');
      const labels = {
        link: '已复制分享链接',
        'copy-img': '已复制分享卡片图片，可直接粘贴到微信 / 飞书',
        'save-png': '已导出高清 PNG 分享卡片',
        quote: '已复制推荐语与链接',
        extra: '演示：已执行扩展分享操作',
        'ask-ai': '已带入当前内容进入小伴 AI 对话',
        'send-friend': '已唤起系统分享发送卡片给好友',
      };
      if (key === 'link' || key === 'copy-img' || key === 'quote') {
        ds.copiedKey = key;
        ctx.render(false);
        setTimeout(() => { if (ds.copiedKey === key) { ds.copiedKey = ''; ctx.render(false); } }, 1800);
      }
      ctx.showToast(labels[key] || '操作成功');
      return true;
    }
    return false;
  }

  window.CommunityProfileAndShare = {
    unreadCount,
    openShareOnInit(mode) { ds.openDevice = mode || 'both'; ds.h5Mode = 'sheet'; },
    closeShare() {
      ds.openDevice = null;
      ds.confirmDelete = false;
      ps.pcWorkMenuId = null;
      ps.profileMoreOpen = false;
      ps.h5WorkSheet = null;
      ps.h5ProfileSheet = false;
    },
    renderMePage,
    renderProfilePage,
    renderWorksPage,
    renderNotificationsPage,
    renderProfileModals,
    renderShareOverlay,
    handleAction,
  };
})();
