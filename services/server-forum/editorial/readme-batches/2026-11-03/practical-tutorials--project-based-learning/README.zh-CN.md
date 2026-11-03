# 项目式学习

[![Gitter](https://badges.gitter.im/practical-tutorials/community.svg)](https://gitter.im/practical-tutorials/community?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge) [![Link rot sweep](https://github.com/practical-tutorials/project-based-learning/actions/workflows/link-rot.yml/badge.svg)](https://github.com/practical-tutorials/project-based-learning/actions/workflows/link-rot.yml)

一份编程教程清单，帮助有志成为软件开发者的人学习如何从零构建应用。教程按主要编程语言分类，也可能涉及多种技术和语言。

开始使用时只需 Fork 本仓库。贡献指南请参阅 [CONTRIBUTING.md](https://github.com/practical-tutorials/project-based-learning/blob/2146917314b5e98e58513c930c0a745cc6c7100c/CONTRIBUTING.md)。

## 目录：

- [C#](#c)
- [C/C++](#cc)
- [Clojure](#clojure)
- [Dart](#dart)
- [Elixir](#elixir)
- [Erlang](#erlang)
- [F#](#f)
- [Go](#go)
- [Haskell](#haskell)
- [HTML/CSS](#html-and-css)
- [Java](#java)
- [JavaScript](#javascript)
- [Kotlin](#kotlin)
- [Lua](#lua)
- [OCaml](#ocaml)
- [PHP](#php)
- [Python](#python)
- [R](#r)
- [Ruby](#ruby)
- [Rust](#rust)
- [Scala](#scala)
- [Swift](#swift)
- [其他资源](#additional-resources)

## C/C++:

- [构建解释器](http://www.craftinginterpreters.com/) （第 14 章起使用 C 编写）
- [内存分配器入门：编写简单内存分配器](https://arjunsreedharan.org/post/148675821737/memory-allocators-101-write-a-simple-memory)
- [用 C 编写 Shell](https://brennan.io/2015/01/16/write-a-shell-in-c/)
- [编写 FUSE 文件系统](https://www.cs.nmsu.edu/~pfeiffer/fuse-tutorial/)
- [构建自己的文本编辑器](http://viewsourcecode.org/snaptoken/kilo/)
- [构建自己的 Lisp](http://www.buildyourownlisp.com/)
- [如何使用 C 编写 NES 游戏](https://nesdoug.com/)
- [从零编写操作系统](https://github.com/tuhdo/os01)
- [如何从零创建操作系统](https://github.com/cfenollosa/os-tutorial)
- [构建 CHIP-8 模拟器](https://austinmorlan.com/posts/chip8_emulator/)
- [使用 C++ 和 SDL 的游戏编程入门](http://lazyfoo.net/tutorials/SDL/)
- [实现键值存储](http://codecapsule.com/2012/11/07/ikvs-implementing-a-key-value-store-table-of-contents/)
- 小型 3D 图形项目
  - [Tiny Renderer：OpenGL 如何工作，500 行代码实现软件渲染](https://github.com/ssloy/tinyrenderer/wiki)
  - [256 行纯 C++ 实现易懂的光线追踪](https://github.com/ssloy/tinyraytracer/wiki)
  - [180 行纯 C++ 实现 KABOOM！](https://github.com/ssloy/tinykaboom/wiki)
  - [486 行 C++：一周末实现复古 FPS](https://github.com/ssloy/tinyraycaster/wiki)
- 使用 C++ 编写最小 x86-64 JIT 编译器
  - [第 1 部分](https://solarianprogrammer.com/2018/01/10/writing-minimal-x86-64-jit-compiler-cpp/)
  - [第 2 部分](https://solarianprogrammer.com/2018/01/12/writing-minimal-x86-64-jit-compiler-cpp-part-2/)
- [为 C++ 构建代码热重载库](http://howistart.org/posts/cpp/1/index.html)
- [用 C 编写哈希表](https://github.com/jamesroutley/write-a-hash-table)
- [一起构建简单数据库](https://cstack.github.io/db_tutorial/)
- [一起编写内核](http://arjunsreedharan.org/post/82710718100/kernel-101-lets-write-a-kernel)
- [用 C 编写引导加载程序](http://3zanders.co.uk/2017/10/13/writing-a-bootloader/)
- [用 500 行代码实现 Linux 容器](https://blog.lizzie.io/linux-containers-in-500-loc.html)
- [编写自己的虚拟机](https://justinmeiners.github.io/lc3-vm/)
- [学习 KVM：实现自己的 Linux 内核](https://david942j.blogspot.com/2018/10/note-learning-kvm-implement-your-own.html)
- [使用 C/C++ 构建自己的 Redis](https://build-your-own.org/redis/)
- 编写 C 编译器
  - [第 1 部分：整数、词法分析和代码生成](https://norasandler.com/2017/11/29/Write-a-Compiler.html)
  - [第 2 部分：一元运算符](https://norasandler.com/2017/12/05/Write-a-Compiler-2.html)
  - [第 3 部分：二元运算符](https://norasandler.com/2017/12/15/Write-a-Compiler-3.html)
  - [第 4 部分：更多二元运算符](https://norasandler.com/2017/12/28/Write-a-Compiler-4.html)
  - [第 5 部分：局部变量](https://norasandler.com/2018/01/08/Write-a-Compiler-5.html)
  - [第 6 部分：条件语句](https://norasandler.com/2018/02/25/Write-a-Compiler-6.html)
  - [第 7 部分：复合语句](https://norasandler.com/2018/03/14/Write-a-Compiler-7.html)
  - [第 8 部分：循环](https://norasandler.com/2018/04/10/Write-a-Compiler-8.html)
  - [第 9 部分：函数](https://norasandler.com/2018/06/27/Write-a-Compiler-9.html)
  - [第 10 部分：全局变量](https://norasandler.com/2019/02/18/Write-a-Compiler-10.html)
- [使用 LLVM 实现一门语言](https://llvm.org/docs/tutorial/#kaleidoscope-implementing-a-language-with-llvm)
- [Meta Crush Saga：C++17 编译期游戏](https://jguegant.github.io//jguegant.github.io/blogs/tech/meta-crush-saga.html)
- [高性能矩阵乘法](https://gist.github.com/nadavrot/5b35d44e8ba3dd718e595e40184d03f0)
- 从零制作太空侵略者
  - [第 1 部分](http://nicktasios.nl/posts/space-invaders-from-scratch-part-1.html)
  - [第 2 部分](http://nicktasios.nl/posts/space-invaders-from-scratch-part-2.html)
  - [第 3 部分](http://nicktasios.nl/posts/space-invaders-from-scratch-part-3.html)
  - [第 4 部分](http://nicktasios.nl/posts/space-invaders-from-scratch-part-4.html)
  - [第 5 部分](http://nicktasios.nl/posts/space-invaders-from-scratch-part-5.html)
- [跨平台 C++ 俄罗斯方块教程](http://javilop.com/gamedev/tetris-tutorial-in-c-platform-independent-focused-in-game-logic-for-beginners/)
- 编写 Linux 调试器
  - [第 1 部分：设置](https://blog.tartanllama.xyz/writing-a-linux-debugger-setup/)
  - [第 2 部分：断点](https://blog.tartanllama.xyz/writing-a-linux-debugger-breakpoints/)
  - [第 3 部分：寄存器和内存](https://blog.tartanllama.xyz/writing-a-linux-debugger-registers/)
  - [第 4 部分：ELF 与 DWARF](https://blog.tartanllama.xyz/writing-a-linux-debugger-elf-dwarf/)
  - [第 5 部分：源代码与信号](https://blog.tartanllama.xyz/writing-a-linux-debugger-source-signal/)
  - [第 6 部分：源码级单步执行](https://blog.tartanllama.xyz/writing-a-linux-debugger-dwarf-step/)
  - [第 7 部分：源码级断点](https://blog.tartanllama.xyz/writing-a-linux-debugger-source-break/)
  - [第 8 部分：栈展开](https://blog.tartanllama.xyz/writing-a-linux-debugger-unwinding/)
  - [第 9 部分：处理变量](https://blog.tartanllama.xyz/writing-a-linux-debugger-variables/)
  - [第 10 部分：高级主题](https://blog.tartanllama.xyz/writing-a-linux-debugger-advanced-topics/)
- 一起编写编译器
  - [第 1 部分：介绍、选择语言并制定计划](https://briancallahan.net/blog/20210814.html)
  - [第 2 部分：词法分析器](https://briancallahan.net/blog/20210815.html)
  - [第 3 部分：解析器](https://briancallahan.net/blog/20210816.html)
  - [第 4 部分：测试](https://briancallahan.net/blog/20210817.html)
  - [第 5 部分：代码生成器](https://briancallahan.net/blog/20210818.html)
  - [第 6 部分：输入和输出](https://briancallahan.net/blog/20210819.html)
  - [第 7 部分：数组](https://briancallahan.net/blog/20210822.html)
  - [第 8 部分：字符串、前向引用和总结](https://briancallahan.net/blog/20210826.html)

### 网络编程

- 一起编写 TCP/IP 协议栈

  - [第 1 部分：以太网与 ARP](http://www.saminiir.com/lets-code-tcp-ip-stack-1-ethernet-arp/)
  - [第 2 部分：IPv4 与 ICMPv4](http://www.saminiir.com/lets-code-tcp-ip-stack-2-ipv4-icmpv4/)
  - [第 3 部分：TCP 基础和握手](http://www.saminiir.com/lets-code-tcp-ip-stack-3-tcp-handshake/)
  - [第 4 部分：TCP 数据流与 Socket API](http://www.saminiir.com/lets-code-tcp-ip-stack-4-tcp-data-flow-socket-api/)
  - [第 5 部分：TCP 重传](http://www.saminiir.com/lets-code-tcp-ip-stack-5-tcp-retransmission/)

- 编写并发服务器

  - [第 1 部分：介绍](https://eli.thegreenplace.net/2017/concurrent-servers-part-1-introduction/)
  - [第 2 部分：线程](https://eli.thegreenplace.net/2017/concurrent-servers-part-2-threads/)
  - [第 3 部分：事件驱动](https://eli.thegreenplace.net/2017/concurrent-servers-part-3-event-driven/)
  - [第 4 部分：libuv](https://eli.thegreenplace.net/2017/concurrent-servers-part-4-libuv/)
  - [第 5 部分：Redis 案例研究](https://eli.thegreenplace.net/2017/concurrent-servers-part-5-redis-case-study/)
  - [第 6 部分：回调、Promise 和 async/await](https://eli.thegreenplace.net/2018/concurrent-servers-part-6-callbacks-promises-and-asyncawait/)

- 从零构建 MQTT 代理
  - [第 1 部分：协议](https://codepr.github.io/posts/sol-mqtt-broker)
  - [第 2 部分：网络](https://codepr.github.io/posts/sol-mqtt-broker-p2)
  - [第 3 部分：服务器](https://codepr.github.io/posts/sol-mqtt-broker-p3)
  - [第 4 部分：数据结构](https://codepr.github.io/posts/sol-mqtt-broker-p4)
  - [第 5 部分：主题抽象](https://codepr.github.io/posts/sol-mqtt-broker-p5)
  - [第 6 部分：处理器](https://codepr.github.io/posts/sol-mqtt-broker-p6)
  - [补充：多线程](https://codepr.github.io/posts/sol-mqtt-broker-bonus)
- [从零实现网络](https://github.com/TanayK07/networking-from-scratch)

### OpenGL:

- 使用 C++ 和 OpenGL 创建 2D 打砖块游戏克隆
  - [打砖块游戏](https://learnopengl.com/In-Practice/2D-Game/Breakout)
  - [设置](https://learnopengl.com/In-Practice/2D-Game/Setting-up)
  - [渲染精灵](https://learnopengl.com/In-Practice/2D-Game/Rendering-Sprites)
  - [关卡](https://learnopengl.com/In-Practice/2D-Game/Levels)
  - 碰撞
    - [球](https://learnopengl.com/In-Practice/2D-Game/Collisions/Ball)
    - [碰撞检测](https://learnopengl.com/In-Practice/2D-Game/Collisions/Collision-detection)
    - [碰撞处理](https://learnopengl.com/In-Practice/2D-Game/Collisions/Collision-resolution)
  - [粒子](https://learnopengl.com/In-Practice/2D-Game/Particles)
  - [后期处理](https://learnopengl.com/In-Practice/2D-Game/Postprocessing)
  - [强化道具](https://learnopengl.com/In-Practice/2D-Game/Powerups)
  - [音频](https://learnopengl.com/In-Practice/2D-Game/Audio)
  - [渲染文本](https://learnopengl.com/In-Practice/2D-Game/Render-text)
  - [总结](https://learnopengl.com/In-Practice/2D-Game/Final-thoughts)
- [Handmade Hero 游戏制作](https://handmadehero.org)
- [如何使用 C++／OpenGL 制作 Minecraft](https://www.youtube.com/playlist?list=PLMZ_9w2XRxiZq1vfw1lrpCMRDufe2MKV_) （视频）

## C#:

- [通过构建简单角色扮演游戏学习 C#](http://scottlilly.com/learn-c-by-building-a-simple-rpg-index/)
- [使用 C# 创建 Roguelike 游戏](https://roguesharp.wordpress.com/)
- [使用 C# 和 Xamarin 创建空白应用（进行中）](https://www.intertech.com/Blog/xamarin-tutorial-part-1-create-a-blank-app/)
- [使用 Xamarin 和 Visual Studio 构建 iOS 相册应用](https://www.raywenderlich.com/134049/building-ios-apps-with-xamarin-and-visual-studio)
- [构建 CoreWiki](https://www.youtube.com/playlist?list=PLVMqA0_8O85yC78I4Xj7z48ES48IQBa7p) 这是使用 C#、ASP.NET Core 和 Razor Pages 完全编写的 Wiki 风格内容管理系统。源代码见[此处](https://github.com/csharpfritz/CoreWiki).

## Clojure:

- [使用 Clojure 构建 Twitter 机器人](http://howistart.org/posts/clojure/1/index.html)
- [构建拼写检查器](https://bernhardwenzel.com/articles/clojure-spellchecker/)
- [使用 Clojure 和 Atlassian Connect 构建 JIRA 集成](https://hackernoon.com/building-a-jira-integration-with-clojure-atlassian-connect-506ebd112807)
- [使用 Clojure 制作原型](https://github.com/aliaksandr-s/prototyping-with-clojure)
- [用 ClojureScript 制作俄罗斯方块](https://shaunlebron.github.io/t3tr0s-slides)

## Dart:

### Flutter:

- [带管理面板的 Amazon 克隆应用](https://youtu.be/O3nmP-lZAdg)
- [外卖应用](https://youtu.be/7dAt-JMSCVQ)
- [Google Docs 克隆应用](https://youtu.be/0_GJ1w_iG44)
- [Instagram 克隆应用](https://youtu.be/mEPm9w5QlJM)
- [多人井字棋游戏](https://youtu.be/Aut-wfXacXg)
- [TikTok 克隆应用](https://youtu.be/4E4V9F3cbp4)
- [票务预订应用](https://youtu.be/71AsYo2q_0Y)
- [旅行应用](https://youtu.be/x4DydJKVvQk)
- [Twitch 克隆应用](https://youtu.be/U9YKZrDX0CQ)
- [WhatsApp 克隆应用](https://youtu.be/yqwfP2vXWJQ)
- [Wordle 克隆应用](https://youtu.be/_W0RN_Cqhpg)
- [Zoom 克隆应用](https://youtu.be/sMA1dKbv33Y)
- [Netflix 克隆应用](https://youtu.be/J8IFNKzs3TI)

## Elixir

- [使用 Elixir 和 Phoenix 构建简单聊天应用](https://sheharyar.me/blog/simple-chat-phoenix-elixir/)
- [如何用 Elixir、Phoenix 和 Mnesia 编写超快速链接缩短器](https://medium.com/free-code-camp/how-to-write-a-super-fast-link-shortener-with-elixir-phoenix-and-mnesia-70ffa1564b3c)

## Erlang

- [ChatBus：使用 Erlang/OTP 构建第一个多人聊天室应用](https://medium.com/@kansi/chatbus-build-your-first-multi-user-chat-room-app-with-erlang-otp-b55f72064901)
- [使用 Erlang、Rebar、Cowboy 和 Bullet 制作聊天应用](https://marianoguerra.org/posts/making-a-chat-app-with-erlang-rebar-cowboy-and-bullet/)

## F#:

- [用 100 行 F# 编写自己的 Excel](http://tomasp.net/blog/2018/write-your-own-excel)

## Java:

- [构建解释器](http://www.craftinginterpreters.com/) （第 4–13 章使用 Java 编写）
- [使用 Java 构建简单 HTTP 服务器](http://javarevisited.blogspot.com/2015/06/how-to-create-http-server-in-java-serversocket-example.html)
- [构建 Android 手电筒应用](https://www.youtube.com/watch?v=dhWL4DC7Krs) （视频）
- [构建带用户认证的 Spring Boot 应用](https://spring.io/guides/gs/securing-web/)

## JavaScript:

- [通过 30 个教程在 30 天里构建 30 件作品](https://javascript30.com)
- [构建 Jupyter Notebook 扩展](https://link.medium.com/wWUO7TN8SS)
- [使用原生 JavaScript 构建简单天气应用](https://webdesign.tutsplus.com/tutorials/build-a-simple-weather-app-with-vanilla-javascript--cms-33893)
- [使用 JavaScript 构建待办事项应用](https://github.com/dwyl/javascript-todo-list-tutorial)

## HTML and CSS:

- [构建加载界面](https://medium.freecodecamp.org/how-to-build-a-delightful-loading-screen-in-5-minutes-847991da509f)
- [使用 JS 构建 HTML 计算器](https://medium.freecodecamp.org/how-to-build-an-html-calculator-app-from-scratch-using-javascript-4454b8714b98)
- [仅用 JavaScript、HTML 和 CSS 构建贪吃蛇](https://www.freecodecamp.org/news/think-like-a-programmer-how-to-build-snake-using-only-javascript-html-and-css-7b1479c3339e/)

### 移动应用：

- [构建 React Native 待办事项应用](https://egghead.io/courses/build-a-react-native-todo-application)
- [使用 Redux Thunk 构建 React Native 应用](https://medium.com/@alialhaddad/how-to-use-redux-thunk-in-react-and-react-native-4743a1321bd0)

### Web 应用：

#### React:

- [创建无服务器 React.js 应用](http://serverless-stack.com/)
- [创建 Trello 克隆应用](http://codeloveandboards.com/blog/2016/01/04/trello-tribute-with-phoenix-and-react-pt-1/)
- [使用 React、Node、MongoDB 和 SocketIO 创建角色投票应用](http://sahatyalkabov.com/create-a-character-voting-app-using-react-nodejs-mongodb-and-socketio)
- [React 教程：克隆 Yelp](https://www.fullstackreact.com/articles/react-tutorial-cloning-yelp/)
- [使用 Mocha、React、Redux 和 Immutable，通过测试优先开发构建全栈电影投票应用](https://teropa.info/blog/2015/09/10/full-stack-redux-tutorial.html)
- [使用 React.js 和 Node.js 构建简单 Medium 克隆应用](https://medium.com/@kris101/clone-medium-on-node-js-and-react-js-731cdfbb6878)
- [在 JS 中集成 MailChimp](https://medium.freecodecamp.org/how-to-integrate-mailchimp-in-a-javascript-web-app-2a889fb43f6f)
- [使用 React 和 Parcel 构建 Chrome 扩展](https://medium.freecodecamp.org/building-chrome-extensions-in-react-parcel-79d0240dd58f)
- [使用 React Native 构建待办事项应用](https://blog.hasura.io/tutorial-fullstack-react-native-with-graphql-and-authentication-18183d13373a)
- [制作聊天应用](https://medium.freecodecamp.org/how-to-build-a-chat-application-using-react-redux-redux-saga-and-web-sockets-47423e4bc21a)
- [使用 React Native 创建新闻应用](https://medium.freecodecamp.org/create-a-news-app-using-react-native-ced249263627)
- [学习用于 React 的 Webpack](https://medium.freecodecamp.org/learn-webpack-for-react-a36d4cac5060)
- [使用 Puppeteer 和 Jest 测试 React 应用](https://blog.bitsrc.io/testing-your-react-app-with-puppeteer-and-jest-c72b3dfcde59)
- [构建自己的 React 样板代码](https://medium.freecodecamp.org/how-to-build-your-own-react-boilerplate-2f8cbbeb9b3f)
- [使用 React 编写生命游戏](https://medium.freecodecamp.org/create-gameoflife-with-react-in-one-hour-8e686a410174)
- [React+Redux 基础入门教程](https://hackernoon.com/a-basic-react-redux-introductory-tutorial-adcc681eeb5e)
- [构建预约排程器](https://hackernoon.com/build-an-appointment-scheduler-using-react-twilio-and-cosmic-js-95377f6d1040)
- [构建带情感分析的聊天应用](https://codeburst.io/build-a-chat-app-with-sentiment-analysis-using-next-js-c43ebf3ea643)
- [搭建全栈 Web 应用](https://hackernoon.com/full-stack-web-application-using-react-node-js-express-and-webpack-97dbd5b9d708)
- [使用 React 和 Firebase 创建 Todoist 克隆应用](https://www.youtube.com/watch?v=hT3j87FMR6M)
- 构建随机名言生成器
  - [第 1 部分](https://www.youtube.com/watch?v=3QngsWA9IEE)
  - [第 2 部分](https://www.youtube.com/watch?v=XnoTmO06OYo)
  - [第 3 部分](https://www.youtube.com/watch?v=us51Jne67_I)
  - [第 4 部分](https://www.youtube.com/watch?v=iZx7hqHb5MU)
  - [第 5 部分](https://www.youtube.com/watch?v=lpba9vBqXl0)
  - [第 6 部分](https://www.youtube.com/watch?v=Jvp8j6zrFHE)
  - [第 7 部分](https://www.youtube.com/watch?v=M_hFfrN8_PQ)
- [React 手机电商项目（视频）](https://www.youtube.com/watch?v=-edmQKcOW8s)

#### Next.js:

- [App Router](https://nextjs.org/learn/dashboard-app)

#### Angular:

- [使用 Angular 1.x 构建 Instagram 克隆应用](https://hackhands.com/building-instagram-clone-angularjs-satellizer-nodejs-mongodb/)
- 使用 Angular 2+ 构建可离线使用的 Hacker News 客户端
  - [第 1 部分](https://houssein.me/angular2-hacker-news)
  - [第 2 部分](https://houssein.me/progressive-angular-applications)
- [使用 Django 和 AngularJS（Angular 1.x）构建 Google+ 克隆应用](https://thinkster.io/django-angularjs-tutorial)
- 使用 Angular 8 构建美观的真实应用：

  - [第 1 部分](https://medium.com/@hamedbaatour/build-a-real-world-beautiful-web-app-with-angular-6-a-to-z-ultimate-guide-2018-part-i-e121dd1d55e)
  - [第 2 部分](https://medium.com/@hamedbaatour/build-a-real-world-beautiful-web-app-with-angular-8-the-ultimate-guide-2019-part-ii-fe70852b2d6d)

- [使用 Bootstrap 4 和 Angular 6 构建响应式布局](https://medium.com/@tomastrajan/how-to-build-responsive-layouts-with-bootstrap-4-and-angular-6-cfbb108d797b)
- 使用 Angular 5 的待办事项应用
  - [Angular 入门](http://www.discoversdk.com/blog/intro-to-angular-and-the-evolution-of-the-web)
  - [第 1 部分](http://www.discoversdk.com/blog/angular-5-to-do-list-app-part-1)

#### Node:

- [使用 Node、Postgres 和 Knex 进行测试驱动开发](http://mherman.org/blog/2016/04/28/test-driven-development-with-node/)
- 使用 Node.js 编写 Twitter 机器人
  - [第 1 部分](https://codeburst.io/build-a-simple-twitter-bot-with-node-js-in-just-38-lines-of-code-ed92db9eb078)
  - [第 2 部分](https://codeburst.io/build-a-simple-twitter-bot-with-node-js-part-2-do-more-2ef1e039715d)
- [30 分钟内构建简单搜索机器人](https://medium.freecodecamp.org/how-to-build-a-simple-search-bot-in-30-minutes-eb56fcedcdb1)
- [构建职位抓取 Web 应用](https://medium.freecodecamp.org/how-i-built-a-job-scraping-web-app-using-node-js-and-indreed-7fbba124bbdc)
- [构建 GitHub 应用](https://blog.scottlogic.com/2017/05/22/gifbot-github-integration.html)
- 如何使用 JavaScript、Node.JS、MongoDB 和 WebSocket 构建自己的 Uber-for-X 应用
  - [第 1 部分](https://www.ashwinhariharan.tech/blog/how-to-build-your-own-uber-for-x-app/)
  - [第 2 部分](https://www.ashwinhariharan.tech/blog/how-to-build-your-own-uber-for-x-app-part-2/)

#### Vue

- [Vue 2 + Firebase：15 分钟内构建带 Firebase 身份认证的 Vue 应用](https://medium.com/@anas.mammeri/vue-2-firebase-how-to-build-a-vue-app-with-firebase-authentication-system-in-15-minutes-fdce6f289c3c)
- [Vue.js 应用教程：创建简单预算应用](https://matthiashager.com/complete-vuejs-application-tutorial/)
- [使用 Vue、GraphQL 和 Apollo 构建博客](https://www.digitalocean.com/community/tutorials/how-to-build-a-blog-with-vue-graphql-and-apollo-client)
- 使用 MEVN（MongoDB、Express、Vue、Node）技术栈构建全栈 Web 应用
  - [第 1 部分](https://medium.com/@anaida07/mevn-stack-application-part-1-3a27b61dcae0)
  - [第 2 部分](https://medium.com/@anaida07/mevn-stack-application-part-2-2-9ebcf8a22753)
- [Vue.js 待办事项列表教程（视频）](https://www.youtube.com/watch?v=78tNYZUS-ps)
- [Vue 2 + Pub/Sub：为游戏构建点对点多用户平台](https://www.ably.io/tutorials/peer-to-peer-vue)

#### 其他（Hapi、Express 等）：

- 构建渐进式 Web 应用（PWA）
  - [第 1 部分](https://bitsofco.de/bitsofcode-pwa-part-1-offline-first-with-service-worker/)
  - [第 2 部分](https://bitsofco.de/bitsofcode-pwa-part-2-instant-loading-with-indexeddb/)
  - [第 3 部分](https://bitsofco.de/bitsofcode-pwa-part-3-push-notifications/)
- [使用 JS 构建原生桌面应用](https://medium.freecodecamp.org/build-native-desktop-apps-with-javascript-a49ede90d8e9)
- 使用 NodeJs、GraphQL 和 Hapi 构建强大的 API
  - [第 1 部分](https://medium.com/@wesharehoodies/how-to-setup-a-powerful-api-with-nodejs-graphql-mongodb-hapi-and-swagger-e251ac189649)

#### D3.js

- [通过实例学习 D3](https://www.sitepoint.com/d3-js-data-visualizations/)
- [学习制作折线图](https://medium.freecodecamp.org/learn-to-create-a-line-chart-using-d3-js-4f43f1ee716b)

### 游戏开发：

- [使用 Phaser 制作 2D 打砖块游戏](https://developer.mozilla.org/en-US/docs/Games/Tutorials/2D_breakout_game_Phaser)
- 使用 Phaser、HTML5 和 JavaScript 制作 Flappy Bird
  - [第 1 部分](http://www.lessmilk.com/tutorial/flappy-bird-phaser-1)
  - [第 2 部分](http://www.lessmilk.com/tutorial/flappy-bird-phaser-2)

### 桌面应用：

- [使用 React 和 Electron 构建桌面聊天应用](https://medium.freecodecamp.org/build-a-desktop-chat-app-with-react-electron-and-chatkit-744d168e6f2f)

### 其他：

- [如何用不到 20 行代码构建 Web 框架](https://www.pubnub.com/blog/build-yourself-a-web-framework-in-less-than-20-lines-of-code/)
- [自己构建 Redux](https://zapier.com/engineering/how-to-build-redux/)
- [如何编写自己的虚拟 DOM](https://medium.com/@deathmood/how-to-write-your-own-virtual-dom-ee74acc13060)
- [在 AWS 上使用 WebSocket 构建实时无服务器 GraphQL API](https://andrewgriffithsonline.com/blog/serverless-websockets-on-aws/)

## Kotlin:

- [Keddit：通过开发 Android 应用学习 Kotlin](https://medium.com/@juanchosaravia/learn-kotlin-while-developing-an-android-app-introduction-567e21ff9664)

## Lua:

### LÖVE:

- BYTEPATH：使用 Lua 和 LÖVE 创建完整游戏
  - [第 0 部分：介绍](https://github.com/a327ex/blog/issues/30)
  - [第 1 部分：游戏循环](https://github.com/a327ex/blog/issues/15)
  - [第 2 部分：库](https://github.com/a327ex/blog/issues/16)
  - [第 3 部分：房间与区域](https://github.com/a327ex/blog/issues/17)
  - [第 4 部分：练习](https://github.com/a327ex/blog/issues/18)
  - [第 5 部分：游戏基础](https://github.com/a327ex/blog/issues/19)
  - [第 6 部分：玩家基础](https://github.com/a327ex/blog/issues/20)
  - [第 7 部分：玩家属性与攻击](https://github.com/a327ex/blog/issues/21)
  - [第 8 部分：敌人](https://github.com/a327ex/blog/issues/22)
  - [第 9 部分：导演与游戏循环](https://github.com/a327ex/blog/issues/23)
  - [第 10 部分：编码实践](https://github.com/a327ex/blog/issues/24)
  - [第 11 部分：被动技能](https://github.com/a327ex/blog/issues/25)
  - [第 12 部分：更多被动技能](https://github.com/a327ex/blog/issues/26)
  - [第 13 部分：技能树](https://github.com/a327ex/blog/issues/27)
  - [第 14 部分：控制台](https://github.com/a327ex/blog/issues/28)
  - [第 15 部分：结尾](https://github.com/a327ex/blog/issues/29)

## Python:

### 网页抓取：

- [使用 Python 挖掘 Twitter 数据](https://marcobonzanini.com/2015/03/02/mining-twitter-data-with-python-part-1/)
- [使用 Scrapy 和 MongoDB 抓取网站](https://realpython.com/blog/python/web-scraping-with-scrapy-and-mongodb/)
- [如何使用 Python 和 Selenium WebDriver 抓取网页](http://www.byperth.com/2018/04/25/guide-web-scraping-101-what-you-need-to-know-and-how-to-scrape-with-python-selenium-webdriver/)
- [使用 BeautifulSoup 决定该看哪部电影](https://medium.com/@nishantsahoo.in/which-movie-should-i-watch-5c83a3c0f5b1)

### Web 应用：

- [使用 Flask 构建微博客](https://blog.miguelgrinberg.com/post/the-flask-mega-tutorial-part-i-hello-world)
- [使用 Django 创建博客 Web 应用](https://tutorial.djangogirls.org/en/)
- [选择自己的冒险式演示](https://www.twilio.com/blog/2015/03/choose-your-own-adventures-presentations-wizard-mode-part-1-of-3.html)
- [使用 Flask 和 RethinkDB 构建待办事项列表](https://realpython.com/blog/python/rethink-flask-a-simple-todo-list-powered-by-flask-and-rethinkdb/)
- [使用 Django 和测试驱动开发构建待办事项列表](http://www.obeythetestinggoat.com/)
- [使用 Python 构建 RESTful 微服务](http://www.skybert.net/python/developing-a-restful-micro-service-in-python/)
- [使用 Docker、Flask 和 React 构建微服务](https://testdriven.io/)
- [使用 Flask 构建简单 Web 应用](https://pythonspot.com/flask-web-app-with-python/)
- [20 分钟内创建 Django API](https://codeburst.io/create-a-django-api-in-under-20-minutes-2a082a60f6f3)
- 使用 Django、Postgres 和 JavaScript 构建社区驱动的配送应用
  - [第 1 部分](https://www.ashwinhariharan.tech/blog/thinking-of-building-a-contact-tracing-application-heres-what-you-can-do-instead/)
  - [第 2 部分](https://www.ashwinhariharan.tech/blog/thinking-of-building-a-contact-tracing-application-heres-what-you-can-do-instead-part-2/)
- 使用 Vue、django-notifs、RabbitMQ 和 uWSGI 构建实时聊天应用
  - [第 1 部分](https://danidee10.github.io/2018/01/01/realtime-django-1.html)
  - [第 2 部分](https://danidee10.github.io/2018/01/03/realtime-django-2.html)
  - [第 3 部分](https://danidee10.github.io/2018/01/07/realtime-django-3.html)
  - [第 4 部分](https://danidee10.github.io/2018/01/10/realtime-django-4.html)
  - [第 5 部分](https://danidee10.github.io/2018/01/13/realtime-django-5.html)
  - [第 6 部分](https://danidee10.github.io/2018/03/12/realtime-django-6.html)
- [使用 Python 创建 API：初学者完整课程（视频）](https://www.freecodecamp.org/news/creating-apis-with-python-free-19-hour-course/)
- [构建第一个 Streamlit 数据应用](https://docs.streamlit.io/get-started/tutorials/create-an-app)

### 机器人：

- [构建 Reddit 机器人](https://www.pythonforengineers.com/blog/build-a-reddit-bot-part-1/)
- [如何制作 Reddit 机器人：YouTube 视频](https://www.youtube.com/watch?v=krTUf7BpTc0) （视频）
- [构建 Facebook Messenger 机器人](https://blog.hartleybrody.com/fb-messenger-bot/)
- [制作 Reddit + Facebook Messenger 机器人](https://pythontips.com/2017/04/13/making-a-reddit-facebook-messenger-bot/)
- 如何使用 Python 创建 Telegram 机器人
  - [第 1 部分](https://khashtamov.com/en/how-to-create-a-telegram-bot-using-python/)
  - [第 2 部分](https://khashtamov.com/en/how-to-deploy-telegram-bot-django/)
- [使用 Python 创建 Twitter 机器人](https://medium.freecodecamp.org/creating-a-twitter-bot-in-python-with-tweepy-ac524157a607)

### 数据科学：

- 通过多个项目学习 Python 数据科学（视频）：
  - [第 1 部分：介绍](https://www.youtube.com/watch?v=T5pRlIbr6gg)
  - [第 2 部分：Twitter 情感分析](https://www.youtube.com/watch?v=o_OZdbCzHUA)
  - [第 3 部分：推荐系统](https://www.youtube.com/watch?v=9gBC9R-msAk&list=PL2-dafEMk2A6QKz1mrk1uIGfHkC1zZ6UU&index=3)
  - [第 4 部分：预测股票价格](https://www.youtube.com/watch?v=SSu00IRRraY&index=4&list=PL2-dafEMk2A6QKz1mrk1uIGfHkC1zZ6UU)
  - [第 5 部分：TensorFlow 中的 Deep Dream](https://www.youtube.com/watch?v=MrBzgvUNr4w&list=PL2-dafEMk2A6QKz1mrk1uIGfHkC1zZ6UU&index=5)
  - [第 6 部分：遗传算法](https://www.youtube.com/watch?v=dSofAXnnFrY&index=6&list=PL2-dafEMk2A6QKz1mrk1uIGfHkC1zZ6UU)

### 机器学习：

- [使用 Python 从零编写线性回归](https://www.youtube.com/watch?v=uwwWVAgJBcM) （视频）
- [使用 Python 循序渐进学习机器学习](https://machinelearningmastery.com/machine-learning-in-python-step-by-step/)
- [预测葡萄酒品质](https://medium.freecodecamp.org/using-machine-learning-to-predict-the-quality-of-wines-9e2e13d7480d)
- [解决水果分类问题](https://towardsdatascience.com/solving-a-simple-classification-problem-with-python-fruits-lovers-edition-d20ab6b071d2)
- [使用 Python 学习无监督学习](https://scikit-learn.org/stable/unsupervised_learning.html)
- [使用 Python 从零构建自己的神经网络](https://towardsdatascience.com/how-to-build-your-own-neural-network-from-scratch-in-python-68998a08e4f6)
- [在 Python 中不使用 sklearn 实现线性回归](https://medium.com/we-are-orb/linear-regression-in-python-without-scikit-learn-50aef4b8d122)
- [不使用 sklearn 实现多元线性回归](https://medium.com/we-are-orb/multivariate-linear-regression-in-python-without-scikit-learn-7091b1d45905)
- [使用 KNN 的音乐推荐器](https://towardsdatascience.com/how-to-build-a-simple-song-recommender-296fcbc8c85)
- 查找相似的 Quora 问题
  - [使用 BOW、TF-IDF 和 XGBoost](https://towardsdatascience.com/finding-similar-quora-questions-with-bow-tfidf-and-random-forest-c54ad88d1370)
  - [使用 Word2Vec 和 XGBoost](https://towardsdatascience.com/finding-similar-quora-questions-with-word2vec-and-xgboost-1a19ad272c0d)
- [使用 Python 和机器学习检测假新闻](https://data-flair.training/blogs/advanced-python-project-detecting-fake-news/)

### OpenCV：

- [构建文档扫描器](https://www.pyimagesearch.com/2014/09/01/build-kick-ass-mobile-document-scanner-just-5-minutes/)
- [使用 OpenCV 和深度学习构建人脸检测器](https://www.pyimagesearch.com/2018/02/26/face-detection-with-opencv-and-deep-learning/)
- [使用 YOLOv3 构建极快速的自定义目标检测系统（视频播放列表）](https://www.youtube.com/playlist?list=PLKHYJbyeQ1a0oGzgRXy-QwAN1tSV4XZxg)
- [使用 OpenCV、Python 和深度学习构建人脸识别系统](https://www.pyimagesearch.com/2018/06/18/face-recognition-with-opencv-python-and-deep-learning/)
- [检测图像中的显著特征](https://www.pyimagesearch.com/2018/07/16/opencv-saliency-detection/)
- [构建条形码扫描器](https://www.pyimagesearch.com/2018/05/21/an-opencv-barcode-and-qr-code-scanner-with-zbar/)
- [使用 Python 学习人脸聚类](https://www.pyimagesearch.com/2018/07/09/face-clustering-with-python/)
- [使用 Camshift 跟踪目标](https://www.pyimagesearch.com/wp-content/uploads/2014/11/opencv_crash_course_camshift.pdf)
- [使用 OpenCV 和深度学习进行语义分割](https://www.pyimagesearch.com/2018/09/03/semantic-segmentation-with-opencv-and-deep-learning/)
- [图像和视频中的文本检测](https://www.pyimagesearch.com/2018/08/20/opencv-text-detection-east-text-detector/)
- [使用 OpenCV 的人数计数器](https://www.pyimagesearch.com/2018/08/13/opencv-people-counter/)
- [使用 OpenCV 跟踪多个目标](https://www.pyimagesearch.com/2018/08/06/tracking-multiple-objects-with-opencv/)
- [使用 OpenCV 进行神经风格迁移](https://www.pyimagesearch.com/2018/08/27/neural-style-transfer-with-opencv/)
- [OpenCV OCR 和文本识别](https://www.pyimagesearch.com/2018/09/17/opencv-ocr-and-text-recognition-with-tesseract/)
- [文本倾斜校正教程](https://www.pyimagesearch.com/2017/02/20/text-skew-correction-opencv-python/)
- [面部关键点检测教程](https://www.pyimagesearch.com/2017/04/03/facial-landmarks-dlib-opencv-python/)
- [使用 Mask R-CNN 进行目标检测](https://www.learnopencv.com/deep-learning-based-object-detection-and-instance-segmentation-using-mask-r-cnn-in-opencv-python-c/)
- [自动目标检测教程](https://www.pyimagesearch.com/2015/05/04/target-acquired-finding-targets-in-drone-and-quadcopter-video-streams-using-python-and-opencv/)
- [使用 OpenCV 实现 EigenFaces](https://www.learnopencv.com/eigenface-using-opencv-c-python/)
- [更快的五点面部关键点检测教程](https://www.pyimagesearch.com/2018/04/02/faster-facial-landmark-detector-with-dlib/)
- [手部关键点检测](https://www.learnopencv.com/hand-keypoint-detection-using-deep-learning-and-opencv/)
- Dlib 相关目标跟踪
  - [单目标跟踪器](https://www.pyimagesearch.com/2018/10/22/object-tracking-with-dlib/)
  - [多目标跟踪器](https://www.pyimagesearch.com/2018/10/29/multi-object-tracking-with-dlib/)
- [使用 OpenCV 和 Python 拼接图像](https://www.pyimagesearch.com/2018/12/17/image-stitching-with-opencv-and-python/)
- [使用 OpenCV 进行实例分割](https://www.pyimagesearch.com/2018/11/26/instance-segmentation-with-opencv/)
- [口罩检测器](https://www.pyimagesearch.com/2020/05/04/covid-19-face-mask-detector-with-opencv-keras-tensorflow-and-deep-learning/)

### 深度学习：

- [使用卷积神经网络检测面部关键点](http://danielnouri.org/notes/2014/12/17/using-convolutional-neural-nets-to-detect-facial-keypoints-tutorial/)
- [使用 Python 和 OpenCV 生成人脸平均图像](https://www.learnopencv.com/average-face-opencv-c-python-tutorial/)
- [使用 CNN 破解验证码系统](https://medium.com/@ageitgey/how-to-break-a-captcha-system-in-15-minutes-with-machine-learning-dbebb035a710)
- [使用预训练 Inception 模型进行图像预测](https://medium.com/google-cloud/keras-inception-v3-on-google-compute-engine-a54918b0058)
- [创建第一个 CNN](https://hackernoon.com/deep-learning-cnns-in-tensorflow-with-gpus-cba6efe0acc2)
- [构建人脸识别流程](https://hackernoon.com/building-a-facial-recognition-pipeline-with-deep-learning-in-tensorflow-66e7645015b8)
- [构建图像描述生成器](https://medium.freecodecamp.org/building-an-image-caption-generator-with-deep-learning-in-tensorflow-a142722e9b1f)
- [制作自己的人脸识别系统](https://medium.freecodecamp.org/making-your-own-face-recognition-system-29a8e728107c)
- [20 分钟内训练语言检测 AI](https://towardsdatascience.com/how-i-trained-a-language-detection-ai-in-20-minutes-with-a-97-accuracy-fdeca0fb7724)
- [使用神经网络进行目标检测](https://towardsdatascience.com/object-detection-with-neural-networks-a4e2c46b4491)
- 学习 Twitter 情感分析
  - [第 1 部分：数据清洗](https://towardsdatascience.com/another-twitter-sentiment-analysis-bb5b01ebad90)
  - [第 2 部分：探索性数据分析与可视化](https://towardsdatascience.com/another-twitter-sentiment-analysis-with-python-part-2-333514854913)
  - [第 3 部分：齐普夫定律与数据可视化](https://towardsdatascience.com/another-twitter-sentiment-analysis-with-python-part-3-zipfs-law-data-visualisation-fc9eadda71e7)
  - [第 4 部分：特征提取（计数向量化器）](https://towardsdatascience.com/another-twitter-sentiment-analysis-with-python-part-4-count-vectorizer-b3f4944e51b5)
  - [第 5 部分：特征提取（TF-IDF 向量化器）](https://towardsdatascience.com/another-twitter-sentiment-analysis-with-python-part-5-50b4e87d9bdd)
  - [第 6 部分：Doc2Vec](https://towardsdatascience.com/another-twitter-sentiment-analysis-with-python-part-6-doc2vec-603f11832504)
  - [第 7 部分：短语建模与 Doc2Vec](https://towardsdatascience.com/another-twitter-sentiment-analysis-with-python-part-7-phrase-modeling-doc2vec-592a8a996867)
  - [第 8 部分：降维](https://towardsdatascience.com/another-twitter-sentiment-analysis-with-python-part-8-dimensionality-reduction-chi2-pca-c6d06fb3fcf3)
  - [第 9 部分：使用 TF-IDF 向量的神经网络](https://towardsdatascience.com/another-twitter-sentiment-analysis-with-python-part-9-neural-networks-with-tfidf-vectors-using-d0b4af6be6d7)
  - [第 10 部分：使用 word2vec/doc2vec 的神经网络](https://towardsdatascience.com/another-twitter-sentiment-analysis-with-python-part-10-neural-network-with-a6441269aa3c)
  - [第 11 部分：结合 Word2Vec 的 CNN](https://towardsdatascience.com/another-twitter-sentiment-analysis-with-python-part-11-cnn-word2vec-41f5e28eda74)
- [使用迁移学习实现自定义图像分类](https://becominghuman.ai/transfer-learning-retraining-inception-v3-for-custom-image-classification-2820f653c557)
- [学习用 11 行 Python 代码编写简单神经网络](https://iamtrask.github.io/2015/07/12/basic-python-network/)
- [使用梯度下降方法构建神经网络](https://iamtrask.github.io/2015/07/27/python-network-part2/)
- [训练 Keras 模型生成颜色](https://heartbeat.fritz.ai/how-to-train-a-keras-model-to-generate-colors-3bc79e54971b)
- [使用自定义数据集开始使用 Keras](https://www.pyimagesearch.com/2018/09/10/keras-tutorial-how-to-get-started-with-keras-deep-learning-and-python/)
- [在 Faces94 数据集上使用 EigenFaces 和 FisherFaces](https://nicholastsmith.wordpress.com/2016/02/18/eigenfaces-versus-fisherfaces-on-the-faces94-database-with-scikit-learn/)
- [Kaggle MNIST 数字识别教程](https://medium.com/@lvarruda/how-to-get-top-2-position-on-kaggles-mnist-digit-recognizer-48185d80a2d4)
- [使用 tf.keras 的 Fashion MNIST 教程](https://medium.com/tensorflow/hello-deep-learning-fashion-mnist-with-keras-50fcff8cd74a)
- [使用 Keras 中的 CNN 自动分类根系健康状况](https://www.pyimagesearch.com/2018/10/15/deep-learning-hydroponics-and-medical-marijuana/)
- [Keras 与 TensorFlow 对比](https://www.pyimagesearch.com/2018/10/08/keras-vs-tensorflow-which-one-is-better-and-which-one-should-i-learn/)
- [利用深度学习和医学图像分析检测疟疾](https://www.pyimagesearch.com/2018/12/03/deep-learning-and-medical-image-analysis-with-keras/)
- [使用 Keras 进行图像分类迁移学习](https://towardsdatascience.com/transfer-learning-for-image-classification-using-keras-c47ccf09c8c8)
- [在 Python 中使用 CNN 编写微笑分类器](https://github.com/kylemcdonald/SmileCNN)
- [使用 scikit-learn 进行自然语言处理](https://towardsdatascience.com/natural-language-processing-count-vectorization-with-scikit-learn-e7804269bb5e)
- [编写 Taylor Swift 歌词生成器](https://towardsdatascience.com/ai-generates-taylor-swifts-song-lyrics-6fd92a03ef7e)
- [使用 PyTorch Lightning 检测口罩](https://towardsdatascience.com/how-i-built-a-face-mask-detector-for-covid-19-using-pytorch-lightning-67eb3752fd61)

### 其他：

- [构建简单解释器](https://ruslanspivak.com/lsbasi-part1/)
- [使用 Python 构建简单区块链](https://hackernoon.com/learn-blockchains-by-building-one-117428612f46)
- [使用 OpenCV、Python 和 iOS 构建加油泵扫描器](https://hackernoon.com/building-a-gas-pump-scanner-with-opencv-python-ios-116fe6c9ae8b)
- [使用 Python 和 Kafka 构建分布式流处理系统](https://codequs.com/p/S14jQ5UyG/build-a-distributed-streaming-system-with-apache-kafka-and-python)
- [用标准 Python 从零编写基础 x86-64 JIT 编译器](https://csl.name/post/python-jit/)
- 制作底层 Linux 调试器
  - [第 1 部分](https://blog.asrpo.com/making_a_low_level_debugger)
  - [第 2 部分：C](https://blog.asrpo.com/making_a_low_level_debugger_part_2)
- 实现搜索引擎
  - [第 1 部分](http://www.ardendertat.com/2011/05/30/how-to-implement-a-search-engine-part-1-create-index/)
  - [第 2 部分](http://www.ardendertat.com/2011/05/31/how-to-implement-a-search-engine-part-2-query-index/)
  - [第 3 部分](http://www.ardendertat.com/2011/07/17/how-to-implement-a-search-engine-part-3-ranking-tf-idf/)
- [构建生命游戏](https://robertheaton.com/2018/07/20/project-2-game-of-life/)
- [创建终端 ASCII 艺术](https://robertheaton.com/2018/06/12/programming-projects-for-advanced-beginners-ascii-art/)
- [编写井字棋 AI](https://robertheaton.com/2018/10/09/programming-projects-for-advanced-beginners-3-a/)
- [创建照片马赛克艺术](https://robertheaton.com/2018/11/03/programming-project-4-photomosaics/)
- [在终端中构建贪吃蛇游戏](https://robertheaton.com/2018/12/02/programming-project-5-snake/)
- [自己编写 Git](https://wyag.thb.lt/)
- [用 Python 实现 Python 字节码执行器](https://www.aosabook.org/en/500L/a-python-interpreter-written-in-python.html)
- [使用 Python 创建语音助手](https://www.geeksforgeeks.org/voice-assistant-using-python/)

## Go:

- [使用 Golang、Angular 2 和 WebSocket 创建实时聊天应用](https://web.archive.org/web/20260214230813/https://www.thepolyglotdeveloper.com/2016/12/create-real-time-chat-app-golang-angular-2-websockets/)
- [使用 Gin 构建 Go Web 应用和微服务](https://semaphoreci.com/community/tutorials/building-go-web-applications-and-microservices-using-gin)
- [如何使用 Godog 在 Go 中进行行为驱动开发](https://semaphoreci.com/community/tutorials/how-to-use-godog-for-behavior-driven-development-in-go)
- 使用 Go 构建区块链
  - [第 1 部分：基本原型](https://jeiwan.net/posts/building-blockchain-in-go-part-1/)
  - [第 2 部分：工作量证明](https://jeiwan.net/posts/building-blockchain-in-go-part-2/)
  - [第 3 部分：持久化和 CLI](https://jeiwan.net/posts/building-blockchain-in-go-part-3/)
  - [第 4 部分：交易 1](https://jeiwan.net/posts/building-blockchain-in-go-part-4/)
  - [第 5 部分：地址](https://jeiwan.net/posts/building-blockchain-in-go-part-5/)
  - [第 6 部分：交易 2](https://jeiwan.net/posts/building-blockchain-in-go-part-6/)
  - [第 7 部分：网络](https://jeiwan.net/posts/building-blockchain-in-go-part-7/)
- [使用 Go 从零构建容器：Liz Rice（Microscaling Systems，视频）](https://www.youtube.com/watch?v=8fi7uSYlOdc)
- [使用 GoLang 构建 Web 应用](https://astaxie.gitbooks.io/build-web-application-with-golang/content/en/)
- 使用 Go 和 ReactJS 构建聊天应用
  - [第 1 部分：初始设置](https://tutorialedge.net/projects/chat-system-in-go-and-react/part-1-initial-setup/)
  - [第 2 部分：简单通信](https://tutorialedge.net/projects/chat-system-in-go-and-react/part-2-simple-communication/)
  - [第 3 部分：设计前端](https://tutorialedge.net/projects/chat-system-in-go-and-react/part-3-designing-our-frontend/)
  - [第 4 部分：处理多个客户端](https://tutorialedge.net/projects/chat-system-in-go-and-react/part-4-handling-multiple-clients/)
  - [第 5 部分：改进前端](https://tutorialedge.net/projects/chat-system-in-go-and-react/part-5-improved-frontend/)
  - [第 6 部分：将后端容器化](https://tutorialedge.net/projects/chat-system-in-go-and-react/part-6-dockerizing-your-backend/)
- [Go WebAssembly 教程：构建计算器](https://tutorialedge.net/golang/go-webassembly-tutorial/)
- 使用 Go 构建 REST 服务器
  - [第 1 部分：标准库](https://eli.thegreenplace.net/2021/rest-servers-in-go-part-1-standard-library/)
  - [第 2 部分：使用路由包](https://eli.thegreenplace.net/2021/rest-servers-in-go-part-2-using-a-router-package/)
  - [第 3 部分：使用 Web 框架](https://eli.thegreenplace.net/2021/rest-servers-in-go-part-3-using-a-web-framework/)
  - [第 4 部分：使用 OpenAPI 和 Swagger](https://eli.thegreenplace.net/2021/rest-servers-in-go-part-4-using-openapi-and-swagger/)
  - [第 5 部分：中间件](https://eli.thegreenplace.net/2021/rest-servers-in-go-part-5-middleware/)
  - [第 6 部分：身份认证](https://eli.thegreenplace.net/2021/rest-servers-in-go-part-6-authentication/)
  - [第 7 部分：GraphQL](https://eli.thegreenplace.net/2021/rest-servers-in-go-part-7-graphql/)
- 使用 Gin 和 Redis 构建 Go URL 缩短器
  - [第 1 部分：项目设置](https://www.eddywm.com/lets-build-a-url-shortener-in-go/)
  - [第 2 部分：存储层](https://www.eddywm.com/lets-build-a-url-shortener-in-go-with-redis-part-2-storage-layer/)
  - [第 3 部分：短链接生成器](https://www.eddywm.com/lets-build-a-url-shortener-in-go-part-3-short-link-generation/)
  - [第 4 部分：转发](https://www.eddywm.com/lets-build-a-url-shortener-in-go-part-iv-forwarding/)
- [使用 Go 构建 TCP 聊天（视频）](https://www.youtube.com/watch?v=Sphme0BqJiY)
- [使用 Go 从零构建 BitTorrent 客户端](https://blog.jse.li/posts/torrent/)
- [使用 Go、PostgreSQL 和 Docker 的 REST API 大师课（视频播放列表）`进行中`](https://www.youtube.com/watch?v=rx6CPDK_5mU&list=PLy_6D98if3ULEtXtNSY_2qN21VCKgoQAE)

## PHP:

- [如何使用 Laravel 构建博客](https://www.youtube.com/playlist?list=PLwAKR305CRO-Q90J---jXVzbOd4CDRbVx) （视频）
- [使用纯 PHP 制作自己的博客](http://ilovephp.jondh.me.uk/en/tutorial/make-your-own-blog)
- [使用 SilverStripe 构建房地产网站示例](https://www.silverstripe.org/learn/lessons/)
- [使用 Laravel 5.4 和 VueJS 构建实时聊天应用](https://www.youtube.com/playlist?list=PLXsbBbd36_uVjOFH_P25__XAyGsohXWlv) （视频）
- [构建社交网络：Laravel 5，YouTube 视频](https://www.youtube.com/playlist?list=PLfdtiltiRHWGGxaR6uFtwZnnbcXqyq8JD) （视频）
- 使用 Laravel 构建功能完整的多租户应用
  - [第 0 部分：介绍](https://medium.com/@ashokgelal/writing-a-full-featured-multi-tenant-laravel-app-from-scratch-a0e1a7350d9d)
  - [第 1 部分：设置](https://medium.com/@ashokgelal/a-full-featured-multi-tenant-app-with-laravel-part-1-4049a3cc229d)
  - [第 2 部分：角色和权限](https://medium.com/@ashokgelal/a-full-featured-multi-tenant-app-with-laravel-part-2-roles-and-permissions-d9a5bfe5d525)
  - [第 3 部分：邀请](https://medium.com/@ashokgelal/a-full-featured-multi-tenant-app-with-laravel-part-3-invitation-c982dca55eb9)
  - [第 4 部分：身份认证](https://medium.com/@ashokgelal/a-full-featured-multi-tenant-app-with-laravel-part-4-tenancy-aware-authentication-e0ee37270bc8)
  - [第 5 部分：测试](https://medium.com/@ashokgelal/a-full-featured-multi-tenant-app-with-laravel-part-2-unit-tests-96d6dfbf0617)
  - [第 6 部分：用户资料](https://medium.com/@ashokgelal/a-full-featured-multi-tenant-app-with-laravel-part-5-user-profile-5c3d0c655f3a)
  - [第 7 部分：部署](https://medium.com/@ashokgelal/a-full-featured-multi-tenant-app-with-laravel-part-7-deployment-40bb3c895627)
- [从零构建 Laravel CRUD 应用](https://www.codewall.co.uk/laravel-crud-demo-with-resource-controller-tutorial/)

## OCaml:

- [在 OCaml 中使用 LLVM 实现一门语言](https://llvm.org/docs/tutorial/#kaleidoscope-implementing-a-language-with-llvm-in-objective-caml)
- [使用 OCaml 编写 Game Boy 模拟器](https://linoscope.github.io/writing-a-game-boy-emulator-in-ocaml/)

## Ruby:

- [使用 Ruby 构建网络协议栈](https://medium.com/geckoboard-under-the-hood/how-to-build-a-network-stack-in-ruby-f73aeb1b661b)
- 构建自己的 Redis
  - [第 0 部分：介绍](https://rohitpaulk.com/articles/redis-0)
  - [第 1 部分：最简 TCP 服务器](https://rohitpaulk.com/articles/redis-1)
  - [第 2 部分：PING <-> PONG](https://rohitpaulk.com/articles/redis-2)
  - [第 3 部分：并发客户端](https://rohitpaulk.com/articles/redis-3)
  - [第 4 部分：ECHO](https://rohitpaulk.com/articles/redis-4)
- [使用 Ruby 重建 Git](https://thoughtbot.com/blog/rebuilding-git-in-ruby)

### Ruby on Rails:

- [Ruby on Rails 教程](https://www.railstutorial.org/book)
- [使用 Ruby on Rails 从零构建 Instagram](https://www.dropbox.com/s/9vq430e9s3q7pu8/Let%27s%20Build%20Instagram%20with%20Ruby%20on%20Rails%20-%20Free%20Edition.pdf?dl=0)
- [使用 Rails 构建社交网络](https://medium.com/rails-ember-beyond/how-to-build-a-social-network-using-rails-eb31da569233)
- [如何构建 Ruby on Rails 应用](https://www.digitalocean.com/community/tutorials/how-to-build-a-ruby-on-rails-application)

## Haskell:

- [编写你自己的 Haskell：构建现代函数式编译器](http://dev.stephendiehl.com/fun/)
- [48 小时内编写自己的 Scheme](https://en.wikibooks.org/wiki/Write_Yourself_a_Scheme_in_48_Hours)
- [编写你自己的 Scheme，版本 2](https://github.com/adamwespiser/scheme)
- [自己编写 IRC 机器人](https://wiki.haskell.org/Roll_your_own_IRC_bot)
- [制作 Movie Monad](https://lettier.github.io/posts/2016-08-15-making-movie-monad.html)
- [使用 Haskell 制作网站 **（已过时）**](http://adit.io/posts/2013-04-15-making-a-website-with-haskell.html)

## R:

- [使用 Shiny 构建 Web 应用](http://shiny.rstudio.com/tutorial/)
- [构建加密货币机器人](https://towardsdatascience.com/build-a-cryptocurrency-trading-bot-with-r-1445c429e1b1)
- [学习使用 R 进行关联规则挖掘](https://towardsdatascience.com/association-rule-mining-in-r-ddf2d044ae50)

## Rust:

- 使用 Rust 构建简单 Web 应用
  - [第 1 部分](http://joelmccracken.github.io/entries/a-simple-web-app-in-rust-pt-1/)
  - [第 2a 部分](http://joelmccracken.github.io/entries/a-simple-web-app-in-rust-pt-2a/)
  - [第 2b 部分](http://joelmccracken.github.io/entries/a-simple-web-app-in-rust-pt-2b/)
- [用纯 Rust 编写操作系统](https://os.phil-opp.com/)
- [使用 Rust 构建浏览器引擎](https://limpet.net/mbrubeck/2014/08/08/toy-layout-engine-1.html)
- [使用 Rust 编写微服务](http://www.goldsborough.me/rust/web/tutorial/2018/01/20/17-01-11-writing_a_microservice_in_rust/)
- [通过大量链表学习 Rust](https://rust-unofficial.github.io/too-many-lists/)
- 深入 Rust：从零编写可扩展聊天服务
  - [第 1 部分：实现 WebSocket，介绍](https://nbaksalyar.github.io/2015/07/10/writing-chat-in-rust.html)
  - [第 2 部分：发送和接收消息](https://nbaksalyar.github.io/2015/11/09/rust-in-detail-2.html)
- [为桌面和 Web 编写 Rust Roguelike 游戏](https://aimlesslygoingforward.com/blog/2019/02/09/writing-a-rust-roguelike-for-the-desktop-and-the-web/)
- [使用 Rust 构建单页应用](http://www.sheshbabu.com/posts/rust-wasm-yew-single-page-application/)
- [使用 Rust 编写 NES 模拟器](https://bugzmanov.github.io/nes_ebook/)
- 使用神经网络和遗传算法创建进化模拟，并将应用编译为 WebAssembly
  - [第 1 部分](https://pwy.io/en/posts/learning-to-fly-pt1/)
  - [第 2 部分](https://pwy.io/en/posts/learning-to-fly-pt2/)
  - [第 3 部分](https://pwy.io/en/posts/learning-to-fly-pt3/)
  - [第 4 部分](https://pwy.io/en/posts/learning-to-fly-pt4/)
- [hecto：使用 Rust 构建自己的文本编辑器](https://www.flenker.blog/hecto/)

## Scala:

- [基于 Actor 的简单区块链](https://www.freecodecamp.org/news/how-to-build-a-simple-actor-based-blockchain-aac1e996c177/)
- [没有魔法：正则表达式](https://rcoh.svbtle.com/no-magic-regular-expressions)

## Swift:

- [Hacking with Swift：通过实践 39 个项目学习 Swift](https://www.hackingwithswift.com/read)
- [从零构建复古第一人称射击游戏](https://github.com/nicklockwood/RetroRampage)
- [使用 SwiftUI 构建 iOS 应用](https://www.swift.org/getting-started/swiftui/)
- [使用 SwiftUI 开发 iOS 应用（斯坦福 CS193p）](https://cs193p.sites.stanford.edu/) （视频）

<a id="additional-resources"></a>
## 其他资源

- [React Redux 链接](https://github.com/markerikson/react-redux-links)
- [Udemy.com](https://www.udemy.com/)
- [Full Stack Python](https://www.fullstackpython.com/)
- [Node School](https://nodeschool.io/)
- [DigitalOcean 社区教程](https://www.digitalocean.com/community/tutorials)
- [Exercism](http://www.exercism.io/)
- [Egghead.io](http://www.egghead.io/)
- [Michael Herman 的博客](http://mherman.org/)
- [Thinkster.io](http://thinkster.io)
- [Enlight](https://enlight.nyc/)
- [Hack Club 工作坊](https://hackclub.com/workshops/)
- [CodeCrafters](https://codecrafters.io/)
