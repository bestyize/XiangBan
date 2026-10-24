-- 专家团（多个专家组成）与套件（一组技能打包安装）：仍属于专家 / 技能，用 variant 区分。
-- 成员以 {namespace, slug} 引用其它条目，存在 metadata.members 里，读取时解析。
ALTER TABLE items ADD COLUMN variant TEXT NOT NULL DEFAULT 'single' CHECK (variant IN ('single', 'team', 'bundle'));
ALTER TABLE items ADD CONSTRAINT items_variant_kind CHECK (
    variant = 'single' OR (variant = 'team' AND kind = 'expert') OR (variant = 'bundle' AND kind = 'skill')
);
CREATE INDEX items_variant_idx ON items (kind, variant, status);
