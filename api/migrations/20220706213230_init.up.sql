create extension if not exists "uuid-ossp";

-- users
create schema "identity";

create table "identity".users (
    id uuid not null primary key default uuid_generate_v4(),
    created_at timestamptz not null default now()
);

-- game templates
create schema bingo;

create table bingo.game_templates (
    id uuid not null primary key default uuid_generate_v4(),
    title text not null default 'unknown' check (
        length(title) > 0
        and length(title) <= 128
    ),
    "language" text check (
        language = 'ger'
        or language = 'eng'
    ),
    public boolean not null default false,
    approved boolean not null default false,
    created_by uuid not null,
    created_at timestamptz not null default now()
);

alter table
    bingo.game_templates
add
    constraint created_by_fkey foreign key (created_by) references "identity".users (id);

-- field templates
create table bingo.field_templates (
    id uuid not null primary key default uuid_generate_v4(),
    game_template_id uuid not null,
    caption text not null check (
        length(trim(caption)) > 0
        and length(trim(caption)) <= 128
    )
);

alter table
    bingo.field_templates
add
    constraint template_id_fkey foreign key (game_template_id) references bingo.game_templates (id);

-- games
create table bingo.games (
    id uuid not null primary key default uuid_generate_v4(),
    access_code text not null check (
        length(trim(access_code)) > 0
        and length(trim(access_code)) < 64
    ),
    closed boolean not null default false,
    created_at timestamptz not null default now(),
    created_by uuid not null
);

alter table
    bingo.games
add
    constraint games_unique_access_code unique (access_code),
add
    constraint games_created_by_fkey foreign key (created_by) references "identity".users (id);

-- fileds
create table bingo.fields (
    id uuid not null primary key default uuid_generate_v4(),
    game_id uuid not null,
    field_template_id uuid not null,
    checked boolean not null default false,
    "user_id" uuid not null
);

alter table
    bingo.fields
add
    constraint game_id_fkey foreign key (game_id) references bingo.games (id),
add
    constraint field_template_id_fkey foreign key (field_template_id) references bingo.field_templates (id),
add
    constraint "user_id_fkey" foreign key ("user_id") references "identity".users (id);

create unique index fields_game_field_template_user on bingo.fields using btree(game_id, field_template_id, "user_id");
