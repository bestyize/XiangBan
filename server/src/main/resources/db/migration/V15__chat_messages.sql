-- 小伴伴聊对话记录：保存用户与小伴的对话历史，支持清空和注销时随账号级联删除
create table chat_messages (
    id         bigserial primary key,
    user_id    bigint      not null references users (id) on delete cascade,
    role       text        not null check (role in ('user', 'assistant')),
    content    text        not null,
    created_at timestamptz not null default now()
);

create index chat_messages_user_recent on chat_messages (user_id, created_at asc);
