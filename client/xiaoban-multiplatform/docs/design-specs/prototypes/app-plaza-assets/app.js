'use strict';
/* 图标路径取自 Icons.kt（XIcons），线宽 1.8 */
const P = {
  message:'M5 6h14a1 1 0 011 1v9a1 1 0 01-1 1h-7l-4 3v-3H5a1 1 0 01-1-1V7a1 1 0 011-1z',
  sparkle:'M12 3l1.8 5.2L19 10l-5.2 1.8L12 17l-1.8-5.2L5 10l5.2-1.8zM19 17l.7 1.8 1.8.7-1.8.7L19 22l-.7-1.8-1.8-.7 1.8-.7z',
  grid:'M6 4h3a2 2 0 0 1 2 2v3a2 2 0 0 1 -2 2h-3a2 2 0 0 1 -2 -2v-3a2 2 0 0 1 2 -2zM15 4h3a2 2 0 0 1 2 2v3a2 2 0 0 1 -2 2h-3a2 2 0 0 1 -2 -2v-3a2 2 0 0 1 2 -2zM6 13h3a2 2 0 0 1 2 2v3a2 2 0 0 1 -2 2h-3a2 2 0 0 1 -2 -2v-3a2 2 0 0 1 2 -2zM15 13h3a2 2 0 0 1 2 2v3a2 2 0 0 1 -2 2h-3a2 2 0 0 1 -2 -2v-3a2 2 0 0 1 2 -2z',
  user:'M8.5 8.5a3.5 3.5 0 1 0 7 0a3.5 3.5 0 1 0 -7 0zM5 20c.8-3.6 3.5-5.5 7-5.5s6.2 1.9 7 5.5', plus:'M12 5v14M5 12h14',
  menu:'M4 7h16M4 12h16M4 17h10', newchat:'M12 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-6M17.5 3.5a2 2 0 013 3L12 15l-4 1 1-4z',
  search:'M4.5 11a6.5 6.5 0 1 0 13 0a6.5 6.5 0 1 0 -13 0zM16 16l4.5 4.5', right:'M9 6l6 6-6 6', left:'M15 6l-6 6 6 6', close:'M6 6l12 12M18 6L6 18',
  more:'M3.4 12a1.6 1.6 0 1 0 3.2 0a1.6 1.6 0 1 0 -3.2 0zM10.4 12a1.6 1.6 0 1 0 3.2 0a1.6 1.6 0 1 0 -3.2 0zM17.4 12a1.6 1.6 0 1 0 3.2 0a1.6 1.6 0 1 0 -3.2 0z',
  play:'M8 5.5v13l10.5-6.5z', heart:'M12 20s-7-4.4-7-10a4 4 0 017-2.6A4 4 0 0119 10c0 5.6-7 10-7 10z',
  star:'M12 3.5l2.6 5.4 5.9.8-4.3 4.1 1 5.9-5.2-2.8-5.2 2.8 1-5.9-4.3-4.1 5.9-.8z', share:'M12 15V4M8 8l4-4 4 4M5 12v6a2 2 0 002 2h10a2 2 0 002-2v-6',
  flag:'M5 21V4M5 4h11l-2 4 2 4H5', refresh:'M20 12a8 8 0 10-2.3 5.6M20 5v5h-5', pen:'M4 20l4-1L20 7l-3-3L5 16ZM14 6l3 3',
  portrait:'M8 2.5h8a2 2 0 0 1 2 2v15a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2v-15a2 2 0 0 1 2-2zM11 18.5h2',
  landscape:'M2.5 8a2 2 0 0 1 2-2h15a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2h-15a2 2 0 0 1-2-2zM18.5 11v2',
  image:'M5 4h14a2 2 0 0 1 2 2v12a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-12a2 2 0 0 1 2 -2zM7.4 10a1.6 1.6 0 1 0 3.2 0a1.6 1.6 0 1 0 -3.2 0zM4 18l5-5 4 4 3-3 4 4',
  gamepad:'M6 8h12a4 4 0 0 1 4 4v1a4 4 0 0 1-7 2.6L14 15h-4l-1 .6A4 4 0 0 1 2 13v-1a4 4 0 0 1 4-4ZM8 11v3M6.5 12.5h3M16 12h.01M18 13.5h.01',
  monitor:'M5 5h14a2 2 0 0 1 2 2v8a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-8a2 2 0 0 1 2 -2zM8 21h8M12 17v4', wifi:'M3 9a15 15 0 0 1 18 0M6 13a10 10 0 0 1 12 0M9 17a5 5 0 0 1 6 0M12 21h.01',
  up:'M12 19V5M6 11l6-6 6 6', mic:'M12 3h0a3 3 0 0 1 3 3v5a3 3 0 0 1 -3 3h0a3 3 0 0 1 -3 -3v-5a3 3 0 0 1 3 -3zM5 11a7 7 0 0014 0M12 18v3',
  wave:'M4 11v2M8 8v8M12 5v19M16 8v8M20 11v2', lock:'M7 11h10a2 2 0 0 1 2 2v5a2 2 0 0 1 -2 2h-10a2 2 0 0 1 -2 -2v-5a2 2 0 0 1 2 -2zM8 11V8a4 4 0 018 0v3',
  chip:'M8 6h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2zM10 3v3M14 3v3M10 18v3M14 18v3M3 10h3M3 14h3M18 10h3M18 14h3',
  comment:'M5 6h14a1 1 0 011 1v9a1 1 0 01-1 1h-7l-4 3v-3H5a1 1 0 01-1-1V7a1 1 0 011-1z', eye:'M2.5 12s3.5-6.5 9.5-6.5S21.5 12 21.5 12 18 18.5 12 18.5 2.5 12 2.5 12zM9.5 12a2.5 2.5 0 1 0 5 0a2.5 2.5 0 1 0 -5 0z',
  book:'M5 4h10a3 3 0 013 3v13H8a3 3 0 01-3-3zM5 17a3 3 0 013-3h10'
};
P.wave = 'M4 11v2M8 8v8M12 5v14M16 8v8M20 11v2';
const icon = (n, cls = '') => `<svg class="${cls}" aria-hidden="true" viewBox="0 0 24 24"><path d="${P[n]}"/></svg>`;
const G = 'game-center-assets/';

