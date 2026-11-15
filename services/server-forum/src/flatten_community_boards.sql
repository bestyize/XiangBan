LOCK TABLE categories, topics, editor_drafts IN SHARE ROW EXCLUSIVE MODE;

CREATE TEMP TABLE board_lineage ON COMMIT DROP AS
WITH RECURSIVE lineage AS (
    SELECT id, id AS root_id, slug AS root_slug, status = 'active' AS active,
           posting_enabled AS posting_open
    FROM categories WHERE parent_id IS NULL
    UNION ALL
    SELECT c.id, p.root_id, p.root_slug, p.active AND c.status = 'active',
           p.posting_open AND c.posting_enabled
    FROM categories c JOIN lineage p ON p.id = c.parent_id
)
SELECT lineage.*, c.slug, c.section, c.parent_id, c.on_home
FROM lineage JOIN categories c USING (id)
WHERE c.section IN ('ideas', 'discuss', 'site');

-- 分类页可直接进入的创意和休闲板块保留自己的 ID。
UPDATE categories c
SET parent_id = NULL,
    status = CASE WHEN old.active THEN 'active' ELSE 'archived' END,
    posting_enabled = old.posting_open,
    allow_short_title = c.allow_short_title OR (old.root_slug = 'casual' AND old.section = 'discuss')
FROM board_lineage old
WHERE c.id = old.id
  AND (old.section = 'ideas' OR (old.section = 'discuss' AND old.root_slug = 'casual'));

CREATE TEMP TABLE merged_boards ON COMMIT DROP AS
SELECT id, root_id AS destination_id, slug, section, active, posting_open, on_home
FROM board_lineage
WHERE section IN ('discuss', 'site') AND id <> root_id AND root_slug <> 'casual'
UNION ALL
SELECT id, (SELECT id FROM categories WHERE slug = 'ideas-uncategorized'),
       slug, section, active, posting_open, on_home
FROM board_lineage old
WHERE section = 'ideas' AND slug <> 'ideas-uncategorized'
  AND (slug = 'create' OR EXISTS (SELECT 1 FROM board_lineage child WHERE child.parent_id = old.id));

-- 合并后的板块仍允许原先有权限的成员发帖。旧帖保留原有列表可见性。
UPDATE topics t SET on_home = false
FROM categories c
WHERE t.category_id = c.id AND NOT c.on_home
  AND c.id IN (SELECT destination_id FROM merged_boards WHERE section IN ('discuss', 'site'));

WITH settings AS (
    SELECT m.destination_id, bool_or(m.on_home) AS on_home,
           min(CASE WHEN src.admin_only THEN 3 ELSE src.min_post_level END)
               FILTER (WHERE m.active AND m.posting_open) AS min_post_level,
           bool_and(src.admin_only) FILTER (WHERE m.active AND m.posting_open) AS admin_only
    FROM merged_boards m JOIN categories src ON src.id = m.id
    WHERE m.section IN ('discuss', 'site')
    GROUP BY m.destination_id
)
UPDATE categories c
SET on_home = c.on_home OR s.on_home,
    posting_enabled = s.min_post_level IS NOT NULL,
    min_post_level = COALESCE(s.min_post_level, c.min_post_level),
    admin_only = COALESCE(s.admin_only, c.admin_only)
FROM settings s WHERE c.id = s.destination_id;

UPDATE topics t
SET category_id = m.destination_id,
    visible_in_active_lists = t.visible_in_active_lists AND m.active,
    on_home = t.on_home AND m.on_home
FROM merged_boards m WHERE t.category_id = m.id;

UPDATE editor_drafts d SET category_id = m.destination_id
FROM merged_boards m WHERE d.category_id = m.id;

INSERT INTO category_redirects (old_id, old_slug, category_id)
SELECT id, slug, destination_id FROM merged_boards;

DELETE FROM categories WHERE id IN (SELECT id FROM merged_boards);

DELETE FROM categories c
WHERE c.slug = 'casual' AND c.section = 'discuss'
  AND EXISTS (SELECT 1 FROM board_lineage b WHERE b.root_id = c.id AND b.id <> c.id)
  AND NOT EXISTS (SELECT 1 FROM topics WHERE category_id = c.id)
  AND NOT EXISTS (SELECT 1 FROM editor_drafts WHERE category_id = c.id);

UPDATE topics t SET pin_scope = 'category'
FROM categories c
WHERE t.category_id = c.id AND c.section IN ('ideas', 'discuss', 'site') AND t.pin_scope = 'parent';

ALTER TABLE categories ADD CONSTRAINT community_boards_without_parent
    CHECK (section = 'resources' OR parent_id IS NULL);
ALTER TABLE categories ADD CONSTRAINT categories_id_section_key UNIQUE (id, section);
ALTER TABLE categories ADD CONSTRAINT category_parent_in_section
    FOREIGN KEY (parent_id, section) REFERENCES categories (id, section) DEFERRABLE INITIALLY DEFERRED;
