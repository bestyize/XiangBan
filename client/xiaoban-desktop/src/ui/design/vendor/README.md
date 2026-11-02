# Figma 静态编解码器

`figma-codec.js` / `.d.ts` 由 `scripts/build-design-figma-codec.mjs` 生成并入库，生产环境不运行 Kiwi 的动态编译器，不放宽控制台 CSP。

- 模板与内嵌 schema：[openfig-core 0.4.1](https://github.com/OpenFig-org/openfig-core)，npm 作者 `rcoenen`，上游 README 与 npm 元数据声明 MIT。
- 编译器与 ByteBuffer：[kiwi-schema 0.5.0](https://github.com/evanw/kiwi)，MIT，Copyright (c) 2016–2023 Evan Wallace。
- 文件编码使用 openfig-core 的容器与矢量辅助函数；消息压缩使用本地 RFC 8878 原始 Zstandard 块，不需要 WASM、Figma 插件、账号或网络请求。

重新生成：安装桌面与 `libs/agent-tools/open-claude-design/runtime` 的锁定依赖，在桌面目录运行 `npm run design:figma-codec`。升级任一依赖后，运行 `editable-export.test.ts`，并用 Figma 重新验收导入；本地解析往返不能替代 Figma 自身的验收。

## Kiwi MIT 许可证

Copyright (c) 2016-2023 Evan Wallace

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
