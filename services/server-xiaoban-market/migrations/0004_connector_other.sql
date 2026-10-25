-- 固定兜底分类，插件导入不会按上游文本新建分类。
INSERT INTO categories (kind, slug, name, sort_order)
VALUES ('connector', 'other', '其他', 1000)
ON CONFLICT (kind, slug) DO NOTHING;
