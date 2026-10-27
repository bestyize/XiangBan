# 路由发现

- core：主页骨架。通过 phone_routes 读取。
- flows/publishing：主页 → 发布方式面板 → 相册 → 图片编辑 → 正文编辑 → 结果。通过 phone_routes 按需加载。

小米 17 / Android 16 / 9.51.0 已验证到正文编辑，提交和结果仍是未验收参考。以实时页面验证入口；限时活动只记录到带到期时间的 temporary 补丁，默认不纳入主导航。
