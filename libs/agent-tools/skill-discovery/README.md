# Agent Tools / Skill Discovery

\`skill_search\`, \`skill_inspect\`, \`skill_install\`: official Tencent SkillHub Open API.
Requires user approval for each installation (always_ask). The model must inspect a **pinned version** and supply the reviewed SHA256. Source is untrusted.

First release intentionally supports only single-file SKILL.md packages; executable/multi-file skills are refused, not silently installed. Existing local skills never overwritten. Search uses https://api.skillhub.cn/api/skills and inspect uses /api/v1/skills/{slug}/files and /file.
