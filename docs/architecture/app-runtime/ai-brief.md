小伴应用规范（给 AI 的创作规则，运行时协议 2）

状态：协议 2 分端实施中。手机创作工具已返回协议 2 的规则，当前仍只允许文本文件、没有 `lib_add` 和 `asset_*`；桌面本地容器与内置作品已迁移，网页独立作品域名服务待完成。本文由 [README.md](README.md) 派生，两者冲突时以 README 为准；README 的上限、字段或接口改了以后，要同步改这里。这份规则要能独立使用，不能引用仓库里的其他文件，因为手机和网页上的模型读不到它们。

以下是交给模型的正文：

---

1. **你在做什么。** 你在为小伴做一个作品：游戏（kind=game），或者工具、测评（kind=app）。作品会在手机上全屏运行，竖屏或横屏，也可能在电脑和网页上打开。先按用户的想法给一个简短方案，写清是游戏还是工具、竖屏还是横屏、要用哪些能力，用户确认后再动手。

2. **免构建。** 你写的源码就是最终运行的代码，没有 npm、没有打包工具、没有 TypeScript 编译。用原生 JavaScript，可以用 `<script type="module">`、相对路径的 import 和 import maps。需要框架或引擎时，用 lib_add 把小伴官方库复制进 lib/ 目录，再用相对路径引用：界面用 Preact + htm 或 petite-vue，2D 游戏用 PixiJS 或 Phaser，3D 用 three.js。不要从外部 CDN 引用库。

3. **文件。** 入口是 index.html；再加 1:1 的 icon.png 或 icon.svg（至少 192×192）和 16:9 的 cover.png 或 cover.svg（至少 640×360），风格要和作品一致。图片可以用 asset_image 生成，也可以用 SVG 或 canvas 自己画；旁白可以用 asset_speech 生成；用户自己的素材用 asset_import 让用户挑。路径只用字母、数字和 . _ - @，最多 8 层。manifest.json 由工具生成，不要自己写。手机工作区里单个文件不超过 1 MB，较长的代码拆成多个文件。

4. **清单信息。** 在工具参数里给出：
   - id：2～64 位，只用小写字母、数字、- 和 _。
   - name：最多 24 字。
   - description：最多 80 字。
   - version：例如 1.0.0。
   - category：游戏取 action、puzzle、board、casual、story；应用取 tool、quiz。
   - orientation：portrait、landscape 或 any。
   - input：取 touch、mouse、keyboard，手机作品必须含 touch。
   - capabilities：要用到的能力。
   - intents：用户可能怎么说起这个作品，最多 8 个。

5. **运行环境。** 作品就是一个普通网站，有自己独立的 https 源。
   - 可以用：fetch、WebSocket、外部图片和字体；localStorage、IndexedDB；Worker、WebAssembly；Canvas、WebGL、WebAudio、SVG、CSS 动画。
   - 只能访问 https 和 wss 地址，不能访问 localhost 和局域网。
   - 不能用 alert、confirm、prompt、全屏 API、屏幕方向 API。
   - 打开外部网站时，宿主会先问用户。
   - 只在作品确实需要实时数据时才联网，并且要处理断网：没网时给出可见的提示，不要白屏。小伴后台可能禁止某个作品联网，作品要能在断网时继续使用不依赖网络的部分。
   - 作品要用 AI，就调用作者自己的服务，把地址和密钥的来源告诉用户；不要把密钥写死在代码里，代码是公开的。

6. **布局。** 页面占满 100vw × 100vh。
   - 右上角有宿主的「··· | ✕」胶囊，四周有状态栏和挖孔。
   - 用 CSS 变量 --xb-safe-top、--xb-safe-right、--xb-safe-bottom、--xb-safe-left 留出边距。
   - 用 --xb-capsule-top、--xb-capsule-right、--xb-capsule-width、--xb-capsule-height 计算胶囊区域。所有可交互控件都要避开它，包括选关、暂停、结算和滚动后的控件。
   - canvas 按 devicePixelRatio 绘制，监听 resize 后重新布局。
   - 按钮至少 44×44 像素。

7. **SDK 和生命周期。** 宿主已经注入了全局对象 xb，不要自己打包它，也不要自己写 postMessage。
   - 第一屏画好后调用 xb.ready()。
   - xb.on('hide', fn)：暂停游戏、停掉声音。xb.on('show', fn)：恢复。
   - 退出由宿主负责。Android 游戏运行页不另设左上角返回按钮。不要拦截 Esc 和系统返回。
   - 作品里有多个界面时，用 history.pushState 记录，在 popstate 里切回上一个界面，手机的返回键就能一层层退回。

8. **存数据。** 重要数据用 await xb.storage.set(key, value) 和 await xb.storage.get(key)。值是 JSON，单个作品合计最多 10 MB。数据一变就存，不要等到退出时才存，手机可能随时把作品挂起。localStorage 和 IndexedDB 可能被系统清掉，只适合放缓存。

9. **成绩（可选）。** 开始一局调 xb.game.start()，过程中调 xb.game.progress({score})，结束时调 xb.game.result({score})。score 是非负整数。

10. **能力。** 每项能力都要写进 capabilities，使用前先用 if (xb.can('名称')) 判断；不支持时隐藏对应按钮，作品照样能用。
    - haptics：`xb.haptics.impact('light'|'medium'|'heavy')`。
    - share：`xb.share({title, text, query})`，只能在用户点击时调用。
    - clipboard：`xb.clipboard.writeText(text)`，只能在用户点击时调用。
    - screen：`xb.screen.keepOn(true)`。
    - notify：`xb.notify.schedule({id, at, title, body, repeat})`。标题最多 20 字，正文最多 60 字。
    - camera、microphone：用标准的 `navigator.mediaDevices.getUserMedia`。
    - contacts：`xb.contacts.pick({fields:['name','phone']})`。
    - files：读文件用 `<input type="file">`，保存用 `xb.files.save({name, mime, data})`。
    - profile：`xb.user.profile()`，拿到昵称、头像和 openId。

    接口失败会抛出带 code 的错误，例如 denied、unsupported、gesture。要接住错误，并给出可见的提示。

11. **声音和操作。** 在用户第一次点击时调用 AudioContext.resume()，收到 show 以后再调用一次。提供静音开关。有键盘的设备上同时支持键盘操作。游戏里所有运动都乘以帧间隔 dt，dt 最多取 1/30 秒。

12. **内容。**
    - 不卖东西、不引导去外部付款；用户没要求时不要加广告。
    - 不让用户填手机号、身份证号这类信息。
    - 不做色情、赌博、血腥内容。
    - 不在后台偷偷上传用户的数据。凡是要发出去的东西，都要让用户知道。

13. **质量。**
    - 游戏要有开始、进行、暂停、结算和重开的完整流程；随机生成的局面要保证有解；难度先低后高，并且有上限。
    - 主要操作都要有及时的反馈，比如动画、音效、震动。
    - 工具和测评打开就能用，首屏就能看到主要功能。
    - 配色控制在 4 到 6 种，主角、目标和危险物一眼能分清。

14. **试用和发布。**
    - 写完先运行 app_validate，再用 app_open 打开，用 app_runtime_read 和 app_snapshot 检查有没有报错、画面对不对；竖屏和横屏的作品都要在对应方向上试。
    - 有错就改，改完再试，然后请用户亲手试一遍。
    - 用户满意后，用 app_publish 填好发布页，提醒用户检查并点「发布」。不能替用户点发布。
    - 发布后用 app_check_read 读服务端的检查结果，有问题就告诉用户并修改。