const AUTHORS = {'纸鹤':'#5b6a9a','二十四':'#a57d43','听雨':'#5f7484','阿渔':'#3f8580','墨点':'#3b3733','小满':'#9a4a3c','Kiko':'#6d5a8f','我':'#b06a34','南风':'#6f7f5a','七七':'#8a6a8c','老周':'#5a6470'};
const APPS = [
  {id:'ink-blade', name:'水墨剑客', kind:'游戏', src:'official', o:'L', cover:G+'ink-blade.svg', plays:'2.3 万', line:'竹林、雪江、烽火三卷',
    desc:'横屏水墨动作：竹林、雪江、烽火三卷各六幅，逐幅解锁、集三印、求逸品。朱光一闪时出剑弹反，斩破月下无常、寒江钓叟与铁骑将军。'},
  {id:'dusk-glider', name:'暮色滑翔', kind:'游戏', src:'official', o:'L', cover:G+'dusk-glider.svg', plays:'1.6 万', line:'日落前飞过群岛', desc:'横屏 3D 滑翔：按住俯冲，松开起飞，在日落之前飞过一座座岛。'},
  {id:'jade-gomoku', name:'雅韵五子棋', kind:'游戏', src:'official', o:'L', cover:G+'jade-gomoku.svg', plays:'9,840', line:'人机与双人对弈', desc:'水墨玉石质感五子棋，支持人机对弈与双人对局，落子有金石音效与连珠特效。'},
  {id:'merge-2048', name:'2048', kind:'游戏', src:'official', o:'P', cover:G+'merge-2048.svg', plays:'3.1 万', line:'慢慢合出 2048', desc:'滑动合并相同数字，慢慢合出 2048。'},
  {id:'water-sort', name:'奶茶大师', kind:'游戏', src:'official', o:'P', cover:G+'water-sort.svg', plays:'1.2 万', line:'没有倒计时', desc:'把同色饮料倒在一起。8 关慢慢调，随时撤销，没有倒计时。'},
  {id:'block-puzzle', name:'消失高手', kind:'游戏', src:'official', o:'P', cover:G+'block-puzzle.svg', plays:'8,210', line:'填满一行就消掉', desc:'拖动方块填满整行或整列，一次消得越多，分数越高。'},
  {id:'pro-cube', name:'3D 全阶魔方', kind:'游戏', src:'official', o:'L', cover:G+'pro-cube.svg', plays:'6,532', line:'3 到 9 阶', desc:'3~9 阶真实质感磁力魔方：手势划动切层、多阶层深控制与智能还原。'},
  {id:'snake-arena', name:'小伴贪吃蛇', kind:'游戏', src:'official', o:'L', cover:G+'snake-arena.svg', plays:'1.4 万', line:'三分钟冲分', desc:'无尽成长或三分钟冲分，吃糖果长成大蛇，反杀 AI。'},
  {id:'mbti', name:'MBTI 性格测评', kind:'测评', src:'official', o:'P', glyph:'格', t:'--ochre', plays:'4.8 万', line:'48 道题，约 10 分钟',
    desc:'48 道情境题，约 10 分钟。做完给出四个维度的倾向和一份不贴标签的解读，结果只保存在你的手机上。'},
  {id:'focus', name:'番茄专注', kind:'工具', src:'official', o:'P', glyph:'专', t:'--pine', plays:'2.2 万', line:'25 分钟，只做一件事', desc:'25 分钟专注、5 分钟休息。专注时小伴不打扰你，结束时轻轻提醒。'},
  {id:'mood', name:'情绪天气', kind:'工具', src:'official', o:'P', glyph:'晴', t:'--indigo', plays:'1.1 万', line:'用一种天气记下今天', desc:'每天选一种天气记下心情，一个月后看看自己的晴雨表。'},
  {id:'decide', name:'帮我决定', kind:'工具', src:'official', o:'P', glyph:'择', t:'--plum', plays:'9,120', line:'选择困难时转一转', desc:'把几个选项写进去，转一下。转出来那一刻你心里的反应，往往就是答案。'},
  {id:'breathe', name:'呼吸练习', kind:'工具', src:'official', o:'P', glyph:'息', t:'--indigo', plays:'7,330', line:'三分钟慢下来', desc:'跟着圆的起伏呼吸：四秒吸、七秒停、八秒呼，三分钟让自己慢下来。'},
  {id:'enneagram', name:'九型人格', kind:'测评', src:'official', o:'P', glyph:'九', t:'--plum', plays:'2.6 万', line:'36 道题，约 8 分钟', desc:'36 道题，看看你在九种人格里离哪几种更近，附一段成长建议。'},
  {id:'days', name:'倒数日', kind:'工具', src:'official', o:'P', glyph:'日', t:'--ochre', plays:'1.9 万', line:'重要的日子还有几天', desc:'记下生日、纪念日、考试和假期，打开就知道还有几天。'},
  {id:'love', name:'恋爱观测评', kind:'测评', src:'official', o:'P', glyph:'缘', t:'--pine', plays:'3.4 万', line:'20 道题，约 5 分钟', desc:'20 道情境题，看看你在亲密关系里更在意什么。结果只保存在你的手机上。'},
  {id:'origami', name:'星河折纸', kind:'游戏', o:'L', art:'origami', author:'纸鹤', pick:true, plays:'5,214', likes:612, comments:88, time:'3 天前', tags:'#横屏 #睡前',
    note:'整局只有一个操作：按住折、松开飞。纸鹤擦过星星时的那一声很轻，适合睡前玩两局。', desc:'按住把纸折起来，松开让纸鹤起飞。穿过星河里的光点，折得越准飞得越远。'},
  {id:'solar', name:'节气小历', kind:'工具', o:'P', art:'solar', author:'二十四', pick:true, plays:'3,980', likes:420, comments:51, time:'1 周前', tags:'#节气 #日历',
    note:'每个节气配一句古诗和一件当季小事，做得很克制。霜降这一页尤其好看。', desc:'打开就是今天的节气：一句古诗、一件当季小事、离下个节气还有几天。'},
  {id:'rain', name:'雨窗白噪音', kind:'工具', o:'P', art:'rain', author:'听雨', pick:true, allowRemix:false, plays:'2,760', likes:301, comments:27, time:'2 周前', tags:'#白噪音 #专注',
    note:'雨声、雷声和远处的车声可以分别调。作者自己录的雨，比素材库的好听。', desc:'窗边下雨的白噪音。雨、雷、远处车流三条声音分开调，支持定时关闭。'},
  {id:'fish', name:'像素钓鱼，等浮漂一沉就收线', kind:'游戏', o:'P', art:'fish', author:'阿渔', remixes:['shrimp', 'nightfish', 'icefish'], plays:'312', likes:46, comments:12, time:'2 小时前', tags:'#像素 #休闲',
    desc:'一根竿、一片湖，等浮漂一沉就收线。钓到的鱼会进图鉴，一共 24 种。第一次做游戏，请大家多提意见。'},
  {id:'lot', name:'今日一签', kind:'工具', o:'P', art:'lot', author:'小满', plays:'1,024', likes:138, comments:40, time:'5 小时前', tags:'#每日 #小签', desc:'每天可以抽一支签，签文都是我写的大白话，不吓人。'},
  {id:'bounce', name:'弹弹球：三颗球不许落地', kind:'游戏', o:'L', art:'bounce', author:'Kiko', plays:'208', likes:19, comments:6, time:'昨天', tags:'#横屏 #反应', desc:'横屏玩，左右两只手各管一块板，三颗球不许落地。'},
  {id:'stroke', name:'一笔画 60 关', kind:'游戏', o:'P', art:'stroke', author:'墨点', plays:'560', likes:73, comments:15, time:'昨天', tags:'#益智 #水墨', desc:'不抬笔，一笔把图案连完。60 关，越往后越像书法。'},
  {id:'plum', name:'寒梅计时', kind:'工具', o:'P', art:'plum', author:'小满', plays:'97', likes:11, comments:3, time:'3 天前', tags:'#计时', desc:'倒计时的时候，枝头的梅花一朵朵开，开满就到时间。'},
  {id:'tea', name:'雨夜小馆', kind:'游戏', o:'P', art:'tea', author:'我', pick:true, remixN:3, plays:'1,208', likes:214, comments:36, time:'12 天前', tags:'#剧情', note:'客人的心事写得很真，面也是。', desc:'深夜小馆，客人进门说一句心事，你给他做一碗面。'},
  {id:'shrimp', name:'钓虾也行', kind:'游戏', o:'P', art:'fish', filter:'hue-rotate(-40deg) saturate(1.2)', author:'老周', remixOf:'fish', plays:'88', likes:21, comments:5, time:'12 分钟前', tags:'#改编 #休闲', desc:'把鱼都换成了虾，收线要快，虾会倒着跑。'},
  {id:'nightfish', name:'夜钓', kind:'游戏', o:'P', art:'fish', filter:'brightness(.55) hue-rotate(30deg)', author:'南风', remixOf:'fish', plays:'143', likes:37, comments:9, time:'1 小时前', tags:'#改编 #夜晚', desc:'天黑以后的湖，浮漂会发光，偶尔能钓到萤火鱼。'},
  {id:'icefish', name:'冰湖垂钓', kind:'游戏', o:'P', art:'fish', filter:'hue-rotate(160deg) saturate(.5) brightness(1.15)', author:'七七', remixOf:'fish', plays:'61', likes:12, comments:2, time:'40 分钟前', tags:'#改编 #冬天', desc:'先在冰面上凿个洞，再慢慢等。'},
  {id:'water', name:'喝水提醒', kind:'工具', o:'P', art:'water', author:'我', plays:'86', likes:9, comments:2, time:'4 天前', tags:'#健康', desc:'按你的作息提醒喝水，杯子会慢慢变空。'}
];
const A = Object.fromEntries(APPS.map(a => [a.id, a]));
const GAMES = ['ink-blade', 'origami', 'dusk-glider', 'merge-2048', 'jade-gomoku', 'water-sort', 'block-puzzle', 'pro-cube', 'snake-arena'].map(i => A[i]);
const TOOLS = APPS.filter(a => a.src === 'official' && a.kind !== '游戏');
const PICKS = ['origami', 'solar', 'rain'].map(i => A[i]);
const WORKS = ['fish', 'origami', 'lot', 'bounce', 'stroke', 'rain', 'plum', 'solar'].map(i => A[i]);
const COMMENTS = [['南风', '浮漂沉下去那一下好有手感，求出夜钓', '1 小时前', 8], ['七七', '图鉴第 17 种在哪钓的？找了好久', '40 分钟前', 3], ['老周', '做一个我的版本，改成了钓虾 😂', '12 分钟前', 1]];
const recent = ['ink-blade', 'mbti', 'merge-2048', 'origami', 'focus', 'solar'];
const favs = new Set(['solar', 'water-sort', 'mood']);
const likes = new Set();
const follows = new Set();

