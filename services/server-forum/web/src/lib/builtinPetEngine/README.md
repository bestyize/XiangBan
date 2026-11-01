本目录由 `scripts/sync-builtin-pets.mjs` 从桌面纯伙伴渲染与动画代码同步。保留官方 Animator、Mind、PetView 及图形实现；bridge.ts 只包含预览所需的类型，不接入原生窗口、账号或网络能力。详情按需加载，组件卸载时销毁动画与监听器。修改源文件后运行 `npm run sync:builtins` 更新。
