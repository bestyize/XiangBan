# 待审稿抽查记录

本次对非本人整理的五个 ready 稿进行独立抽查，依据批次内源文、译文、元数据及许可文件；结果表示可进入后续编辑审查，不代表已经做完所有外部链接或图片权利核验。

| 项目 | 来源与正文检查 | 授权和未核验范围 |
| --- | --- | --- |
| withastro/astro | Git blob SHA 按原始字节匹配；7 个主要章节对应，2 个代码块逐字一致，软件包和更新日志固定版本链接保留。 | MIT 原文与版权已保存；上游图片外链的独立权利待核。 |
| x64dbg/x64dbg | Git blob SHA 按原始字节匹配；源文为 CRLF，不应先按文本模式转换换行；10 个章节与历史捐赠表保留。 | 保存上游修改版 GPLv3 及其动态插件例外，未误称标准 GPL；虫子图标权利未确认。 |
| paperless-ngx/paperless-ngx | Git blob SHA 匹配，主要章节及明文存储、不可信主机、演示重置警告保留；1 个代码块逐字一致。 | GPL 原文已保存；文档域名及演示返回 403 的范围已记录，使用固定提交文档作替代入口。 |
| payloadcms/payload | Git blob SHA 匹配，主要章节及功能、模板、部署内容对应；2 个代码块逐字一致。 | MIT 原文已保存；性能、安全和首创说法为上游声明，部署短链与部分页面 403 的范围已记录。 |
| mpv-player/mpv | Git blob SHA 匹配，主要章节对应，系统版本、依赖与 DASH 安全说明保留。 | Copyright、GPL、LGPL 原文均保存；已说明 `-Dgpl=false` 自身不授予 LGPL 权利及文档例外。 |

五项的版本号未发现遗漏；指向上游 main、master 或 development 的文档链接在阅读稿中改成对应固定提交是预期差异。外链、商标和第三方素材未作全量联网或独立授权核验。

# 五项许可阻塞复核

下列均为缺少可核实的根 README 转载、翻译授权依据，并非仅因 GitHub API 返回 `NOASSERTION`。这不等同于已证明权利人明确禁止转载；获得额外授权证据后应重新审查。

- mapeditor/tiled：固定提交 `83e4cfd9faa653e543e64d1af3b1010f8d5794da` 的 [COPYING](https://github.com/mapeditor/tiled/blob/83e4cfd9faa653e543e64d1af3b1010f8d5794da/COPYING) 指明各文件和代码目录各自许可，列表未覆盖根 README；当前未找到该文件许可头。
- 4ian/GDevelop：固定提交 `4d867e222bc7c462ca310425c95080b28b403a8d` 的 [LICENSE.md](https://github.com/4ian/GDevelop/blob/4d867e222bc7c462ca310425c95080b28b403a8d/LICENSE.md) 将 MIT 授权限定到列明的核心、引擎、IDE 和扩展目录；未找到根 README 许可依据。
- anthropics/skills：固定提交 `ca45fc496a3436251f191354091d557da31c15b5` 的仓库根及根 README 未发现通用许可；子目录技能许可不能自动扩大到根 README。
- laurent22/joplin：固定提交 `18ffcff40ea18687a70a82fd193d2ef6e88cead9` 的 [LICENSE](https://github.com/laurent22/joplin/blob/18ffcff40ea18687a70a82fd193d2ef6e88cead9/LICENSE) 明确将默认 AGPL 适用于仓库代码；根 README 的复制、翻译授权仍待核实，Logo 也另有限制。
- beyond-all-reason/Beyond-All-Reason：固定提交 `1d900169cdafe4867b16b076b85d8a67c84176a7` 的 [LICENSE.md](https://github.com/beyond-all-reason/Beyond-All-Reason/blob/1d900169cdafe4867b16b076b85d8a67c84176a7/LICENSE.md) 对代码、模型、纹理、动画及其他美术分别规定许可；未找到根 README 明确许可。其他美术保留权利的条款不推断为 README 条款。