/* ---------- 演示封面 ---------- */
function art(kind) {
  const svg = (inner, extra = '') => `<svg viewBox="0 0 100 100" preserveAspectRatio="xMidYMid slice" ${extra}>${inner}</svg>`;
  switch (kind) {
    case 'origami': {
      const stars = Array.from({length: 22}, (_, i) => `<circle cx="${(i * 37) % 100}" cy="${(i * 23 + 7) % 100}" r="${i % 4 ? .5 : .9}" fill="#fff" opacity="${.35 + (i % 3) * .2}"/>`).join('');
      return `<div class="art a-origami">${svg(stars + `<path d="M-5 80 Q 50 55 105 70" stroke="#ffffff22" stroke-width="10" fill="none"/><g class="crane"><polygon points="38,56 60,40 52,60" fill="#f1e6d0"/><polygon points="52,60 60,40 72,62" fill="#d6c7a9"/><polygon points="28,50 38,56 47,59" fill="#e9dcc2"/><polygon points="60,40 64,30 62,44" fill="#f1e6d0"/></g><g class="crane" style="animation-delay:-3s"><polygon points="70,26 80,19 76,28" fill="#e6d7bb" opacity=".7"/><polygon points="76,28 80,19 86,29" fill="#cbbb9c" opacity=".7"/></g>`)}</div>`;
    }
    case 'solar': return `<div class="art a-solar">${svg(`<circle cx="64" cy="38" r="24" fill="#c98a42" opacity=".85"/><path d="M0 78 Q 30 70 55 76 T 100 74 V100 H0Z" fill="#d9ccb3"/><path d="M0 86 Q 40 80 100 88 V100 H0Z" fill="#c8b897"/><text x="22" y="28" font-size="15" fill="#3b352c" font-family="Songti SC,STSong,serif" font-weight="700" style="writing-mode:vertical-rl;letter-spacing:4px">霜降</text>`)}</div>`;
    case 'rain': return `<div class="art a-rain">${Array.from({length: 28}, (_, i) => `<i style="left:${(i * 37) % 100}%;top:${(i * 19) % 60}%;animation-delay:-${(i % 7) * .16}s"></i>`).join('')}${svg(`<circle cx="74" cy="30" r="12" fill="#f3d9a4" opacity=".22"/><rect x="8" y="8" width="84" height="84" fill="none" stroke="#1f2830" stroke-width="3"/><line x1="50" y1="8" x2="50" y2="92" stroke="#1f2830" stroke-width="2.5"/>`, 'style="position:absolute;inset:0"')}</div>`;
    case 'fish': return `<div class="art a-fish">${svg(`<rect x="0" y="0" width="100" height="34" fill="#e8f1ee"/><path d="M0 34 Q 12 31 25 34 T 50 34 T 75 34 T 100 34 V40 H0Z" fill="#9cc9c3"/><line x1="78" y1="0" x2="58" y2="36" stroke="#5b4636" stroke-width="1"/><rect x="56" y="33" width="4" height="6" fill="#e05a4a"/>`, 'style="position:absolute;inset:0"')}<i style="left:16%;top:58%"></i><i style="left:40%;top:76%;animation-delay:-1.4s;background:#f6d06a;box-shadow:6px 0 0 #f6d06a,-4px 0 0 #e5b54c"></i><i style="left:8%;top:86%;animation-delay:-2.2s"></i></div>`;
    case 'stroke': return `<div class="art a-stroke">${svg(`<path d="M24 66 C 26 30, 64 18, 76 42 S 58 82, 34 74 L 70 62"/><circle cx="24" cy="66" r="2.5" fill="#b8352a"/>`)}</div>`;
    case 'lot': return `<div class="art a-lot">${svg(`<path d="M30 92 L36 40 H64 L70 92Z" fill="#2c0f0b" opacity=".6"/><g transform="rotate(8 50 40)"><rect x="45" y="12" width="10" height="56" rx="2" fill="#d9a35b"/><text x="50" y="38" font-size="7" text-anchor="middle" fill="#5a2a14" font-family="Songti SC,serif" font-weight="700">上</text><text x="50" y="48" font-size="7" text-anchor="middle" fill="#5a2a14" font-family="Songti SC,serif" font-weight="700">签</text></g>`)}</div>`;
    case 'bounce': return `<div class="art a-bounce">${svg(`<line x1="0" y1="84" x2="100" y2="84" stroke="#ffffff30" stroke-width=".8"/><rect x="10" y="78" width="22" height="3" rx="1.5" fill="#e9e3d6"/><rect x="66" y="78" width="22" height="3" rx="1.5" fill="#e9e3d6"/>`, 'style="position:absolute;inset:0"')}<i style="left:22%;background:#f0a35e"></i><i style="left:46%;background:#7fc4b8;animation-delay:-.35s"></i><i style="left:70%;background:#e8798a;animation-delay:-.7s"></i></div>`;
    case 'tea': return `<div class="art a-tea">${svg(`<circle cx="50" cy="34" r="22" fill="#f0a24a" opacity=".18"/><rect x="43" y="22" width="14" height="22" rx="6" fill="#d65b3a"/><rect x="47" y="18" width="6" height="4" fill="#2a1d15"/><text x="50" y="37" font-size="7" text-anchor="middle" fill="#fbe3c0" font-family="Songti SC,serif" font-weight="700">面</text><path d="M20 78 H80 L74 92 H26Z" fill="#8a5a3a"/>`)}</div>`;
    case 'water': return `<div class="art a-water">${svg(`<path d="M34 22 H66 L62 82 H38Z" fill="none" stroke="#7aa0ad" stroke-width="2"/><path d="M36.5 52 H63.5 L62 82 H38Z" fill="#8fc2d2" opacity=".7"/>`)}</div>`;
    case 'plum': return `<div class="art a-plum">${svg(`<path d="M10 90 C 30 70, 40 60, 52 40 S 74 18, 90 12" stroke="#3b2c24" stroke-width="2.4" fill="none"/><path d="M40 60 C 30 52, 24 50, 18 40" stroke="#3b2c24" stroke-width="1.6" fill="none"/>${[[52,40],[66,28],[80,18],[30,50],[20,42],[44,54]].map(([x, y], i) => `<g opacity="${i < 4 ? 1 : .35}"><circle cx="${x}" cy="${y}" r="4" fill="#c8434a"/><circle cx="${x}" cy="${y}" r="1.2" fill="#f4d48a"/></g>`).join('')}`)}</div>`;
  }
  return '<div class="art"></div>';
}
function cover(a, style = '') {
  if (a.cover) return `<img src="${a.cover}" alt="" style="${style}">`;
  if (a.art) return a.filter ? `<div class="art" style="filter:${a.filter}">${art(a.art)}</div>` : art(a.art);
  return `<div class="art" style="background:color-mix(in srgb,var(${a.t}) 14%,var(--surface));display:grid;place-items:center"><span style="font-family:Songti SC,STSong,serif;font-weight:700;font-size:60px;color:var(${a.t})">${a.glyph}</span></div>`;
}
function ico(a, size = '') {
  if (a.glyph) return `<div class="ico glyph ${size}" style="--t:var(${a.t})"><span>${a.glyph}</span></div>`;
  return `<div class="ico ${size}">${cover(a)}</div>`;
}
const ava = (name, cls = '') => `<span class="ava ${cls}" style="background:${AUTHORS[name] || '#888'}">${name[0]}</span>`;
const ori = a => a.o === 'L' ? `<span class="ori">${icon('landscape')}横屏</span>` : '';
const oriText = a => a.o === 'L' ? '横屏' : '竖屏';
const isPost = a => a.src !== 'official';
const short = a => a.name.split(/[，：]/)[0];
const byline = a => a.src === 'official' ? '小伴官方' : '@' + a.author;
const canRemix = a => a.allowRemix !== false && a.author !== '我';
const esc = t => String(t).replace(/[&<>"]/g, c => ({'&':'&amp;', '<':'&lt;', '>':'&gt;', '"':'&quot;'}[c]));

/* ---------- 画板 ---------- */
const BOARDS = [
  {id:'b1', title:'小伴AI · 首页', start:{screen:'home'}},
  {id:'b2', title:'小伴AI · 下拉打开我的应用', start:{screen:'home', drawer:true, hinted:true}},
  {id:'b3', title:'小伴AI · 对话里递上应用', start:{screen:'chat', chat:'mbti'}},
  {id:'b4', title:'应用 · 广场', start:{screen:'plaza'}},
  {id:'b5', title:'应用 · 社区作品（帖子）', start:{screen:'post', post:'fish'}},
  {id:'b9', title:'做一个我的版本', start:{screen:'post', post:'fish', remix:'fish'}},
  {id:'b6', title:'应用 · 长按官方应用查看详情', start:{screen:'plaza', sheet:'mbti'}},
  {id:'b7', title:'全屏 · 竖屏', start:{screen:'run', app:'mbti'}},
  {id:'b8', title:'全屏 · 横屏', start:{screen:'run', app:'ink-blade'}}
];
const phones = {};
const MBTI_Q = [
  ['在一场热闹的聚会上，你更常……', '找人聊天，越聊越有精神', '待一会儿就想找个安静角落'],
  ['做一个新决定时，你更依赖……', '具体的事实和过往经验', '直觉，和事情可能变成的样子'],
  ['朋友向你倾诉烦恼，你第一反应是……', '帮他分析问题，想办法', '先陪着他，理解他的感受'],
  ['周末的计划，你更喜欢……', '提前安排好，按计划走', '看当天心情，随走随定']
];

function init() {
  const host = document.getElementById('boards');
  host.innerHTML = BOARDS.map(b => `<section class="board"><h2>${b.title}</h2><div class="phone" id="${b.id}"></div>
    <div class="board-tools"><button data-sysback="${b.id}">${icon('left')}系统返回</button><button data-reset="${b.id}">${icon('refresh')}回到起点</button></div></section>`).join('');
  BOARDS.forEach(b => reset(b.id));
  host.addEventListener('click', e => {
    const sb = e.target.closest('[data-sysback]'); if (sb) return sysBack(sb.dataset.sysback);
    const rs = e.target.closest('[data-reset]'); if (rs) return reset(rs.dataset.reset);
  });
  document.getElementById('themeBtn').onclick = e => { const d = document.body.classList.toggle('dark'); e.target.textContent = d ? '浅色预览' : '深色预览'; };
  document.getElementById('sizeBtn').onclick = e => { const n = document.body.classList.toggle('narrow'); e.target.textContent = n ? '360 → 390' : '390 → 360'; };
  document.getElementById('safeBtn').onclick = e => { const on = document.body.classList.toggle('safe-on'); e.target.textContent = on ? '隐藏安全区' : '显示安全区'; };
}
function reset(id) {
  const b = BOARDS.find(x => x.id === id);
  phones[id] = {screen:'plaza', sub:'广场', stack:[], sheet:null, drawer:false, chat:null, list:null, filter:'全部', wtab:'推荐', post:null, app:null, menu:false, armed:false, pop:false, q:0, sel:null, hinted:false, remix:null, remixSrc:null, idea:'', ...b.start};
  phones[id].home = b.start.screen === 'run' ? 'plaza' : b.start.screen;
  const el = document.getElementById(id);
  if (!el.dataset.bound) { bind(el, id); el.dataset.bound = '1'; }
  render(id);
}
const snapshot = s => ({screen:s.screen, sub:s.sub, chat:s.chat, list:s.list, filter:s.filter, post:s.post});
function go(id, patch) { const s = phones[id]; s.stack.push(snapshot(s)); Object.assign(s, {sheet:null, drawer:false, pop:false}, patch); render(id); }
function pop(id) {
  const s = phones[id], prev = s.stack.pop();
  Object.assign(s, prev || {screen:s.home}, {sheet:null, pop:false, menu:false, armed:false});
  render(id);
}
function sysBack(id) {
  const s = phones[id];
  if (s.screen === 'run') {
    if (s.menu) return exitRun(id);
    s.menu = true; s.armed = true; render(id); return;
  }
  if (s.pop) { s.pop = false; return render(id); }
  if (s.sheet) { s.sheet = null; return render(id); }
  if (s.drawer) { s.drawer = false; return render(id); }
  if (!s.stack.length) return toast(id, '已在起点');
  pop(id);
}
function launch(id, appId) {
  const s = phones[id], a = A[appId];
  s.stack.push(snapshot(s));
  Object.assign(s, {screen:'launch', app:appId, sheet:null, drawer:false, pop:false, menu:false, armed:false, q:0, sel:null});
  const i = recent.indexOf(appId); if (i >= 0) recent.splice(i, 1); recent.unshift(appId);
  render(id);
  setTimeout(() => { if (s.screen === 'launch' && s.app === appId) { s.screen = 'run'; render(id); } }, a.o === 'L' ? 1400 : 700);
}
function exitRun(id) {
  const s = phones[id], name = A[s.app].name.split(/[，：]/)[0];
  const prev = s.stack.pop();
  Object.assign(s, prev || {screen:s.home}, {menu:false, armed:false, sheet:null, pop:false});
  render(id); toast(id, `已退出${name}，进度已保存`);
}
function toast(id, text) {
  const t = document.querySelector(`#${id} .toast`); if (!t) return;
  t.textContent = text; t.classList.add('show'); clearTimeout(t._h); t._h = setTimeout(() => t.classList.remove('show'), 1800);
}
function openItem(id, appId) {
  if (isPost(A[appId])) go(id, {screen:'post', post:appId});
  else launch(id, appId);
}
function showOfficialDetails(id, appId) {
  Object.assign(phones[id], {sheet:appId, pop:false});
  render(id);
}

function bind(el, id) {
  el.addEventListener('input', e => { if (e.target.matches('.idea')) phones[id].idea = e.target.value; });
  el.addEventListener('click', e => {
    const idea = e.target.closest('[data-idea]');
    if (idea) { const ta = el.querySelector('.idea'); ta.value = (ta.value ? ta.value + '，' : '') + idea.dataset.idea; phones[id].idea = ta.value; return; }
    if (e.target.closest('[data-stop]') && !e.target.closest('[data-a]')) return;
    const t = e.target.closest('[data-a]'); if (!t || !el.contains(t)) return;
    const s = phones[id], [act, arg] = t.dataset.a.split(':');
    switch (act) {
      case 'tab':
        if (arg === 'home' && s.screen !== 'home') go(id, {screen:'home'});
        else if (arg === 'plaza' && s.screen !== 'plaza') go(id, {screen:'plaza', sub:'广场'});
        else if (arg === 'publish') toast(id, '发布：帖子、图片、作品');
        else if (arg === 'community') toast(id, '社区');
        else if (arg === 'mine') toast(id, '我的');
        break;
      case 'sub': s.sub = arg; render(id); break;
      case 'drawer': s.drawer = !s.drawer; s.hinted = true; render(id); break;
      case 'chat': go(id, {screen:'chat', chat:arg}); break;
      case 'open': openItem(id, arg); break;
      case 'closesheet': s.sheet = null; s.pop = false; render(id); break;
      case 'launch': launch(id, arg); break;
      case 'list': go(id, {screen:'list', list:arg, filter:'全部'}); break;
      case 'filter': s.filter = arg; render(id); break;
      case 'wtab': s.wtab = arg; render(id); break;
      case 'back': pop(id); break;
      case 'plaza': go(id, {screen:'plaza', sub:'广场'}); break;
      case 'fav': favs.has(arg) ? favs.delete(arg) : favs.add(arg); render(id); toast(id, favs.has(arg) ? '已收藏，首页下拉就能找到' : '已取消收藏'); break;
      case 'like': likes.has(arg) ? likes.delete(arg) : likes.add(arg); render(id); break;
      case 'follow': follows.has(arg) ? follows.delete(arg) : follows.add(arg); render(id); break;
      case 'pop': s.pop = !s.pop; render(id); break;
      case 'menu': s.menu = true; render(id); break;
      case 'closemenu': s.menu = false; s.armed = false; render(id); break;
      case 'exit': exitRun(id); break;
      case 'restart': s.menu = false; s.armed = false; s.q = 0; s.sel = null; render(id); toast(id, '已重新开始'); break;
      case 'answer': s.sel = +arg; render(id); setTimeout(() => { s.q++; s.sel = null; render(id); }, 260); break;
      case 'remix':
        if (s.screen === 'run') exitRun(id);
        s.remix = arg; s.idea = ''; s.pop = false; s.sheet = null; render(id); break;
      case 'closeremix': s.remix = null; render(id); break;
      case 'startremix': s.remixSrc = s.remix; s.remix = null; go(id, {screen:'chat', chat:'remix'}); break;
      case 'remixes': go(id, {screen:'remixes', post:arg}); break;
      case 'post': go(id, {screen:'post', post:arg}); break;
      case 'allow': A[arg].allowRemix = A[arg].allowRemix === false; render(id); toast(id, A[arg].allowRemix ? '已允许别人改编' : '已关闭，别人看不到「做一个我的版本」'); break;
      case 'make': toast(id, '描述想法，小伴帮你做出来'); break;
      case 'toast': toast(id, t.dataset.msg); break;
    }
  });
  let y0 = null, press = null;
  const officialId = target => {
    const [act, appId] = (target.closest('[data-a]')?.dataset.a || '').split(':');
    return act === 'open' && A[appId]?.src === 'official' ? appId : null;
  };
  el.addEventListener('pointerdown', e => {
    const sc = e.target.closest('.home-scroll');
    if (sc && sc.scrollTop <= 0 && !phones[id].drawer) y0 = e.clientY;
    const appId = officialId(e.target);
    if (appId && e.button === 0) press = {appId, x:e.clientX, y:e.clientY, at:e.timeStamp};
  });
  el.addEventListener('pointermove', e => {
    if (press && Math.hypot(e.clientX - press.x, e.clientY - press.y) > 8) press = null;
    if (y0 !== null && e.clientY - y0 > 46) { y0 = null; phones[id].drawer = true; phones[id].hinted = true; render(id); }
  });
  el.addEventListener('pointerup', e => {
    if (press && e.timeStamp - press.at >= 500 && press.appId === officialId(e.target)) showOfficialDetails(id, press.appId);
  });
  el.addEventListener('contextmenu', e => {
    const appId = officialId(e.target);
    if (!appId) return;
    e.preventDefault();
    press = null;
    showOfficialDetails(id, appId);
  });
  const clearPointer = () => { y0 = null; press = null; };
  window.addEventListener('pointerup', clearPointer);
  window.addEventListener('pointercancel', clearPointer);
}

/* ---------- 渲染 ---------- */
const statusBar = (style = '') => `<div class="status" style="${style}">9:41<span>${icon('wifi')}<i class="battery"></i></span></div>`;
function nav(active) {
  const slots = [['home', 'message', '小伴AI'], ['plaza', 'sparkle', '应用'], ['publish', 'plus', '发布'], ['community', 'grid', '社区'], ['mine', 'user', '我的']];
  return `<nav class="nav" aria-label="底部导航">${slots.map(([k, ic, label]) => k === 'publish'
    ? `<button class="publish" data-a="tab:publish" aria-label="发布"><span>${icon(ic)}</span></button>`
    : `<button data-a="tab:${k}" class="${active === k ? 'on' : ''}" role="tab" aria-selected="${active === k}">${icon(ic)}${label}</button>`).join('')}</nav>`;
}
function render(id) {
  const el = document.getElementById(id), s = phones[id];
  const keep = {};
  ['.scroll', '.sheet .inner', '.gallery'].forEach(q => { const n = el.querySelector(q); if (n && el.dataset.key === s.screen + (s.post || '')) keep[q] = [n.scrollTop, n.scrollLeft]; });
  el.classList.toggle('landscape', s.screen === 'run' && A[s.app].o === 'L');
  let html = '<div class="island"></div>' + ({home, chat, plaza, list, post, remixes, launch:launchScreen, run}[s.screen])(s);
  if (s.sheet) html += sheet(s);
  if (s.remix) html += remixSheet(s);
  html += '<div class="gesture"></div><div class="toast" role="status"></div>';
  el.innerHTML = html; el.dataset.key = s.screen + (s.post || '');
  Object.entries(keep).forEach(([q, [t, l]]) => { const n = el.querySelector(q); if (n) { n.scrollTop = t; n.scrollLeft = l; } });
}

function backdrop() {
  return `<div class="backdrop"><svg viewBox="0 0 390 320" preserveAspectRatio="none"><defs><linearGradient id="bd" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="var(--soft)" stop-opacity=".9"/><stop offset="1" stop-color="var(--bg)"/></linearGradient></defs>
    <rect width="390" height="320" fill="url(#bd)"/><path d="M0 198 C62 154 117 186 179 160 C242 134 312 192 390 147 V320 H0Z" fill="var(--accent)" opacity=".05"/><path d="M0 250 C86 205 148 256 226 224 C289 198 343 243 390 224 V320 H0Z" fill="var(--accent)" opacity=".07"/>
    <rect y="190" width="390" height="130" fill="url(#bd2)"/><defs><linearGradient id="bd2" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="var(--bg)" stop-opacity="0"/><stop offset="1" stop-color="var(--bg)"/></linearGradient></defs></svg></div>`;
}
function home(s) {
  const tiles = arr => arr.slice(0, 8).map(a => `<button class="tile" data-a="launch:${a.id}">${ico(a)}<span>${a.name}</span></button>`).join('');
  return `<div class="screen">${backdrop()}${statusBar()}
    <div class="bar"><button class="ic" data-a="toast" data-msg="对话列表" aria-label="对话列表">${icon('menu')}</button><h1>小伴AI</h1><button class="ic" data-a="toast" data-msg="新对话" aria-label="新对话">${icon('newchat')}</button></div>
    <div class="scroll home-scroll" style="position:relative;z-index:2"><div class="pad">
      <button class="pull" data-a="drawer" aria-label="打开我的应用"><i></i>${s.hinted ? '' : '下拉打开我的应用'}</button>
      <div class="t-cap" style="color:var(--sub);margin-top:${s.hinted ? 8 : 4}px">11月7日 · 星期六</div>
      <div class="hello">晚上好</div>
      <div class="line">今天辛苦了。来一局<button data-a="open:ink-blade">水墨剑客</button>？烽火卷刚上新。</div>
      <button class="composer" data-a="chat:mbti"><div>问点什么，或描述想要的内容…</div><div class="row"><span class="chip">小伴 Pro ▾</span><small>今日剩余 86 点</small><span class="send">${icon('up')}</span></div></button>
      <div class="sec"><h3>用小伴创作</h3></div>
      <div class="feat">
        <button style="--t:var(--ochre)" data-a="toast" data-msg="创作图片"><span class="i">${icon('image')}</span><b>创作图片</b><small>描述画面，生成插画和壁纸</small></button>
        <button style="--t:var(--pine)" data-a="make"><span class="i">${icon('gamepad')}</span><b>创作应用</b><small>说出想法，做成游戏或小工具</small></button>
        <button style="--t:var(--plum)" data-a="toast" data-msg="桌面装扮"><span class="i">${icon('sparkle')}</span><b>桌面装扮</b><small>壁纸、小组件、特效和伙伴</small></button>
        <button style="--t:var(--indigo)" data-a="toast" data-msg="遥控电脑"><span class="i">${icon('monitor')}</span><b>遥控电脑</b><small>MacBook 在线</small></button>
      </div>
      <div class="sec"><h3>试试这样问我</h3></div>
      <div class="starters">
        <button class="card" style="--t:var(--ochre)" data-a="chat:mbti">${icon('sparkle')}<span>帮我测一下 MBTI</span></button>
        <button class="card" style="--t:var(--pine)" data-a="chat:fun">${icon('gamepad')}<span>有点无聊，来点好玩的</span></button>
        <button class="card" style="--t:var(--plum)" data-a="toast" data-msg="普通对话">${icon('book')}<span>给我讲一个三分钟能读完的小故事</span></button>
      </div>
      <div class="sec"><h3>继续对话</h3></div>
      <div class="card recent-chat"><div><span class="b">${icon('message')}</span><span style="flex:1"><b>周末去哪儿走走</b><small>2 小时前</small></span></div><div><span class="b">${icon('message')}</span><span style="flex:1"><b>帮我改一下简历</b><small>昨天</small></span></div></div>
      <div class="nav-space"></div></div></div>
    ${nav('home')}
    <div class="scrim ${s.drawer ? 'on' : ''}" data-a="drawer"></div>
    <div class="drawer ${s.drawer ? 'open' : ''}" aria-hidden="${!s.drawer}">
      <h4>最近用过</h4><div class="tiles">${tiles(recent.map(i => A[i]))}</div>
      <h4>我的收藏</h4><div class="tiles">${tiles([...favs].map(i => A[i]))}</div>
      <button class="go" data-a="plaza">去应用广场${icon('right')}</button>
      <button class="grab" data-a="drawer" aria-label="收起"></button>
    </div></div>`;
}
function appCard(a) {
  return `<div class="card appcard" data-a="open:${a.id}" role="button">${ico(a, 'sm')}<span style="min-width:0;flex:1"><b>${a.name}</b><small>${a.src === 'official' ? '官方' : a.author} · ${a.kind} · ${oriText(a)} · ${a.line}</small></span><button class="pill" data-a="launch:${a.id}">打开</button></div>`;
}
function chat(s) {
  const body = s.chat === 'remix' ? remixChat(s) : s.chat === 'mbti'
    ? `<div class="me">帮我测一下 MBTI</div>
       <div class="bot">好呀。这个测评一共 48 道题，大概 10 分钟，结果只存在你手机上。做完我陪你看看结果。${appCard(A.mbti)}</div>
       <div class="sugs"><button class="chip" data-a="make">给我做一个 5 道题的轻松版</button></div>`
    : `<div class="me">有点无聊，来点好玩的</div>
       <div class="bot">给你挑了两个，十分钟内能玩完：${appCard(A['ink-blade'])}${appCard(A['merge-2048'])}</div>
       <div class="bot">也可以看看大家最近做的，比如${appCard(A.origami)}</div>
       <div class="sugs"><button class="chip" data-a="make">都不想玩，帮我做一个</button><button class="chip" data-a="plaza">去应用广场</button></div>`;
  return `<div class="screen">${statusBar()}
    <div class="bar"><button class="ic" data-a="back" aria-label="返回">${icon('left')}</button><h1>小伴AI</h1><button class="ic" data-a="toast" data-msg="对话设置" aria-label="更多">${icon('more')}</button></div>
    <div class="scroll"><div class="msgs">${body}</div></div>
    <div class="dock"><div>继续说…</div><div class="tb">${icon('plus')}${icon('chip')}${icon('lock')}<span style="flex:1"></span>${icon('mic')}${icon('wave')}<span class="send sm">${icon('up')}</span></div></div></div>`;
}
function gcard(a) {
  return `<button class="gcard" data-a="open:${a.id}"><div class="cv">${cover(a)}</div><b>${a.name}</b><small>${a.src === 'official' ? '官方' : a.author} · ${oriText(a)}</small></button>`;
}
function ttile(a) {
  return `<button class="tile ttile" data-a="open:${a.id}">${ico(a)}<span>${a.name.replace(' 性格测评', '')}</span></button>`;
}
function trow(a) {
  return `<div class="trow" data-a="open:${a.id}" role="button">${ico(a, 'sm')}<span style="min-width:0;flex:1"><b>${a.name}</b><small>${a.line}</small></span><button class="pill" data-a="launch:${a.id}">打开</button></div>`;
}
function pcard(a) {
  return `<button class="card pcard" data-a="open:${a.id}"><div class="cv">${cover(a)}</div><b>${a.name}</b><div class="who">${ava(a.author)}<span>${a.author}</span></div></button>`;
}
function wcard(a) {
  const liked = likes.has(a.id);
  return `<div class="card wcard" data-a="open:${a.id}" role="button"><div class="cv ${a.o === 'L' ? 'l' : 'p'}">${cover(a)}<div class="stat"><span>${icon('play')}${a.plays}</span><span>${icon('comment')}${a.comments}</span></div></div>
    <div class="meta"><div class="tt">${a.pick ? '<span style="color:var(--brand-ink)">精选 </span>' : ''}${a.name}</div><div class="au">${ava(a.author)}<span class="nm">${a.author}</span><button class="lk ${liked ? 'on' : ''}" data-a="like:${a.id}" aria-label="点赞">${icon('heart')}${a.likes + (liked ? 1 : 0)}</button></div></div></div>`;
}
function waterfall(items) {
  const cols = [[], []], h = [0, 0];
  items.forEach(a => { const c = h[0] <= h[1] ? 0 : 1; cols[c].push(a); h[c] += a.o === 'L' ? 0.62 + 0.5 : 1.33 + 0.5; });
  return `<div class="fall">${cols.map(c => `<div>${c.map(wcard).join('')}</div>`).join('')}</div>`;
}
function plaza(s) {
  const top = `<div class="top-tabs">${['广场', '我的作品'].map(t => `<button data-a="sub:${t}" class="${s.sub === t ? 'active' : ''}">${t}</button>`).join('')}
    <span class="right"><button class="ic" data-a="toast" data-msg="搜索应用、作者和标签" aria-label="搜索">${icon('search')}</button>${s.sub === '我的作品' ? `<button class="plus" data-a="make" aria-label="新建作品">${icon('plus')}</button>` : ''}</span></div>`;
  let body;
  if (s.sub === '广场') {
    const works = s.wtab === '最新' ? [...WORKS].reverse() : s.wtab === '游戏' ? WORKS.filter(a => a.kind === '游戏') : s.wtab === '工具' ? WORKS.filter(a => a.kind !== '游戏') : WORKS;
    body = `
      <div class="recent">${recent.slice(0, 6).map(i => `<button class="tile" data-a="launch:${i}">${ico(A[i])}<span>${A[i].name}</span></button>`).join('')}</div>
      <div class="sec"><h3>精选游戏</h3><button data-a="list:games">全部${icon('right')}</button></div>
      <div class="hscroll">${GAMES.map(gcard).join('')}</div>
      <div class="sec"><h3>精选工具</h3><button data-a="list:tools">全部${icon('right')}</button></div>
      <div class="tgrid">${TOOLS.slice(0, 4).map(ttile).join('')}</div>
      <div class="sec"><h3>创作者精选</h3><button data-a="list:picks">全部${icon('right')}</button></div>
      <div class="hscroll">${PICKS.map(pcard).join('')}</div>
      <div class="sec"><h3>社区作品</h3><button data-a="make">发布我的${icon('right')}</button></div>
      <div class="chips">${['推荐', '最新', '游戏', '工具'].map(t => `<button class="chip ${s.wtab === t ? 'on' : ''}" data-a="wtab:${t}">${t}</button>`).join('')}</div>
      ${waterfall(works)}<div class="endnote">继续下滑，加载更多</div>`;
  } else {
    body = `<button class="card make" data-a="make">${icon('sparkle')}<span style="flex:1"><b>你想做个什么？</b><small>一个游戏、一个小工具、一份测评…</small></span></button>
      <div class="sec"><h3>已发布</h3><span class="t-cap">发布后出现在社区作品里</span></div>
      ${['tea', 'water'].map(i => { const a = A[i]; return `<div class="card work"><div class="work-top" data-a="open:${a.id}" role="button"><div class="art-box">${cover(a)}</div><span style="flex:1"><b>${a.name}</b><small>${a.kind} · ${oriText(a)} · ${a.time}发布</small>${a.pick ? '<small class="picked">已选入创作者精选</small>' : ''}</span></div>
        <div class="stats"><span>${icon('play')}${a.plays}</span><span>${icon('heart')}${a.likes}</span><span>${icon('comment')}${a.comments}</span>${a.remixN ? `<span>${icon('pen')}${a.remixN} 个改编</span>` : ''}</div>
        <div class="allow"><span>允许别人改编</span><button class="sw ${a.allowRemix === false ? '' : 'on'}" data-a="allow:${a.id}" role="switch" aria-checked="${a.allowRemix !== false}" aria-label="允许别人改编"><i></i></button></div></div>`; }).join('')}
      <div class="sec"><h3>草稿</h3></div>
      <div class="card work"><div class="work-top" data-a="make" role="button"><div class="art-box">${art('plum')}</div><span style="flex:1"><b>未命名作品</b><small>还在和小伴一起做 · 昨天</small></span></div></div>`;
  }
  return `<div class="screen">${statusBar()}${top}<div class="scroll"><div class="pad">${body}<div class="nav-space"></div></div></div>${nav('plaza')}</div>`;
}
function list(s) {
  const map = {games:['精选游戏', GAMES], tools:['精选工具', TOOLS], picks:['创作者精选', PICKS]};
  const [title, items] = map[s.list];
  const shown = items.filter(a => s.filter === '全部' || (s.filter === '横屏') === (a.o === 'L'));
  const chipSet = ['全部', '竖屏', '横屏'];
  const chips = s.list === 'tools' ? '' : `<div class="chips">${chipSet.map(f => `<button class="chip ${s.filter === f ? 'on' : ''}" data-a="filter:${f}">${f}</button>`).join('')}</div>`;
  const content = s.list === 'tools' ? `<div class="tgrid" style="row-gap:16px">${shown.map(ttile).join('')}</div>`
    : s.list === 'picks' ? waterfall(shown)
    : `<div style="display:grid;grid-template-columns:repeat(3,1fr);gap:16px 10px">${shown.map(a => gcard(a).replace('class="gcard"', 'class="gcard" style="width:auto"')).join('')}</div>`;
  return `<div class="screen">${statusBar()}<div class="bar"><button class="ic" data-a="back" aria-label="返回">${icon('left')}</button><h1>${title}</h1><span class="ic"></span></div>
    <div class="scroll"><div class="pad">${chips}${content}<div style="height:40px"></div></div></div></div>`;
}
function post(s) {
  const a = A[s.post], liked = likes.has(a.id), fav = favs.has(a.id), mine = a.author === '我', fol = follows.has(a.author);
  const pos = ['', 'transform:scale(1.25);object-position:30% 60%', 'transform:scale(1.5);object-position:70% 30%'];
  const gallery = `<div class="gallery ${a.o === 'L' ? 'l' : 'p'}">${pos.map((p, i) => `<div style="${a.cover ? '' : `filter:hue-rotate(${i * 12}deg) brightness(${1 - i * .05})`}">${cover(a, p)}<span class="idx">${i + 1}/3</span></div>`).join('')}</div>`;
  return `<div class="screen post-page">${statusBar()}
    <div class="post-bar"><button class="ic" data-a="back" aria-label="返回">${icon('left')}</button><span class="who">${ava(a.author, 'lg')}${mine ? '我' : a.author}</span>
      ${mine ? '' : `<button class="follow ${fol ? 'done' : ''}" data-a="follow:${a.author}">${fol ? '已关注' : '关注'}</button>`}<button class="ic" data-a="pop" aria-label="更多">${icon('more')}</button></div>
    ${s.pop ? `<div class="menu-pop" style="top:96px"><button data-a="toast" data-msg="已复制链接">复制链接</button><button data-a="toast" data-msg="分享卡片">分享卡片</button>${mine ? '<button data-a="toast" data-msg="编辑帖子">编辑</button>' : '<button data-a="toast" data-msg="举报已提交">举报</button>'}</div>` : ''}
    <div class="scroll">${gallery}
      <div class="post-body"><h2>${a.name}</h2>${a.remixOf ? `<button class="from" data-a="post:${a.remixOf}">改编自 @${A[a.remixOf].author}《${short(A[a.remixOf])}》${icon('right')}</button>` : ''}<p>${a.desc}</p><div class="tags">${a.tags}</div>
        <div class="when">${a.time}<span>·</span>${a.kind} · ${oriText(a)}<span>·</span>${a.plays} 人玩过</div>
        <button class="big enter" style="margin-top:16px" data-a="launch:${a.id}">${icon(a.o === 'L' ? 'landscape' : 'play')}${a.o === 'L' ? '横屏进入' : '进入'}</button>
        ${mine ? `<button class="big grey" data-a="toast" data-msg="打开创作对话，让小伴接着改">${icon('pen')}继续修改</button>` : canRemix(a) ? `<button class="big grey" data-a="remix:${a.id}">${icon('pen')}做一个我的版本</button>` : ''}
        ${a.pick && a.note ? `<div class="editor"><b>小伴选</b>${a.note}</div>` : ''}
        ${a.remixes ? `<div class="card remix-row" data-a="remixes:${a.id}" role="button"><span class="stackav">${a.remixes.map(i => ava(A[i].author, 'md')).join('')}</span><span><b>${a.remixes.length} 个改编版本</b><small>大家在这个作品上做的新玩法</small></span>${icon('right')}</div>` : ''}
      </div>
      <div class="cmts"><h4>共 ${a.comments} 条评论</h4>${COMMENTS.map(([n, t, w, l]) => `<div class="cmt">${ava(n, 'md')}<div class="main"><div class="name">${n}</div><div class="txt">${t}</div><div class="meta">${w} · 回复</div></div><span class="lk">${icon('heart')}${l}</span></div>`).join('')}</div>
    </div>
    <div class="post-dock"><span class="say">说点什么…</span><button class="act ${liked ? 'on' : ''}" data-a="like:${a.id}">${icon('heart')}${a.likes + (liked ? 1 : 0)}</button><button class="act ${fav ? 'on' : ''}" data-a="fav:${a.id}">${icon('star')}收藏</button><button class="act" data-a="toast" data-msg="评论">${icon('comment')}${a.comments}</button></div></div>`;
}
function sheet(s) {
  const a = A[s.sheet];
  const pos = ['', 'transform:scale(1.3);object-position:30% 50%', 'transform:scale(1.6);object-position:70% 80%'];
  const shots = `<div class="shots ${a.o === 'L' ? 'l' : 'p'}">${pos.map((p, i) => `<div>${a.cover ? cover(a, p) : demoShot(a, i)}</div>`).join('')}</div>`;
  return `<div class="sheet-scrim" data-a="closesheet"></div><div class="sheet" role="dialog" aria-label="${a.name}"><button class="grab" data-a="closesheet" aria-label="收起"></button><div class="inner">
    <div class="head">${ico(a, 'lg')}<div style="min-width:0"><h2>${a.name}</h2><div class="ln"><span class="lbl">官方</span>${a.kind} · ${oriText(a)}</div></div><span class="more" style="display:flex"><button class="ic" data-a="toast" data-msg="分享：卡片、链接" aria-label="分享">${icon('share')}</button><button class="ic" data-a="pop" aria-label="更多">${icon('more')}</button></span></div>
    ${s.pop ? '<div class="menu-pop"><button data-a="toast" data-msg="已复制链接">复制链接</button><button data-a="toast" data-msg="反馈已提交">问题反馈</button></div>' : ''}
    ${shots}
    <button class="big" data-a="launch:${a.id}">${icon(a.o === 'L' ? 'landscape' : 'play')}${a.o === 'L' ? '横屏打开' : '打开'}</button>
    <div class="trio" style="grid-template-columns:1fr 1fr"><button data-a="fav:${a.id}" class="${favs.has(a.id) ? 'on' : ''}">${icon('star')}${favs.has(a.id) ? '已收藏' : '收藏'}</button><button data-a="remix:${a.id}">${icon('pen')}做一个我的版本</button></div>
    <p class="desc">${a.desc}</p>
    <div class="facts"><div><b>${a.plays}</b><small>人用过</small></div><div><b>${a.kind}</b><small>分类</small></div><div><b>${oriText(a)}</b><small>方向</small></div></div></div></div>`;
}
function demoShot(a, i) {
  if (a.id !== 'mbti') return cover(a);
  const pages = [['12 / 48', '在一场热闹的聚会上，你更常……'], ['30 / 48', '做一个新决定时，你更依赖……'], ['完成', 'INFJ']];
  const [p, q] = pages[i];
  return `<div class="art" style="background:linear-gradient(#f6f1e7,#efe7d8);padding:22px 10px;color:#3b352c;font-family:Songti SC,serif"><div style="font-size:8px;color:#8c8270;font-family:sans-serif">${p}</div><div style="font-size:${i === 2 ? 26 : 12}px;font-weight:600;margin-top:${i === 2 ? 60 : 22}px;line-height:1.5;text-align:${i === 2 ? 'center' : 'left'}">${q}</div>${i < 2 ? '<div style="margin-top:16px;height:26px;border-radius:7px;background:#fffaf0;border:1px solid #e6dccb"></div><div style="margin-top:6px;height:26px;border-radius:7px;background:#fffaf0;border:1px solid #e6dccb"></div>' : ''}</div>`;
}
function remixSheet(s) {
  const a = A[s.remix], game = a.kind === '游戏';
  const ideas = game ? ['换一个主题', '加一个新玩法', '改得简单一点', '加一个双人模式'] : ['换一种风格', '加一个功能', '只留我常用的'];
  return `<div class="sheet-scrim" data-a="closeremix"></div><div class="sheet" role="dialog" aria-label="做一个我的版本"><button class="grab" data-a="closeremix" aria-label="收起"></button><div class="inner">
    <h2 class="t-title" style="margin:0">做一个我的版本</h2>
    <div class="src-line">${ico(a, 'xs')}<span>基于《${short(a)}》 · ${byline(a)}</span></div>
    <textarea class="idea" placeholder="想怎么改？${game ? '比如：换成雪山，再加一个双人模式' : '比如：换成水墨风，每天早上提醒我'}">${esc(s.idea)}</textarea>
    <div class="sugs">${ideas.map(t => `<button class="chip" data-idea="${t}">${t}</button>`).join('')}</div>
    <div class="cost">${icon('sparkle')}<span>小伴先给你看方案，你确认后才开始做。预计约 30 点，今日剩余 86 点。</span></div>
    <button class="big" data-a="startremix">开始</button>
    <div class="t-cap" style="text-align:center;margin-top:10px">发布时会自动注明「改编自 ${byline(a)}」</div></div></div>`;
}
function remixChat(s) {
  const a = A[s.remixSrc], idea = s.idea.trim();
  return `<div class="me">做一个我的版本：《${short(a)}》${idea ? '<br>' + esc(idea) : ''}</div>
    <div class="bot">好，我先读了《${short(a)}》的源码。方案是这样：<br>· 保留原来的操作和手感<br>· ${idea ? '按你说的：' + esc(idea) : '换一个新主题，再加一个小彩蛋'}<br>· 做好先在手机上试玩，满意再发布，会注明改编自 ${byline(a)}<br>大约消耗 30 点，开始做吗？</div>
    <div class="sugs"><button class="pill" data-a="toast" data-msg="小伴开始制作">开始做</button><button class="chip" data-a="toast" data-msg="继续说你的想法">再改改方案</button></div>`;
}
function remixes(s) {
  const a = A[s.post], list = a.remixes.map(i => A[i]);
  return `<div class="screen">${statusBar()}<div class="bar"><button class="ic" data-a="back" aria-label="返回">${icon('left')}</button><h1>${list.length} 个改编版本</h1><span class="ic"></span></div>
    <div class="scroll"><div class="pad"><div class="card appcard" style="margin:4px 0 12px" data-a="back" role="button">${ico(a, 'sm')}<span style="flex:1;min-width:0"><b>原作《${short(a)}》</b><small>@${a.author} · ${a.plays} 人玩过</small></span>${canRemix(a) ? `<button class="pill" data-a="remix:${a.id}">${icon('pen')}我也改一个</button>` : ''}</div>
    ${waterfall(list)}<div class="endnote">没有更多了</div><div style="height:30px"></div></div></div></div>`;
}
function launchScreen(s) {
  const a = A[s.app];
  return `<div class="screen launch">${statusBar('position:absolute;top:0;left:0;right:0')}${ico(a, 'xl')}<b>${a.name.split(/[，：]/)[0]}</b>${a.o === 'L' ? `<div class="rot">${icon('portrait')}</div><small>横过手机</small>` : '<div class="dots"><i></i><i></i><i></i></div>'}</div>`;
}
function run(s) {
  const a = A[s.app], L = a.o === 'L';
  let content;
  if (a.id === 'mbti') {
    if (s.q < MBTI_Q.length) {
      const [q, x, y] = MBTI_Q[s.q];
      content = `<div class="mbti"><div class="prog">${12 + s.q} / 48<div><i style="width:${(12 + s.q) / 48 * 100}%"></i></div></div><h2>${q}</h2>
        <button class="opt ${s.sel === 0 ? 'sel' : ''}" data-a="answer:0">${x}</button><button class="opt ${s.sel === 1 ? 'sel' : ''}" data-a="answer:1">${y}</button>
        <div class="foot"><span>上一题</span><span>跳过</span></div></div>`;
    } else {
      content = `<div class="mbti"><div class="result"><small style="color:#8c8270">你的倾向</small><div class="type">INFJ</div><p>安静，但心里有很多想法。<br>你在意别人的感受，也需要独处的时间。<br>这只是倾向，不是标签。</p>
        <button class="opt" style="margin-top:28px;text-align:center" data-a="toast" data-msg="回到对话，小伴陪你聊这个结果">和小伴聊聊这个结果</button><button class="opt" style="text-align:center" data-a="restart">再测一次</button></div></div>`;
    }
  } else if (a.id === 'ink-blade') {
    content = `<div class="ink-run"><img src="${a.cover}" alt=""><div class="ink-hud">卷三 · 烽火<span>第四幅 · 铁骑将军</span></div>
      <div class="ink-btn" style="left:64px;bottom:30px;width:96px;height:96px;background:#2a201a30;box-shadow:inset 0 0 0 1.5px #2a201a50"></div>
      <div class="ink-btn" style="right:120px;bottom:30px;width:58px;height:58px;font-size:18px">闪</div><div class="ink-btn" style="right:36px;bottom:44px;width:74px;height:74px;font-size:24px;background:#9e2f25e0">斩</div></div>`;
  } else {
    content = `<div class="generic-run">${a.cover ? cover(a) : cover(a)}<div class="label">演示画面</div></div>`;
  }
  const safe = L
    ? `<div class="safe" style="left:0;top:0;bottom:0;width:52px"><em style="left:4px;bottom:8px">摄像头</em></div><div class="safe safe-capsule" style="right:8px;top:8px;height:44px"><em style="right:0;top:48px">宿主胶囊</em></div>`
    : `<div class="safe" style="left:0;right:0;top:0;height:44px"><em style="left:8px;top:48px">状态栏</em></div><div class="safe safe-capsule" style="right:6px;top:44px;height:42px"><em style="right:0;top:46px">宿主胶囊</em></div>`;
  const fav = favs.has(a.id);
  const menu = s.menu ? `<div class="run-menu" data-a="closemenu"><div class="panel" data-stop>
    <div class="who">${ico(a, 'sm')}<span><b>${a.name.split(/[，：]/)[0]}</b><small>${a.src === 'official' ? '<span class="lbl">官方</span>' : a.author + ' ·'} ${a.kind}</small></span></div>
    <div class="grid"><button data-a="restart"><span>${icon('refresh')}</span>重新开始</button><button data-a="fav:${a.id}" class="${fav ? 'on' : ''}"><span>${icon('star')}</span>${fav ? '已收藏' : '收藏'}</button><button data-a="toast" data-msg="分享：卡片、链接"><span>${icon('share')}</span>分享</button><button data-a="toast" data-msg="${a.src === 'official' ? '反馈已提交' : '举报已提交'}"><span>${icon('flag')}</span>${a.src === 'official' ? '反馈' : '举报'}</button></div>
    ${canRemix(a) ? `<button class="exit remixbtn" data-a="remix:${a.id}">${icon('pen')}做一个我的版本</button>` : ''}<button class="exit" data-a="exit">退出</button>${s.armed ? '<span class="hint">再返回一次也会退出</span>' : ''}</div></div>` : '';
  return `<div class="screen run">${L ? '' : statusBar(a.id === 'mbti' ? 'color:#3b352c' : 'color:#fff')}<div class="app-frame">${content}</div>${safe}
    <div class="capsule"><button data-a="menu" aria-label="更多">${icon('more')}</button><i></i><button data-a="exit" aria-label="退出">${icon('close')}</button></div>${menu}</div>`;
}
init();
