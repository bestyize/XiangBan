# 路由发现

- core：主页骨架。通过 phone_routes 读取。
- flows/publishing：主页 → 相册 → 编辑 → 结果。通过 phone_routes 按需加载。

当前路径为低可信度种子参考。以实时页面验证入口；限时活动只记录到带到期时间的 temporary 补丁，默认不纳入主导航。
